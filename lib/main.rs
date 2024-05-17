use bevy::{asset::AssetMetaCheck, prelude::*};
use shaders::{
    compute::{MainWorldReceiver, OCTreeComputePlugin},
    fragment::{FragmentPlugin, FragmentSettings},
};

mod js_reader;
mod shaders;

use js_reader::CustomAssetReaderPlugin;

fn main() {
    App::new()
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
                    meta_check: AssetMetaCheck::Never,
                    ..Default::default()
                }),
            OCTreeComputePlugin,
            FragmentPlugin,
        ))
        .add_systems(Startup, setup)
        // .add_systems(FixedUpdate, receive)
        .insert_resource(Time::<Fixed>::from_seconds(10. /* one minute */))
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

/// This system will poll the channel and try to get the data sent from the render world
fn receive(receiver: Res<MainWorldReceiver>) {
    // We don't want to block the main world on this,
    // so we use try_recv which attempts to receive without blocking
    if let Ok(data) = receiver.try_recv() {
        info!("Received data from render world: {data:?}");
    }
}
