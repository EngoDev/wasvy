use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use thiserror::Error;
use wasmtime::Engine;

#[derive(Asset, TypePath, Clone)]
pub struct WasmComponentAsset {
    pub component: wasmtime::component::Component,
}

pub struct WasmComponentAssetLoader {
    pub engine: Engine,
}

/// Possible errors that can be produced by [`WasmComponentAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum WasmComponentAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for WasmComponentAssetLoader {
    type Asset = WasmComponentAsset;
    type Settings = ();
    type Error = WasmComponentAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await.unwrap();
        let component = wasmtime::component::Component::from_binary(&self.engine, &bytes).unwrap();

        Ok(WasmComponentAsset { component })
    }

    fn extensions(&self) -> &[&str] {
        &["wasm"]
    }
}
