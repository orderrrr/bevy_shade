use bevy::{
    app::{App, Plugin},
    asset::{load_internal_asset, Handle},
    ecs::{
        event::{Event, EventWriter},
        reflect::ReflectResource,
        schedule::{
            common_conditions::{not, resource_exists, run_once},
            IntoSystemConfigs,
        },
        system::{Commands, Res, ResMut, Resource},
    },
    reflect::{std_traits::ReflectDefault, Reflect},
    render::{
        extract_resource::ExtractResource,
        render_resource::{Shader, ShaderType, UniformBuffer},
        renderer::{RenderDevice, RenderQueue},
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

pub const GLOBALS_TYPE_HANDLE: Handle<Shader> = Handle::weak_from_u128(093847598037245798);

pub struct OCTreeSettingsPlugin;

impl Plugin for OCTreeSettingsPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            GLOBALS_TYPE_HANDLE,
            "octree_types.wgsl",
            Shader::from_wgsl
        );

        app.register_type::<OCTreeUniform>();

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_event::<OCTreeBufferReady>()
                .add_systems(ExtractSchedule, extract_octree_settings)
                .add_systems(
                    Render,
                    (
                        construct_buffer
                            .in_set(RenderSet::PrepareResources)
                            .run_if(not(resource_exists::<OCTreeBuffer>)),
                        prepare_octree_buffer
                            .in_set(RenderSet::PrepareResources)
                            .run_if(resource_exists::<OCTreeBuffer>)
                            .run_if(run_once()),
                        octree_ready.run_if(run_once()),
                    )
                        .chain(),
                );
        }
    }
}

fn extract_octree_settings(mut commands: Commands, s: Extract<Res<OCTreeSettings>>) {
    commands.insert_resource(**s);
}

#[derive(Copy, Default, Clone, Resource, ExtractResource, Reflect, ShaderType)]
#[reflect(Resource, Default)]
pub struct OCTreeSettings {
    pub depth: u32,
    pub scale: f32,
}

/// Contains global values useful when writing shaders.
/// Currently only contains values related to time.
#[derive(Default, Clone, Resource, ExtractResource, Reflect, ShaderType)]
#[reflect(Resource, Default)]
pub struct OCTreeUniform {
    pub depth: u32,
    pub scale: f32,
}

/// Contains global values useful when writing shaders.
/// Currently only contains values related to time.
#[derive(Default, Clone, Resource, ExtractResource, Reflect, ShaderType)]
#[reflect(Resource, Default)]
pub struct OCTreeRuntime {
    depth: u32,
}

/// The buffer containing the [`GlobalsUniform`]
#[derive(Resource)]
pub struct OCTreeBuffer {
    pub buffer: UniformBuffer<OCTreeUniform>,
    pub runtime_buffer: Vec<UniformBuffer<OCTreeRuntime>>,
}

fn construct_buffer(mut commands: Commands, settings: Res<OCTreeSettings>) {
    let buffer = UniformBuffer::from(OCTreeUniform {
        depth: settings.depth,
        scale: settings.scale,
    });

    let values = (0..settings.depth + 1)
        .into_iter()
        .map(|i| UniformBuffer::from(OCTreeRuntime { depth: i }))
        .collect();

    commands.insert_resource(OCTreeBuffer {
        buffer,
        runtime_buffer: values,
    });
}

fn prepare_octree_buffer(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut octree_buffer: ResMut<OCTreeBuffer>,
    settings: Res<OCTreeSettings>,
) {
    let buffer = octree_buffer.buffer.get_mut();
    buffer.depth = settings.depth;
    buffer.scale = settings.scale;

    (0..settings.depth + 1).into_iter().for_each(|i| {
        let rb = octree_buffer.runtime_buffer[i as usize].get_mut();
        rb.depth = i;
    });

    octree_buffer
        .buffer
        .write_buffer(&render_device, &render_queue);
    octree_buffer
        .runtime_buffer
        .iter_mut()
        .for_each(|b| b.write_buffer(&render_device, &render_queue));
}

fn octree_ready(mut event_writer: EventWriter<OCTreeBufferReady>) {
    event_writer.send(OCTreeBufferReady);
}

#[derive(Event)]
pub struct OCTreeBufferReady;
