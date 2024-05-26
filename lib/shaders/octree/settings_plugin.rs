use bevy::{
    app::{App, Plugin},
    asset::{load_internal_asset, Handle},
    ecs::{
        event::{Event, EventWriter},
        reflect::ReflectResource,
        schedule::{common_conditions::run_once, IntoSystemConfigs},
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

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_event::<OCTreeBufferReady>()
                .init_resource::<OCTreeBuffer>()
                .add_systems(ExtractSchedule, extract_octree_settings)
                .add_systems(
                    Render,
                    (
                        prepare_octree_buffer.in_set(RenderSet::PrepareResources),
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
    pub current_depth: u32,
}

/// The buffer containing the [`GlobalsUniform`]
#[derive(Resource, Default)]
pub struct OCTreeBuffer {
    pub buffer: UniformBuffer<OCTreeUniform>,
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
    buffer.current_depth = settings.depth;

    octree_buffer
        .buffer
        .write_buffer(&render_device, &render_queue);
}

fn octree_ready(mut event_writer: EventWriter<OCTreeBufferReady>) {
    event_writer.send(OCTreeBufferReady);
}

#[derive(Event)]
pub struct OCTreeBufferReady;
