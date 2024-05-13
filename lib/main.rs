//! Implements a custom asset io loader.
//! An [`AssetReader`] is what the asset server uses to read the raw bytes of assets.
//! It does not know anything about the asset formats, only how to talk to the underlying storage.

use bevy::{
    asset::AssetMetaCheck,
    math::vec2,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use fragment::{Globals, PostProcessPlugin, FragmentSettings};

mod fragment;
mod js_reader;

use js_reader::CustomAssetReaderPlugin;
// use web_sys::HtmlCanvasElement;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((
            CustomAssetReaderPlugin,
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // canvas: Some("#bevy_shade_canvas".into()),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..Default::default()
                }),
            PostProcessPlugin,
        ))
        .add_systems(Startup, setup)
        // .add_systems(PostUpdate, fit_canvas_to_parent)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window_query: Query<&Window>,
) {
    let res = &window_query.single().resolution;
    let x = res.physical_width();
    let y = res.physical_height();

    // camera
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
              hdr: true,
                ..default()
            },
            ..default() },
        Globals {
            resolution: vec2(x as f32, y as f32),
            ..default()
        },
        FragmentSettings {
            reset: false,
        }
    ));

    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Circle::new(100.0))),
        material: materials.add(Color::rgb(1.0, 1.0, 1.0)),
        ..default()
    });
}
