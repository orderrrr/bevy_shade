//! Implements a custom asset io loader.
//! An [`AssetReader`] is what the asset server uses to read the raw bytes of assets.
//! It does not know anything about the asset formats, only how to talk to the underlying storage.

use bevy::{
    asset::{
        io::{AssetReader, AssetReaderError, AssetSource, AssetSourceId, PathStream, Reader},
        AssetMetaCheck,
    },
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::BoxedFuture,
    window::PrimaryWindow,
};
use fragment::{PostProcessPlugin, PostProcessSettings};
use std::path::Path;

mod fragment;
mod js_reader;

use js_reader::JsWasmAssetReader;

/// A custom asset reader implementation that wraps a given asset reader implementation
struct CustomAssetReader(Box<dyn AssetReader>);

impl AssetReader for CustomAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        info!("Reading {:?}", path);
        self.0.read(path)
    }
    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        self.0.read_meta(path)
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
        self.0.read_directory(path)
    }

    fn is_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
        self.0.is_directory(path)
    }
}

/// A plugins that registers our new asset reader
struct CustomAssetReaderPlugin;

impl Plugin for CustomAssetReaderPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_source(
            AssetSourceId::Default,
            AssetSource::build().with_reader(|| {
                Box::new(CustomAssetReader(
                    // This is the default reader for the current platform
                    Box::new(JsWasmAssetReader::new("assets".to_string())),
                ))
            }),
        );
    }
}

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((
            CustomAssetReaderPlugin,
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#bevy_shade_canvas".into()),
                    ..default()
                }),
                ..default()
            }),
            PostProcessPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 5.0))
                .looking_at(Vec3::default(), Vec3::Y),
            camera: Camera {
                clear_color: Color::WHITE.into(),
                ..default()
            },
            ..default()
        },
        // Add the setting to the camera.
        // This component is also used to determine on which camera to run the post processing effect.
        PostProcessSettings {
            intensity: 0.02,
            ..default()
        },
    ));

    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
    ));
    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1_000.,
            ..default()
        },
        ..default()
    });
}
