use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*, render::{settings::{Backends, RenderCreation, WgpuSettings}, RenderPlugin},
};
use shaders::{
    compute::OCTreeComputePlugin,
    fragment::{FragmentPlugin, FragmentSettings},
    octree::settings_plugin::{OCTreeSettings, OCTreeSettingsPlugin},
};

mod js_reader;
mod shaders;

fn main() {
    App::new()
        .insert_resource(OCTreeSettings {
            depth: 5,
            scale: 2.0,
        })
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window { ..default() }),

                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..default()
                    }),
                    ..Default::default()
                }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
            OCTreeSettingsPlugin,
            OCTreeComputePlugin,
            FragmentPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // camera
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: false,
                ..default()
            },
            ..default()
        },
        FragmentSettings { reset: false },
    ));
}