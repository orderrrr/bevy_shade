use std::path::{Path, PathBuf};
use std::pin::{pin, Pin};
use std::task::Poll;

// import a JS function called `foo` from the module `mod`
use bevy::asset::io::{AssetReader, AssetReaderError, Reader};
use bevy::asset::io::{PathStream, VecReader};
use bevy::log::error;
use bevy::tasks::futures_lite::Stream;
use bevy::utils::BoxedFuture;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(module = "/static/js_reader.js")]
extern "C" {
    fn fetch_shader(path: &str) -> JsValue;
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
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async move {
            self.fetch_bytes(path.to_string_lossy().to_string()).await
        })
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async move {
            let meta_path = get_meta_path(&path);
            Ok(self.fetch_bytes(meta_path.to_string_lossy().to_string()).await?)
        })
    }

    fn read_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
        let stream: Box<PathStream> = Box::new(EmptyPathStream);
        error!("Reading directories is not supported with the HttpWasmAssetReader");
        Box::pin(async move { Ok(stream) })
    }

    fn is_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> BoxedFuture<'a, std::result::Result<bool, AssetReaderError>> {
        error!("Reading directories is not supported with the HttpWasmAssetReader");
        Box::pin(async move { Ok(false) })
    }
}
