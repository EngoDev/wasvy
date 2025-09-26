use bevy::{
    asset::AssetId,
    ecs::{component::Tick, schedule::Schedules},
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::{asset::ModAsset, engine::Engine, send_sync_ptr::SendSyncPtr};

pub(crate) type Store = wasmtime::Store<WasmHost>;

/// Used to contruct a [`Store`] in order to run mods
pub(crate) struct Runner {
    host: Option<WasmHost>,
}

impl Runner {
    pub(crate) fn new() -> Self {
        let table = ResourceTable::new();
        let ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_network()
            .allow_ip_name_lookup(true)
            .build();

        let host = WasmHost {
            inner: Inner::Uninitialized,
            table,
            ctx,
        };

        Self { host: Some(host) }
    }

    pub(crate) fn use_store<'a, F, R>(&mut self, engine: &Engine, config: Config<'a>, mut f: F) -> R
    where
        F: FnMut(&mut Store) -> R,
    {
        let Some(mut host) = self.host.take() else {
            panic!("Cannot re-borrow host for use in another store");
        };

        host.inner = match config {
            Config::Setup(ConfigSetup {
                schedules,
                asset_id,
                asset_version,
                mod_name,
            }) => Inner::Setup {
                // Erase lifetime of schedules
                schedules: SendSyncPtr::new(schedules.into()),
                app_init: false,
                asset_id: *asset_id,
                asset_version,
                mod_name: mod_name.to_string(),
            },
            Config::RunSystem => Inner::RunSystem,
        };

        let engine = engine.inner();
        let mut store = Store::new(&engine, host);

        let ret = f(&mut store);

        let mut host = store.into_data();

        // Avoid leaking refs stored in inner scoped to 'a
        host.inner = Inner::Uninitialized;
        self.host = Some(host);

        ret
    }
}

pub(crate) struct WasmHost {
    inner: Inner,
    table: ResourceTable,
    ctx: WasiCtx,
}

enum Inner {
    Uninitialized,
    Setup {
        schedules: SendSyncPtr<Schedules>,
        app_init: bool,
        mod_name: String,
        asset_id: AssetId<ModAsset>,
        asset_version: Tick,
    },
    RunSystem,
}

impl WasmHost {
    pub(crate) fn access(&mut self) -> State<'_> {
        let table = &mut self.table;
        match &mut self.inner {
            Inner::Setup {
                schedules,
                app_init,
                asset_id,
                asset_version,
                mod_name,
            } => State::Setup {
                // Safety: Always contains a reference to an initialized value, and borrow_mut ensures this is the only borrow
                schedules: unsafe { schedules.as_mut() },
                app_init,
                asset_id,
                asset_version,
                mod_name,
                table,
            },
            Inner::RunSystem => State::RunSystem,
            Inner::Uninitialized => panic!("Attempting to get state from unscoped WasmHost"),
        }
    }
}

pub(crate) enum State<'s> {
    Setup {
        schedules: &'s mut Schedules,
        table: &'s mut ResourceTable,
        app_init: &'s mut bool,
        mod_name: &'s str,
        asset_id: &'s AssetId<ModAsset>,
        asset_version: &'s Tick,
    },
    RunSystem,
}

pub(crate) enum Config<'s> {
    Setup(ConfigSetup<'s>),
    RunSystem,
}

pub(crate) struct ConfigSetup<'s> {
    pub(crate) schedules: &'s mut Schedules,
    pub(crate) asset_id: &'s AssetId<ModAsset>,
    pub(crate) asset_version: Tick,
    pub(crate) mod_name: &'s str,
}

impl WasiView for WasmHost {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}
