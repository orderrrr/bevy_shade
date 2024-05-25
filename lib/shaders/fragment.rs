use bevy::core::FrameCount;
use bevy::render::camera::ExtractedCamera;
use bevy::render::globals::{GlobalsBuffer, GlobalsUniform};
use bevy::render::render_resource::binding_types::storage_buffer_read_only;
use bevy::render::texture::{CachedTexture, TextureCache};
use bevy::render::view::ExtractedView;
use bevy::render::{MainWorld, Render, RenderSet};
use bevy::{
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        texture::BevyDefault,
        view::ViewTarget,
        RenderApp,
    },
};

use super::compute::ComputeBuffers;
use super::octree::settings_plugin::{OCTreeBuffer, OCTreeUniform};
use super::{OCTree, Voxel};

pub const FRAGMENT_001: &str = "shaders/fragment.wgsl";

// pub const BRICK_RES: i32 = 8_i32.pow(3);
// pub const OCTREE_SMALLEST: i32 = 8;
// pub const OCTREE_MAX_DEPTH: i32 = 1;

/// It is generally encouraged to set up post-processing effects as a plugin
pub struct FragmentPlugin;

impl Plugin for FragmentPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FragmentSettings>().add_plugins(());

        // We need to get the render app from the main app
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            // Bevy's renderer uses a render graph which is a collection of nodes in a directed acyclic graph.
            // It currently runs on each view/camera and executes each node in the specified order.
            // It will make sure that any node that needs a dependency from another node
            // only runs when that dependency is done.
            //
            // Each node can execute arbitrary work, but it generally runs at least one render pass.
            // A node only has access to the render world, so if you need data from the main world
            // you need to extract it manually or with the plugin like above.
            // Add a [`Node`] to the [`RenderGraph`]
            // The Node needs to impl FromWorld
            //
            // The [`ViewNodeRunner`] is a special [`Node`] that will automatically run the node for each view
            // matching the [`ViewQuery`]
            .init_resource::<SpecializedRenderPipelines<FragmentPipeline>>()
            .add_systems(ExtractSchedule, extract_fragment_settings)
            .add_systems(
                Render,
                (
                    prepare_fragment_pipelines.in_set(RenderSet::Prepare),
                    prepare_fragment_history_textures.in_set(RenderSet::PrepareResources),
                ),
            )
            .add_render_graph_node::<ViewNodeRunner<FragmentNode>>(
                // Specify the label of the graph, in this case we want the graph for 3d
                Core2d,
                // It also needs the label of the node
                FragmentLabel,
            )
            .add_render_graph_edges(
                Core2d,
                // Specify the node ordering.
                // This will automatically create all required node edges to enforce the given ordering.
                (Node2d::EndMainPass, FragmentLabel, Node2d::Tonemapping),
            );
    }

    fn finish(&self, app: &mut App) {
        // We need to get the render app from the main app
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            // Initialize the pipeline
            .init_resource::<FragmentPipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct FragmentLabel;

// The post process node used for the render graph
#[derive(Default)]
struct FragmentNode;

#[derive(Component)]
pub struct FragmentPipelineId(CachedRenderPipelineId);

#[derive(Component)]
pub struct FragmentHistoryTexture {
    write: CachedTexture,
    read: CachedTexture,
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct FragmentPipelineKey {
    hdr: bool,
    reset: bool,
}

fn prepare_fragment_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<FragmentPipeline>>,
    pipeline: Res<FragmentPipeline>,
    views: Query<(Entity, &ExtractedView, &FragmentSettings)>,
) {
    for (entity, view, fragment_settings) in &views {
        let mut pipeline_key = FragmentPipelineKey {
            hdr: view.hdr,
            reset: fragment_settings.reset,
        };
        let pipeline_id = pipelines.specialize(&pipeline_cache, &pipeline, pipeline_key.clone());

        // if pipeline_key.reset {
        pipeline_key.reset = false;
        pipelines.specialize(&pipeline_cache, &pipeline, pipeline_key);
        // }

        commands
            .entity(entity)
            .insert(FragmentPipelineId(pipeline_id));
    }
}

impl ViewNode for FragmentNode {
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static ViewTarget,
        &'static FragmentHistoryTexture,
        &'static FragmentPipelineId,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (camera, view_target, fragment_history_textures, fragment_pipeline_id): QueryItem<
            Self::ViewQuery,
        >,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let fragment_pipeline = world.resource::<FragmentPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let octree_settings = world.resource::<OCTreeBuffer>();

        // Get the pipeline from the cache
        let (Some(pipeline), Some(compute_buffers)) = (
            pipeline_cache.get_render_pipeline(fragment_pipeline_id.0),
            world.get_resource::<ComputeBuffers>(),
        ) else {
            return Ok(());
        };

        let globals_buffer = world.resource::<GlobalsBuffer>();
        let octree_cpu = &compute_buffers.octree_gpu;
        let voxel_cpu = &compute_buffers.voxel_gpu;

        // This will start a new "post process write", obtaining two texture
        // views from the view target - a `source` and a `destination`.
        // `source` is the "current" main texture and you _must_ write into
        // `destination` because calling `post_process_write()` on the
        // [`ViewTarget`] will internally flip the [`ViewTarget`]'s main
        // texture to the `destination` texture. Failing to do so will cause
        // the current main texture information to be lost.
        let fragment = view_target.post_process_write();

        // The bind_group gets created each frame.
        //
        // Normally, you would create a bind_group in the Queue set,
        // but this doesn't work with the post_process_write().
        // The reason it doesn't work is because each post_process_write will alternate the source/destination.
        // The only way to have the correct source/destination for the bind_group
        // is to make sure you get it during the node execution.
        let bind_group = render_context.render_device().create_bind_group(
            "fragment_group",
            &fragment_pipeline.group_layout,
            // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
            &BindGroupEntries::sequential((
                // Make sure to use the source view
                &globals_buffer.buffer,
                octree_cpu.as_entire_binding(),
                voxel_cpu.as_entire_binding(),
                &octree_settings.buffer,
                fragment.source,
                &fragment_history_textures.read.default_view,
                &fragment_pipeline.nearest_sampler,
                &fragment_pipeline.linear_sampler,
            )),
        );

        // Begin the render pass
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("fragment_pass"),
            color_attachments: &[
                Some(RenderPassColorAttachment {
                    // We need to specify the post process destination view here
                    // to make sure we write to the appropriate texture.
                    view: fragment.destination,
                    resolve_target: None,
                    ops: Operations::default(),
                }),
                Some(RenderPassColorAttachment {
                    view: &fragment_history_textures.write.default_view,
                    resolve_target: None,
                    ops: Operations::default(),
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
        // using the pipeline/bind_group created above
        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        if let Some(viewport) = camera.viewport.as_ref() {
            render_pass.set_camera_viewport(viewport);
        }
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

// This contains global data used by the render pipeline. This will be created once on startup.
#[derive(Resource)]
struct FragmentPipeline {
    group_layout: BindGroupLayout,
    shader: Handle<Shader>,
    nearest_sampler: Sampler,
    linear_sampler: Sampler,
}

impl FromWorld for FragmentPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let nearest_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("feedback_nearest_sampler"),
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..SamplerDescriptor::default()
        });
        let linear_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("feedback_linear_sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..SamplerDescriptor::default()
        });

        // We need to define the bind group layout used for our pipeline
        let group_layout = render_device.create_bind_group_layout(
            "post_process_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                // The layout entries will only be visible in the fragment stage
                ShaderStages::FRAGMENT,
                (
                    uniform_buffer::<GlobalsUniform>(false),
                    storage_buffer_read_only::<Vec<OCTree>>(false),
                    storage_buffer_read_only::<Vec<Voxel>>(false),
                    uniform_buffer::<OCTreeUniform>(false),
                    // The screen texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // The sampler that will be used to sample the screen texture
                    sampler(SamplerBindingType::NonFiltering),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        // Get the shader handle
        let shader = world.resource::<AssetServer>().load(FRAGMENT_001);

        Self {
            shader,
            group_layout,
            nearest_sampler,
            linear_sampler,
        }
    }
}

#[derive(Component, Reflect, Default, Clone)]
pub struct FragmentSettings {
    pub reset: bool,
}

fn prepare_fragment_history_textures(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    frame_count: Res<FrameCount>,
    views: Query<(Entity, &ExtractedCamera, &ExtractedView)>,
) {
    for (entity, camera, view) in &views {
        if let Some(physical_viewport_size) = camera.physical_viewport_size {
            let mut texture_descriptor = TextureDescriptor {
                label: None,
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: physical_viewport_size.x,
                    height: physical_viewport_size.y,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: if view.hdr {
                    ViewTarget::TEXTURE_FORMAT_HDR
                } else {
                    TextureFormat::bevy_default()
                }, // no hdr for now.
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            };

            texture_descriptor.label = Some("fragment_history_1_texture");
            let history_1_texture = texture_cache.get(&render_device, texture_descriptor.clone());

            texture_descriptor.label = Some("fragment_history_2_texture");
            let history_2_texture = texture_cache.get(&render_device, texture_descriptor);

            let textures = if frame_count.0 % 2 == 0 {
                FragmentHistoryTexture {
                    write: history_1_texture,
                    read: history_2_texture,
                }
            } else {
                FragmentHistoryTexture {
                    write: history_2_texture,
                    read: history_1_texture,
                }
            };

            commands.entity(entity).insert(textures);
        }
    }
}

fn extract_fragment_settings(mut commands: Commands, mut main_world: ResMut<MainWorld>) {
    let mut cameras =
        main_world.query_filtered::<(Entity, &Camera, &mut FragmentSettings), With<Camera2d>>();

    for (entity, camera, mut fragment_settings) in cameras.iter_mut(&mut main_world) {
        if camera.is_active {
            commands
                .get_or_spawn(entity)
                .insert(fragment_settings.clone());
            fragment_settings.reset = false;
        }
    }
}

impl SpecializedRenderPipeline for FragmentPipeline {
    type Key = FragmentPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = vec![];

        let format = if key.hdr {
            shader_defs.push("FRAGMENT".into());
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        if key.reset {
            shader_defs.push("RESET".into());
        }

        RenderPipelineDescriptor {
            label: Some("fragment_pipeline".into()),
            layout: vec![self.group_layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![
                    Some(ColorTargetState {
                        format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    }),
                    Some(ColorTargetState {
                        format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    }),
                ],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: Vec::new(),
        }
    }
}
