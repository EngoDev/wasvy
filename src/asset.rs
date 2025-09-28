use std::mem::replace;

use anyhow::{Context, Result, anyhow, bail};
use bevy::{
    asset::{Asset, AssetId, AssetLoader, LoadContext, io::Reader},
    ecs::{component::Tick, world::World},
    reflect::TypePath,
};
use wasmtime::component::{Component, InstancePre, Val};

use crate::{
    engine::{Engine, Linker},
    host::WasmHost,
    runner::{Config, ConfigRunSystem, ConfigSetup, Runner},
};

/// An asset representing a loaded wasvy Mod
#[derive(Asset, TypePath)]
pub struct ModAsset(Inner);

enum Inner {
    Placeholder,
    Loaded {
        instance_pre: InstancePre<WasmHost>,
    },
    Ready {
        version: Tick,
        instance_pre: InstancePre<WasmHost>,
    },
}

const SETUP: &'static str = "setup";

impl ModAsset {
    pub(crate) async fn new(loader: &ModAssetLoader, reader: &mut dyn Reader) -> Result<Self> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;

        let component = Component::from_binary(&loader.linker.engine(), &bytes)?;
        let instance_pre = loader.linker.instantiate_pre(&component)?;

        Ok(Self(Inner::Loaded { instance_pre }))
    }

    pub(crate) fn version(&self) -> Option<Tick> {
        match &self.0 {
            Inner::Ready { version, .. } => Some(version.clone()),
            _ => None,
        }
    }

    /// Take this asset and leave a placeholder behind
    pub(crate) fn take(&mut self) -> Self {
        replace(self, Self(Inner::Placeholder))
    }

    /// Take this asset and leave a placeholder behind
    pub(crate) fn put(&mut self, value: Self) {
        let _ = replace(self, value);
    }

    pub(crate) fn setup(
        self,
        world: &mut World,
        asset_id: &AssetId<ModAsset>,
        mod_name: &str,
    ) -> Result<Self> {
        let Inner::Loaded { instance_pre } = self.0 else {
            bail!("Mod is not in Loaded state");
        };

        let version = world.change_tick();

        let engine = world.get_resource::<Engine>().unwrap();
        let mut runner = Runner::new(&engine);
        let results = call(
            &mut runner,
            &instance_pre,
            Config::Setup(ConfigSetup {
                world,
                asset_id: &asset_id,
                asset_version: version,
                mod_name,
            }),
            SETUP,
            &[],
        )?;

        if !results.is_empty() {
            bail!("Mod setup returned values: {:?}, expected []", results);
        }

        Ok(Self(Inner::Ready {
            version,
            instance_pre,
        }))
    }

    pub(crate) fn run_system<'a, 'w, 's>(
        &self,
        runner: &mut Runner,
        name: &str,
        config: ConfigRunSystem<'a, 'w, 's>,
        params: &[Val],
    ) -> Result<Vec<Val>> {
        let Inner::Ready { instance_pre, .. } = &self.0 else {
            bail!("Mod is not in Ready state");
        };

        call(
            runner,
            instance_pre,
            Config::RunSystem(config),
            name,
            params,
        )
    }
}

fn call(
    runner: &mut Runner,
    instance_pre: &InstancePre<WasmHost>,
    config: Config,
    name: &str,
    params: &[Val],
) -> Result<Vec<Val>> {
    runner.use_store(config, move |mut store| {
        let instance = instance_pre
            .instantiate(&mut store)
            .context("Failed to instantiate component")?;

        let func = instance
            .get_func(&mut store, name)
            .ok_or(anyhow!("Missing {} function", name))?;

        let mut results = vec![];
        func.call(&mut store, params, &mut results)
            .context("Failed to run the desired wasm function")?;

        Ok(results)
    })
}

/// The bevy [`AssetLoader`] for [`ModAsset`]
pub struct ModAssetLoader {
    pub(crate) linker: Linker,
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
