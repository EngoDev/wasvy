use bevy::{
    asset::AssetId,
    ecs::{
        component::Tick,
        schedule::Schedules,
        world::{FromWorld, World},
    },
};
use wasmtime_wasi::ResourceTable;

use crate::{asset::ModAsset, engine::Engine, host::WasmHost, send_sync_ptr::SendSyncPtr};

pub(crate) type Store = wasmtime::Store<WasmHost>;

/// Used to contruct a [`Store`] in order to run mods
pub(crate) struct Runner {
    store: Store,
}

impl Runner {
    pub(crate) fn new(engine: &Engine) -> Self {
        let host = WasmHost::new();
        let store = Store::new(&engine, host);

        Self { store }
    }

    pub(crate) fn use_store<'a, F, R>(&mut self, config: Config<'a>, mut f: F) -> R
    where
        F: FnMut(&mut Store) -> R,
    {
        self.store.data_mut().set_data(match config {
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

        let ret = f(&mut self.store);

        // Avoid leaking refs stored in inner scoped to 'a
        self.store.data_mut().set_data(Data::uninitialized());

        ret
    }
}

impl FromWorld for Runner {
    fn from_world(world: &mut World) -> Self {
        let engine = world.get_resource::<Engine>().unwrap();
        Runner::new(engine)
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
                // Safety: Runner::use_store ensures that this always contains a valid reference
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
