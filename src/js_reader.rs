use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::Poll;

// import a JS function called `foo` from the module `mod`
use crate::AssetReader;
use bevy::asset::io::PathStream;
use bevy::asset::io::{AssetReader, AssetReaderError, Reader, get_meta_path};
use bevy::log::error;
use bevy::tasks::futures_lite::Stream;

use crate::PathStream;

#[link(wasm_import_module = "shade")]
extern "C" {
    fn get_shader(path: String) -> String;
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
    async fn fetch_bytes<'a>(&self, path: PathBuf) -> Result<Box<Reader<'a>>, AssetReaderError> {
        // TODO

        error!("TODO");
    }
}

impl AssetReader for HttpWasmAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<Box<Reader<'a>>, AssetReaderError> {
        let path = self.root_path.join(path);
        self.fetch_bytes(path).await
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<Box<Reader<'a>>, AssetReaderError> {
        let meta_path = get_meta_path(&self.root_path.join(path));
        self.fetch_bytes(meta_path).await
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
