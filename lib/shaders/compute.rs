use bevy::{
    prelude::*,
    render::{
        render_graph::*,
        render_resource::{BindGroup, BindGroupLayout, *},
        renderer::{RenderContext, RenderDevice},
        Render, RenderApp, RenderSet,
    },
};
use std::borrow::Cow;

use crate::shaders::OCTree;

use super::{OCTreeData, Voxel};

const WORKGROUP_SIZE_X: u32 = 8;
const WORKGROUP_SIZE_Y: u32 = 8;

pub struct OCTreeComputePlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeLabel;

impl Plugin for OCTreeComputePlugin {
    fn build(&self, app: &mut App) {
        // Extract the game of life image resource from the main world into the render world
        // for operation on by the compute shader and display on the sprite.
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
        );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(ComputeLabel, ComputeNode::default());
        render_graph.add_node_edge(ComputeLabel, bevy::render::graph::CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.insert_resource(OCTreeData::default());
        render_app.init_resource::<ComputePipeline>();
    }
}

#[derive(Resource)]
struct ComputeBindGroup(BindGroup);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputePipeline>,
    octree_data: Res<OCTreeData>,
    render_device: Res<RenderDevice>,
) {
    let octree = &octree_data.octree;
    let voxels = &octree_data.voxels;

    info!("about to unwrap");

    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.octree_bind_group_layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: octree.buffer().unwrap(),
                    offset: 0,
                    size: None,
                }),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: voxels.buffer().unwrap(),
                    offset: 0,
                    size: None,
                }),
            },
        ],
    );

    info!("done with unwrap");

    commands.insert_resource(ComputeBindGroup(bind_group));
}

#[derive(Resource)]
struct ComputePipeline {
    octree_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // TODO - DO this
        // let limits = render_device.limits();
        // bevy::log::info!(
        //     "GPU limits:\n- max_compute_invocations_per_workgroup={}\n- max_compute_workgroup_size_x={}\n- max_compute_workgroup_size_y={}\n- max_compute_workgroup_size_z={}\n- max_compute_workgroups_per_dimension={}\n- min_storage_buffer_offset_alignment={}",
        //     limits.max_compute_invocations_per_workgroup, limits.max_compute_workgroup_size_x, limits.max_compute_workgroup_size_y, limits.max_compute_workgroup_size_z,
        //     limits.max_compute_workgroups_per_dimension, limits.min_storage_buffer_offset_alignment
        // );

        let octree_bind_group_layout = render_device.create_bind_group_layout(
            "octree:bind_group_layout",
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(OCTree::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(Voxel::min_size()),
                    },
                    count: None,
                },
            ],
        );

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/compute.wgsl"); // TODO rename

        let pipeline_cache = world.resource::<PipelineCache>();

        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
        });

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        ComputePipeline {
            octree_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

enum ComputeState {
    Loading,
    Init,
    Update,
}

struct ComputeNode {
    state: ComputeState,
}
impl ComputeNode {
    fn update_state(&mut self, pipeline_cache: &PipelineCache, pipeline: &ComputePipeline) {
        match self.state {
            ComputeState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = ComputeState::Init;
                }
            }
            ComputeState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = ComputeState::Update;
                }
            }
            ComputeState::Update => {}
        };
    }
}

impl Default for ComputeNode {
    fn default() -> Self {
        Self {
            state: ComputeState::Loading,
        }
    }
}

impl Node for ComputeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<ComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        {
            self.update_state(pipeline_cache, pipeline);
        }
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ComputePipeline>();
        let compute_bind_group = &world.resource::<ComputeBindGroup>().0;

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, compute_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            ComputeState::Loading => {}
            ComputeState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, 1);
            }
            ComputeState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, 1);
            }
        }

        Ok(())
    }
}
