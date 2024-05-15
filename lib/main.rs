use bevy::{asset::AssetMetaCheck, prelude::*};
use fragment::{FragmentSettings, FragmentPlugin};

mod fragment;
mod js_reader;

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
