use std::collections::{HashMap, HashSet};

use bevy::{ecs::system::SystemState, prelude::*};
use serde::{Deserialize, Serialize};
use wasmtime::{Engine, Store};

use crate::{
    asset::{WasmComponentAsset, WasmComponentAssetLoader},
    bindings,
    component::WasmComponents,
    component_registry::WasmComponentRegistry,
    host::WasmHost,
    runner::{Runner, WasmRunState},
    state::States,
    systems::{WasmGuestSystem, WasmSystemParamBuilder},
};

pub struct WasvyHostPlugin;

/// Cross engine instatiation of WASM components is not supported.
/// This resources is the global [`Engine`] that is used for instatiation.
///
/// Check the [`Engine`] docs for more information.
#[derive(Resource, Clone, Deref)]
pub struct WasmEngine(Engine);

/// This component is the wrapper component for all the Bevy components that are registered in a
/// WASM.
///
/// # Description
///
/// When you call the spawn method in WASM you need to provide a component id, that id is used to
/// add a new [`WasmComponent`] under that id with the `serialized_value` that is given.
///
/// This approach makes it possible to register components that don't exist in Rust.
#[derive(Component)]
pub struct WasmComponent {
    // pub serialized_value: String,
    pub type_path: String,
    pub value: Box<dyn Reflect>,
}

impl Plugin for WasvyHostPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (run_setup, run_systems));
        app.register_type::<WasmGuestSystem>();
        // app.register_type::<WasmComponent>();

        let engine = Engine::default();

        app.init_asset::<WasmComponentAsset>()
            .register_asset_loader(WasmComponentAssetLoader {
                engine: engine.clone(),
            });

        app.insert_resource(WasmEngine(engine))
            .init_resource::<WasmComponentRegistry>();
    }
}

fn run_systems(world: &mut World) {
    let wasm_systems: Vec<WasmGuestSystem> = world
        .query::<&WasmGuestSystem>()
        .iter(world)
        .cloned()
        .collect();

    let assests: HashMap<AssetId<WasmComponentAsset>, WasmComponentAsset> = world
        .get_resource::<Assets<WasmComponentAsset>>()
        .unwrap()
        .iter()
        .map(|(id, asset)| (id, asset.clone()))
        .collect();

    let engine = world.get_resource::<WasmEngine>().unwrap().clone();

    let runner = create_runner(engine.0);

    let systems: Vec<WasmSystemParamBuilder> = wasm_systems
        .into_iter()
        .map(|system| WasmSystemParamBuilder::new(system, world))
        .collect();

    for wasm_system in systems.into_iter() {
        let wasm_host = WasmHost::new(world, wasm_system.system.wasm_asset_id);
        let wasi_view = States::new(wasm_host);
        let mut store = Store::new(&runner.engine, wasi_view);
        let module = assests.get(&wasm_system.system.wasm_asset_id).unwrap();
        // let mut system_param =
        //     WasmSystemWithParams::new(wasm_system.clone()).create_system_param(world, store);

        let mut results = ();
        let system_name = wasm_system.system.name.clone();
        let params = wasm_system.build(&mut store);
        runner.run_function(WasmRunState {
            function_name: system_name,
            component: &module.component,
            params: (params, 0_u64),
            // params: &[wasmtime::component::Val::List(
            //     wasmtime::component::Val::from(wasm_system.build(&mut store)),
            // )],
            store,
            results: &mut results,
        });
    }
}

fn run_setup(world: &mut World, mut already_ran: Local<HashSet<AssetId<WasmComponentAsset>>>) {
    let assets_to_setup = get_assets_to_setup(world, &mut already_ran);
    if assets_to_setup.is_empty() {
        return;
    }

    let engine = world.get_resource::<WasmEngine>().unwrap().clone();
    let runner = create_runner(engine.0);

    for (id, asset) in assets_to_setup {
        let wasm_host = WasmHost::new(world, id);
        let wasi_view = States::new(wasm_host);
        let store = Store::new(&runner.engine, wasi_view);

        let mut results = ();
        runner.run_function(WasmRunState {
            component: &asset.component,
            function_name: "setup".to_string(),
            store,
            params: (),
            results: &mut results,
        });
    }
}

fn get_assets_to_setup(
    world: &mut World,
    already_ran: &mut Local<HashSet<AssetId<WasmComponentAsset>>>,
) -> Vec<(AssetId<WasmComponentAsset>, WasmComponentAsset)> {
    #[allow(clippy::type_complexity)]
    let mut system_state: SystemState<(
        EventReader<AssetEvent<WasmComponentAsset>>,
        ResMut<Assets<WasmComponentAsset>>,
    )> = SystemState::new(world);

    let (mut asset_events, assets) = system_state.get_mut(world);
    let mut assets_to_setup = Vec::new();

    for ev in asset_events.read() {
        match ev {
            AssetEvent::LoadedWithDependencies { id } => {
                if !already_ran.contains(id) {
                    if let Some(wasm_asset) = assets.get(*id) {
                        assets_to_setup.push((*id, wasm_asset.clone()));
                        already_ran.insert(*id);
                    }
                }
            }
            AssetEvent::Modified { id } => {
                if let Some(wasm_asset) = assets.get(*id) {
                    assets_to_setup.push((*id, wasm_asset.clone()));
                }
            }
            _ => {}
        }
    }

    assets_to_setup
}

fn create_runner<'a>(engine: Engine) -> Runner<States<'a>> {
    let mut runner = Runner::new(engine);
    runner.add_wasi_sync();
    runner.add_functionality(|linker| {
        bindings::wasvy::ecs::functions::add_to_linker(linker, |state: &mut States| {
            &mut state.host_ecs
        })
        .unwrap();
    });

    runner.add_functionality(|linker| {
        bindings::wasvy::ecs::types::add_to_linker(linker, |state: &mut States| {
            &mut state.host_ecs.components
        })
        .unwrap();
    });
    runner
}
