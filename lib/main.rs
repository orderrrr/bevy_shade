use bevy::{asset::AssetMetaCheck, prelude::*};
use shaders::{compute::OCTreeComputePlugin, fragment::{FragmentPlugin, FragmentSettings}};

mod js_reader;
mod shaders;

use js_reader::CustomAssetReaderPlugin;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((
            CustomAssetReaderPlugin,
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas: Some("#bevy_shade_canvas".into()),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..Default::default()
                }),
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
