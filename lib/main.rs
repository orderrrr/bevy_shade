use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_shade_lib::shaders::{
    compute::{MainWorldOCTreeReceiver, OCTreeComputePlugin},
    fragment::{FragmentPlugin, FragmentSettings},
    octree::settings_plugin::{OCTreeSettings, OCTreeSettingsPlugin},
};

// use js_reader::CustomAssetReaderPlugin;

#[cfg(feature ="readback")]
fn main() {
    App::new()
        .insert_resource(OCTreeSettings {
            depth: 2,
            scale: 2.0,
        })
        .add_plugins((
            // CustomAssetReaderPlugin, // use default assets
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // canvas: Some("#bevy_shade_canvas".into()),
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
            // .set(AssetPlugin {
            //     watch_for_changes_override: Some(true),
            //     meta_check: AssetMetaCheck::Never,
            //     ..Default::default()
            // }),
            OCTreeSettingsPlugin,
            OCTreeComputePlugin,
            FragmentPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, receive)
        .run();
}

#[cfg(not(feature = "readback"))]
fn main() {
    App::new()
        .insert_resource(OCTreeSettings {
            depth: 2,
            scale: 2.0,
        })
        .add_plugins((
            // CustomAssetReaderPlugin, // use default assets
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // canvas: Some("#bevy_shade_canvas".into()),
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
            // .set(AssetPlugin {
            //     watch_for_changes_override: Some(true),
            //     meta_check: AssetMetaCheck::Never,
            //     ..Default::default()
            // }),
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

/// This system will poll the channel and try to get the data sent from the render world
fn receive(receiver: Res<MainWorldOCTreeReceiver>) {
    // We don't want to block the main world on this,
    // so we use try_recv which attempts to receive without blocking
    if let Ok(data) = receiver.try_recv() {
        info!("Received data from render world: {data:?}");
    }
}
