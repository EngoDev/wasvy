use std::collections::{HashMap, HashSet};

use bevy::{ecs::system::SystemState, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    asset::{WasmComponentAsset, WasmComponentAssetLoader},
    bindings,
    component_registry::WasmComponentRegistry,
    host::WasmHost,
    runner::{Runner, WasmRunState},
    state::States,
    systems::{WasmGuestSystem, WasmSystemWithParams},
};

/// This plugin adds Wasvy modding support to [`App`]
///
/// ```rust
///  App::new()
///    .add_plugins(DefaultPlugins)
///    .add_plugins(ModloaderPlugin)
///    // etc
/// ```
///
/// Looking for next steps? See: [`Mods`](crate::mods::Mods)
/// ```
pub struct ModloaderPlugin;

/// Cross engine instatiation of WASM components is not supported.
/// This resources is the global [`Engine`] that is used for instatiation.
///
/// Check the [`Engine`] docs for more information.
#[derive(Resource, Clone, Deref)]
pub struct Engine(wasmtime::Engine);

/// This component is the wrapper component for all the Bevy components that are registered in a
/// WASM.
///
/// # Description
///
/// When you call the spawn method in WASM you need to provide a component id, that id is used to
/// add a new [`WasmComponent`] under that id with the `serialized_value` that is given.
///
/// This approach makes it possible to register components that don't exist in Rust.
#[derive(Component, Serialize, Deserialize, Reflect)]
pub struct WasmComponent {
    pub serialized_value: String,
}

impl Plugin for ModloaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (run_setup, run_systems));
        app.register_type::<WasmGuestSystem>();
        app.register_type::<WasmComponent>();

        let engine = wasmtime::Engine::default();

        app.init_asset::<WasmComponentAsset>()
            .register_asset_loader(WasmComponentAssetLoader {
                engine: engine.clone(),
            });

        app.insert_resource(Engine(engine))
            .init_resource::<WasmComponentRegistry>();

        let asset_plugins = app.get_added_plugins::<AssetPlugin>();
        let asset_plugin = asset_plugins
            .get(0)
            .expect("ModloaderPlugin requires AssetPlugin to be loaded.");

        // Warn a user running the App in debug; they probably want hot-reloading
        if cfg!(debug_assertions) {
            let user_overrode_watch_setting = asset_plugin.watch_for_changes_override.is_some();
            let resolved_watch_setting = app
                .world()
                .get_resource::<AssetServer>()
                .unwrap()
                .watching_for_changes();

            if !user_overrode_watch_setting && !resolved_watch_setting {
                warn!(
                    "Enable Bevy's watch feature to enable hot-reloading Wasvy mods.\
                You can do this by running the command `cargo run --features bevy/file_watcher`.\
                In order to hide this message, set the `watch_for_changes_override` to\
                `Some(true)` or `Some(false)` in the AssetPlugin."
                );
            }
        }
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

    let engine = world.get_resource::<Engine>().unwrap().clone();

    let runner = create_runner(engine.0);

    let systems: Vec<WasmSystemWithParams> = wasm_systems
        .into_iter()
        .map(|system| WasmSystemWithParams::new(system, world))
        .collect();

    for wasm_system in systems.iter() {
        let wasm_host = WasmHost {
            world,
            wasm_asset_id: wasm_system.system.wasm_asset_id,
        };
        let wasi_view = States::new(wasm_host);
        let store = wasmtime::Store::new(&runner.engine, wasi_view);
        let module = assests.get(&wasm_system.system.wasm_asset_id).unwrap();

        let mut results = vec![];
        runner.run_function(WasmRunState {
            component: &module.component,
            store,
            function_name: wasm_system.system.name.clone(),
            params: &[wasmtime::component::Val::List(
                wasm_system.system_param.clone(),
            )],
            results: &mut results,
        });
    }
}

fn run_setup(world: &mut World, mut already_ran: Local<HashSet<AssetId<WasmComponentAsset>>>) {
    let assets_to_setup = get_assets_to_setup(world, &mut already_ran);
    if assets_to_setup.is_empty() {
        return;
    }

    let engine = world.get_resource::<Engine>().unwrap().clone();
    let runner = create_runner(engine.0);

    for (id, asset) in assets_to_setup {
        let wasm_host = WasmHost {
            world,
            wasm_asset_id: id,
        };
        let wasi_view = States::new(wasm_host);
        let store = wasmtime::Store::new(&runner.engine, wasi_view);

        let mut results = vec![];
        runner.run_function(WasmRunState {
            component: &asset.component,
            function_name: "setup".to_string(),
            store,
            params: &[],
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

fn create_runner<'a>(engine: wasmtime::Engine) -> Runner<States<'a>> {
    let mut runner = Runner::new(engine);
    runner.add_wasi_sync();
    runner.add_functionality(|linker| {
        bindings::wasvy::ecs::functions::add_to_linker(linker, |state: &mut States| {
            &mut state.host_ecs
        })
        .unwrap();
    });
    runner
}
