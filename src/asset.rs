use anyhow::{Context, Result, anyhow, bail};
use bevy::{
    asset::{Asset, AssetLoader, LoadContext, io::Reader},
    ecs::component::Tick,
    reflect::TypePath,
};
use wasmtime::component::{Component, InstancePre, Val};

use crate::{
    engine::Engine,
    host::WasmHost,
    runner::{Config, ConfigSetup, Runner},
};

/// An asset representing a loaded wasvy Mod
#[derive(Asset, TypePath)]
pub struct ModAsset {
    pub(crate) version: Tick,
    instance_pre: InstancePre<WasmHost>,
}

const SETUP: &'static str = "setup";

impl ModAsset {
    pub(crate) async fn new(loader: &ModAssetLoader, reader: &mut dyn Reader) -> Result<Self> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;

        let component = Component::from_binary(&loader.linker.engine(), &bytes)?;
        let instance_pre = loader.linker.instantiate_pre(&component)?;

        Ok(Self {
            version: Tick::MAX,
            instance_pre,
        })
    }

    fn call(
        &self,
        runner: &mut Runner,
        config: Config,
        name: &str,
        params: &[Val],
    ) -> Result<Vec<Val>> {
        runner.use_store(config, move |mut store| {
            let instance = self
                .instance_pre
                .instantiate(&mut store)
                .context("Failed to instantiate component")?;

            let func = instance
                .get_func(&mut store, name)
                .ok_or(anyhow!("Missing {} function", name))?;

            let mut results = vec![];
            func.call(&mut store, params, &mut results)
                .expect("failed to run the desired function");

            Ok(results)
        })
    }

    pub(crate) fn setup(&self, runner: &mut Runner, config: ConfigSetup<'_>) -> Result<()> {
        let results = self.call(runner, Config::Setup(config), SETUP, &[])?;

        if !results.is_empty() {
            bail!("Mod setup returned values: {:?}, expected []", results);
        }

        Ok(())
    }

    pub(crate) fn run_system(&self, runner: &mut Runner, name: &str) -> Result<Vec<Val>> {
        self.call(runner, Config::RunSystem, name, &[])
    }
}

/// The bevy [`AssetLoader`] for [`ModAsset`]
pub struct ModAssetLoader {
    linker: wasmtime::component::Linker<WasmHost>,
}

impl ModAssetLoader {
    pub(crate) fn new(engine: &Engine) -> Self {
        let engine = engine.inner();

        let mut linker: wasmtime::component::Linker<WasmHost> =
            wasmtime::component::Linker::new(engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).unwrap();

        type Data = wasmtime::component::HasSelf<WasmHost>;
        crate::bindings::wasvy::ecs::app::add_to_linker::<_, Data>(&mut linker, |state| state)
            .unwrap();

        Self { linker }
    }
}

impl AssetLoader for ModAssetLoader {
    type Asset = ModAsset;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset> {
        let asset = ModAsset::new(self, reader).await?;

        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["wasm"]
    }
}
