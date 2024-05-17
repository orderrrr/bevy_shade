use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::Poll;

use bevy::app::{App, Plugin};
use bevy::asset::AssetServer;
use bevy::prelude::*;
// import a JS function called `foo` from the module `mod`
use bevy::asset::io::{
    AssetReader, AssetReaderError, AssetSource, AssetSourceId, ErasedAssetReader, Reader,
};
use bevy::asset::io::{PathStream, VecReader};
use bevy::ecs::event::{Event, EventReader, EventWriter};
use bevy::ecs::system::ResMut;
use bevy::log::{error, info};
use bevy::tasks::futures_lite::Stream;
use bevy::utils::BoxedFuture;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

/// A custom asset reader implementation that wraps a given asset reader implementation
struct CustomAssetReader(Box<dyn ErasedAssetReader>);

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
pub struct CustomAssetReaderPlugin;

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
        )
        .add_event::<AssetChangeEvent>()
        .add_systems(PostUpdate, (check_update, update_assets));
    }
}

#[wasm_bindgen(module = "js_reader")]
extern "C" {
    fn fetch_shader(path: &str) -> JsValue;
    fn get_js_changes() -> Vec<JsValue>;
}

#[derive(Event)]
pub struct AssetChangeEvent(pub Vec<String>);

// TODO - put this in an event
pub fn check_update(mut ev_asset_change: EventWriter<AssetChangeEvent>) {
    let changes = get_changes();
    if !changes.is_empty() {
        info!("asset change event.");
        ev_asset_change.send(AssetChangeEvent(changes));
    }
}

pub fn update_assets(
    mut ev_asset_change_reader: EventReader<AssetChangeEvent>,
    asset_server: ResMut<AssetServer>,
) {
    for event in ev_asset_change_reader.read() {
        event.0.iter().for_each(|asset| {
            asset_server.reload(asset);
        });
    }
}

/// Reader implementation for loading assets via HTTP in WASM.
pub struct JsWasmAssetReader {
    root_path: PathBuf,
}

impl JsWasmAssetReader {
    /// Creates a new `WasmAssetReader`. The path provided will be used to build URLs to query for assets.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            root_path: path.as_ref().to_owned(),
        }
    }
}

impl JsWasmAssetReader {
    async fn fetch_bytes<'a>(&self, path: String) -> Result<Box<Reader<'a>>, AssetReaderError> {
        let res = fetch_shader(path.as_str()).as_string().unwrap();
        let reader: Box<Reader> = Box::new(VecReader::new(res.as_bytes().to_vec()));
        Ok(reader)
    }
}

pub fn get_meta_path(path: &Path) -> PathBuf {
    let mut meta_path = path.to_path_buf();
    let mut extension = path.extension().unwrap_or_default().to_os_string();
    extension.push(".meta");
    meta_path.set_extension(extension);
    meta_path
}

impl AssetReader for JsWasmAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<Box<Reader<'a>>, AssetReaderError> {
        self.fetch_bytes(path.to_string_lossy().to_string()).await
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<Box<Reader<'a>>, AssetReaderError> {
        let meta_path = get_meta_path(&path);
        self.fetch_bytes(meta_path.to_string_lossy().to_string())
            .await
    }

    async fn read_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        let stream: Box<PathStream> = Box::new(EmptyPathStream);
        error!("Reading directories is not supported with the HttpWasmAssetReader");
        Ok(stream)
    }

    async fn is_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> std::result::Result<bool, AssetReaderError> {
        error!("Reading directories is not supported with the HttpWasmAssetReader");
        Ok(false)
    }
}

pub fn get_changes() -> Vec<String> {
    let res = get_js_changes().to_vec();
    res.iter().map(|i| i.as_string().unwrap()).collect()
}

/// A [`PathBuf`] [`Stream`] implementation that immediately returns nothing.
struct EmptyPathStream;

impl Stream for EmptyPathStream {
    type Item = PathBuf;

    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Poll::Ready(None)
    }
}
