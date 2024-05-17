use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        graph::CameraDriverLabel, render_graph::*, render_resource::{binding_types::storage_buffer, BindGroup, BindGroupLayout, *}, renderer::{RenderContext, RenderDevice}, Render, RenderApp, RenderSet
    },
};
use crossbeam_channel::{Receiver, Sender};
use zerocopy::FromBytes;

use crate::shaders::OCTree;

const BUFFER_LEN: usize = 128;

pub struct OCTreeComputePlugin;

impl Plugin for OCTreeComputePlugin {
    fn build(&self, app: &mut App) {
        let (s, r) = crossbeam_channel::unbounded();
        app.insert_resource(MainWorldReceiver(r));

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .insert_resource(RenderWorldSender(s))
            .add_systems(
                Render,
                (
                    prepare_bind_group
                        .in_set(RenderSet::PrepareBindGroups)
                        // We don't need to recreate the bind group every frame
                        .run_if(not(resource_exists::<ComputeBindGroup>)),
                    // We need to run it after the render graph is done
                    // because this needs to happen after submit()
                    map_and_read_buffer.after(RenderSet::Render),
                ),
            );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(ComputeNodeLabel, ComputeNode::default());
        render_graph.add_node_edge(ComputeNodeLabel, CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {

        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ComputePipeline>();
        render_app.init_resource::<Buffers>();
    }
}

#[derive(Resource, Deref)]
pub struct MainWorldReceiver(Receiver<Vec<OCTree>>);

/// This will send asynchronously any data to the main world
#[derive(Resource, Deref)]
struct RenderWorldSender(Sender<Vec<OCTree>>);

#[derive(Resource)]
struct Buffers {
    // The buffer that will be used by the compute shader
    buffer_len: usize,
    octree_gpu: Buffer,
    octree_cpu: Buffer,
}

impl FromWorld for Buffers {
    fn from_world(world: &mut World) -> Self {
        let buffer_len = BUFFER_LEN;
        let render_device = world.resource::<RenderDevice>();

        let mut init_data = encase::StorageBuffer::new(Vec::new());
        let data = vec![OCTree::default(); buffer_len];
        init_data.write(&data).expect("failed to write buffer");

        // The buffer that will be accessed by the gpu
        let gpu_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("gpu_buffer"),
            contents: init_data.as_ref(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        });
        // For portability reasons, WebGPU draws a distinction between memory that is
        // accessible by the CPU and memory that is accessible by the GPU. Only
        // buffers accessible by the CPU can be mapped and accessed by the CPU and
        // only buffers visible to the GPU can be used in shaders. In order to get
        // data from the GPU, we need to use `CommandEncoder::copy_buffer_to_buffer` to
        // copy the buffer modified by the GPU into a mappable, CPU-accessible buffer
        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("readback_buffer"),
            size: (buffer_len * std::mem::size_of::<OCTree>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer_len,
            octree_gpu: gpu_buffer,
            octree_cpu: cpu_buffer,
        }
    }
}

#[derive(Resource)]
struct ComputeBindGroup(BindGroup);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputePipeline>,
    render_device: Res<RenderDevice>,
    buffers: Res<Buffers>,
) {
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::single(buffers.octree_gpu.as_entire_binding()),
    );
    commands.insert_resource(ComputeBindGroup(bind_group));
}

#[derive(Resource)]
struct ComputePipeline {
    layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            "ComputeOCTree",
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                storage_buffer::<Vec<OCTree>>(false),
            ),
        );
        let shader = world.load_asset("shaders/compute.wgsl"); // TODO rename
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: "main".into(),
        });
        ComputePipeline { layout, pipeline }
    }
}

fn map_and_read_buffer(
    render_device: Res<RenderDevice>,
    buffers: Res<Buffers>,
    sender: Res<RenderWorldSender>,
) {
    // Finally time to get our data back from the gpu.
    // First we get a buffer slice which represents a chunk of the buffer (which we
    // can't access yet).
    // We want the whole thing so use unbounded range.
    let buffer_slice = buffers.octree_cpu.slice(..);

    // Now things get complicated. WebGPU, for safety reasons, only allows either the GPU
    // or CPU to access a buffer's contents at a time. We need to "map" the buffer which means
    // flipping ownership of the buffer over to the CPU and making access legal. We do this
    // with `BufferSlice::map_async`.
    //
    // The problem is that map_async is not an async function so we can't await it. What
    // we need to do instead is pass in a closure that will be executed when the slice is
    // either mapped or the mapping has failed.
    //
    // The problem with this is that we don't have a reliable way to wait in the main
    // code for the buffer to be mapped and even worse, calling get_mapped_range or
    // get_mapped_range_mut prematurely will cause a panic, not return an error.
    //
    // Using channels solves this as awaiting the receiving of a message from
    // the passed closure will force the outside code to wait. It also doesn't hurt
    // if the closure finishes before the outside code catches up as the message is
    // buffered and receiving will just pick that up.
    //
    // It may also be worth noting that although on native, the usage of asynchronous
    // channels is wholly unnecessary, for the sake of portability to WASM
    // we'll use async channels that work on both native and WASM.
    
    info!("GOT TO Start of map and read");

    let (s, r) = crossbeam_channel::unbounded::<()>();

    // Maps the buffer so it can be read on the cpu
    buffer_slice.map_async(MapMode::Read, move |r| match r {
        // This will execute once the gpu is ready, so after the call to poll()
        Ok(_) => s.send(()).expect("Failed to send map update"),
        Err(err) => panic!("Failed to map buffer {err}"),
    });

    // In order for the mapping to be completed, one of three things must happen.
    // One of those can be calling `Device::poll`. This isn't necessary on the web as devices
    // are polled automatically but natively, we need to make sure this happens manually.
    // `Maintain::Wait` will cause the thread to wait on native but not on WebGpu.

    // This blocks until the gpu is done executing everything
    render_device.poll(Maintain::wait()).panic_on_timeout();

    info!("GOT TO middle of map and read");

    // This blocks until the buffer is mapped
    r.recv().expect("Failed to receive the map_async message");

    {
        let buffer_view = buffer_slice.get_mapped_range();
        let data = buffer_view
            .chunks(std::mem::size_of::<OCTree>())
            .map(|chunk| {
                OCTree::read_from(chunk.try_into().expect("should be a u32"))
                    .expect("error check here")
            })
            .collect::<Vec<OCTree>>();
        sender
            .send(data)
            .expect("Failed to send data to main world");
    }

    info!("GOT TO end of map and read");

    // We need to make sure all `BufferView`'s are dropped before we do what we're about
    // to do.
    // Unmap so that we can copy to the staging buffer in the next iteration.
    buffers.octree_cpu.unmap();
}

/// Label to identify the node in the render graph
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeNodeLabel;

/// The node that will execute the compute shader
#[derive(Default)]
struct ComputeNode;

impl Node for ComputeNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let bind_group = &world.resource::<ComputeBindGroup>().0;
        let pipeline = world.resource::<ComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let buffers = world.resource::<Buffers>();

        if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
            let mut pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("GPU readback compute pass"),
                        ..default()
                    });

            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_pipeline(init_pipeline);
            pass.dispatch_workgroups(buffers.buffer_len as u32, 1, 1);
        }

        // // Copy the gpu accessible buffer to the cpu accessible buffer
        // render_context.command_encoder().copy_buffer_to_buffer(
        //     &buffers.octree_gpu,
        //     0,
        //     &buffers.octree_cpu,
        //     0,
        //     (buffers.buffer_len * std::mem::size_of::<OCTree>()) as u64,
        // );

        Ok(())
    }
}
