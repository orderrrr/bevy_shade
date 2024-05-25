use crate::shaders::OCTree;
use bevy::{
    prelude::*,
    render::{
        globals::{GlobalsBuffer, GlobalsUniform},
        graph::CameraDriverLabel,
        render_graph::*,
        render_resource::{
            binding_types::{storage_buffer, uniform_buffer},
            BindGroup, BindGroupLayout, *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
};
use bytemuck::Zeroable;

use super::{
    octree::settings_plugin::{OCTreeBuffer, OCTreeBufferReady, OCTreeSettings, OCTreeUniform},
    Voxel,
};

pub struct OCTreeComputePlugin;

impl Plugin for OCTreeComputePlugin {
    #[allow(unused_parens)]
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (
                prepare_compute_buffers
                    .in_set(RenderSet::PrepareBindGroups)
                    .run_if(on_event::<OCTreeBufferReady>()),
                prepare_compute_pipeline
                    .in_set(RenderSet::PrepareBindGroups)
                    .run_if(on_event::<OCTreeBufferReady>()),
                prepare_bind_group
                    .in_set(RenderSet::PrepareBindGroups)
                    .run_if(not(resource_exists::<ComputeBindGroup>))
                    .run_if(resource_exists::<ComputePipeline>),
            ),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();

        graph.add_node(ComputeNodeLabel(3), ComputeNode(3));
        graph.add_node_edge(ComputeNodeLabel(3), CameraDriverLabel);

        // println!("0 runs before {:?}", CameraDriverLabel);
        // graph.add_node(ComputeNodeLabel(0), ComputeNode(0));
        // graph.add_node_edge(ComputeNodeLabel(0), CameraDriverLabel);
        //
        // for i in 1..settings.depth + 1 {
        //     println!("{} runs before {}", i, i - 1);
        //
        //     graph.add_node(ComputeNodeLabel(i), ComputeNode(i));
        //     graph.add_node_edge(ComputeNodeLabel(i), ComputeNodeLabel(i - 1));
        // }
    }
}

#[derive(Resource)]
pub struct ComputeBuffers {
    pub octree_gpu: Buffer,
    pub voxel_gpu: Buffer,
}

impl FromWorld for ComputeBuffers {
    fn from_world(world: &mut World) -> Self {
        let settings = world.resource::<OCTreeBuffer>().buffer.get();
        let render_device = world.resource::<RenderDevice>();

        info!("buffers prepared");

        let depth = settings.depth;

        let max_octree = calculate_full_depth(depth) as usize;
        let max_voxel = calculate_max_voxel(depth) as usize;

        let mut init_data = encase::StorageBuffer::new(Vec::new());
        let data = vec![OCTree::zeroed(); max_octree];
        init_data.write(&data).expect("failed to write buffer");

        // The buffer that will be accessed by the gpu
        let octree_gpu_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("octree_gpu_buffer"),
            contents: init_data.as_ref(),
            // usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC, // use this if we want to communicate with the cpu
            usage: BufferUsages::STORAGE,
        });

        let mut init_data = encase::StorageBuffer::new(Vec::new());
        let data = vec![Voxel::zeroed(); max_voxel];
        init_data.write(&data).expect("failed to write buffer");

        // The buffer that will be accessed by the gpu
        let voxel_gpu_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("voxel_gpu_buffer"),
            contents: init_data.as_ref(),
            // usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC, // use this if we want to communicate with the cpu
            usage: BufferUsages::STORAGE,
        });

        ComputeBuffers {
            octree_gpu: octree_gpu_buffer,
            voxel_gpu: voxel_gpu_buffer,
        }
    }
}

fn prepare_compute_buffers(world: &mut World) {
    let cb = ComputeBuffers::from_world(world);
    world.insert_resource(cb);
}

#[derive(Resource)]
struct ComputeBindGroup(BindGroup);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputePipeline>,
    render_device: Res<RenderDevice>,
    buffers: Res<ComputeBuffers>,
    globals: Res<GlobalsBuffer>,
    settings: Res<OCTreeBuffer>,
) {
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::sequential((
            &globals.buffer,
            &settings.buffer,
            buffers.octree_gpu.as_entire_binding(),
            buffers.voxel_gpu.as_entire_binding(),
        )),
    );

    commands.insert_resource(ComputeBindGroup(bind_group));
}

#[derive(Resource)]
struct ComputePipeline {
    layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    final_pipeline: CachedComputePipelineId,
}

fn prepare_compute_pipeline(world: &mut World) {
    let cp = ComputePipeline::from_world(world);
    world.insert_resource(cp);
}

impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let layout = world.resource::<RenderDevice>().create_bind_group_layout(
            format!("ComputeOCTree depth").as_str(),
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<GlobalsUniform>(false),
                    uniform_buffer::<OCTreeUniform>(false),
                    storage_buffer::<Vec<OCTree>>(false),
                    storage_buffer::<Vec<Voxel>>(false),
                ),
            ),
        );

        let shader = world.load_asset("shaders/compute.wgsl"); // TODO rename
        let pipeline_cache = world.resource::<PipelineCache>();

        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("octree pipeline init".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "init".into(),
        });

        let final_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("octree pipeline finalize".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "finalize".into(),
        });

        ComputePipeline {
            layout,
            init_pipeline,
            final_pipeline,
        }
    }
}

/// Label to identify the node in the render graph
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeNodeLabel(u32);

/// The node that will execute the compute shader
#[derive(Default)]
struct ComputeNode(u32);

impl Node for ComputeNode {
    fn update(&mut self, world: &mut World) {
        world.resource_scope(|world, mut octree_buffer: Mut<OCTreeBuffer>| {
            let div = world.resource::<RenderDevice>();
            let qu = world.resource::<RenderQueue>();

            let buf = octree_buffer.buffer.get_mut();
            buf.current_depth = self.0;

            octree_buffer.buffer.write_buffer(&div, &qu);
        });
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let (Some(bind_group), Some(pipeline), Some(octree_buffer)) = (
            world.get_resource::<ComputeBindGroup>(),
            world.get_resource::<ComputePipeline>(),
            world.get_resource::<OCTreeBuffer>(),
        ) else {
            return Ok(());
        };

        let pipeline_cache = world.resource::<PipelineCache>();

        let depth = self.0;

        let size = calculate_current_size(depth); // first pass, populate data.

        // if depth > 2 {
        //     println!("!!!!!!!!!!!!");
        // }
        // println!("DEPTH IS: {}", depth);
        // println!("SIZE IS: {}", size);

        if let Some(target_pipeline) =
            pipeline_cache.get_compute_pipeline(if octree_buffer.buffer.get().depth == self.0 {
                pipeline.init_pipeline
            } else {
                pipeline.final_pipeline
            })
        {
            let mut pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("GPU readback compute pass"),
                        ..default()
                    });

            pass.set_bind_group(0, &bind_group.0, &[]);
            pass.set_pipeline(target_pipeline);
            pass.dispatch_workgroups(size, size, size);
        }

        Ok(())
    }
}

pub fn calculate_full_depth(depth: u32) -> u32 {
    ((8_f64.powf((depth + 1) as f64) - 1.) / 7.) as u32
}

pub fn calculate_max_voxel(depth: u32) -> u32 {
    8_f64.powf(depth as f64 + 1.) as u32
}

pub fn calculate_current_size(depth: u32) -> u32 {
    if depth == 0 {
        return 1; // first octree is 1.
    }

    2_f64.powf(depth as f64) as u32
}
