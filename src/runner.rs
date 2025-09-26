use bevy::{
    asset::AssetId,
    ecs::{component::Tick, schedule::Schedules},
};
use wasmtime_wasi::ResourceTable;

use crate::{asset::ModAsset, engine::Engine, host::WasmHost, send_sync_ptr::SendSyncPtr};

pub(crate) type Store = wasmtime::Store<WasmHost>;

/// Used to contruct a [`Store`] in order to run mods
pub(crate) struct Runner {
    host: Option<WasmHost>,
}

impl Runner {
    pub(crate) fn new() -> Self {
        Self {
            host: Some(WasmHost::new()),
        }
    }

    pub(crate) fn use_store<'a, F, R>(&mut self, engine: &Engine, config: Config<'a>, mut f: F) -> R
    where
        F: FnMut(&mut Store) -> R,
    {
        let Some(mut host) = self.host.take() else {
            panic!("Cannot re-borrow host for use in another store");
        };

        host.set_data(match config {
            Config::Setup(ConfigSetup {
                schedules,
                asset_id,
                asset_version,
                mod_name,
            }) => Data::Setup {
                // Erase lifetime of schedules
                schedules: SendSyncPtr::new(schedules.into()),
                app_init: false,
                asset_id: *asset_id,
                asset_version,
                mod_name: mod_name.to_string(),
            },
            Config::RunSystem => Data::RunSystem,
        });

        let engine = engine.inner();
        let mut store = Store::new(&engine, host);

        let ret = f(&mut store);

        let mut host = store.into_data();

        // Avoid leaking refs stored in inner scoped to 'a
        host.set_data(Data::uninitialized());
        self.host = Some(host);

        ret
    }
}

/// Data stored in [`WasmHost`]
pub(crate) enum Data {
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

impl Data {
    pub(crate) fn uninitialized() -> Self {
        Self::Uninitialized
    }

    /// A helper so [`WasmHost`] can expose access to the [`Data`] it stores
    ///
    /// The resource table from the host is passed through this for convenience
    pub(crate) fn access<'a>(&'a mut self, table: &'a mut ResourceTable) -> Option<State<'a>> {
        match self {
            Data::Setup {
                schedules,
                app_init,
                asset_id,
                asset_version,
                mod_name,
            } => Some(State::Setup {
                // Safety: Always contains a reference to an initialized value, and borrow_mut ensures this is the only borrow
                schedules: unsafe { schedules.as_mut() },
                app_init,
                asset_id,
                asset_version,
                mod_name,
                table,
            }),
            Data::RunSystem => Some(State::RunSystem),
            Data::Uninitialized => None,
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
