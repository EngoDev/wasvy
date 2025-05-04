use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use bevy::{
    ecs::{
        component::{ComponentId, Components},
        system::SystemState,
        world::FilteredEntityRef,
    },
    prelude::*,
    reflect::{
        ReflectFromPtr, Type,
        serde::{ReflectSerializer, TypedReflectSerializer},
    },
};
use wasmtime::{Engine, Store};
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

use crate::{
    asset::{WasmComponentAsset, WasmComponentAssetLoader},
    bindings::{
        self,
        wasvy::ecs::types::{self, Component as BindingComponent, QueryResultEntry},
    },
    component_registry::WasmComponentRegistry,
    host::{WasmComponent, WasmGuestSystem, WasmHost},
    runner::{Runner, WasmRunState},
};

struct WasmSystemWithParams {
    pub system: WasmGuestSystem,
    pub system_param: Vec<wasmtime::component::Val>,
}

impl WasmSystemWithParams {
    pub fn new(guest_system: WasmGuestSystem, world: &mut World) -> Self {
        Self {
            system_param: Self::create_system_param(guest_system.queries.clone(), world),
            system: guest_system,
        }
    }

    fn create_system_param(
        queries: wasmtime::component::__internal::Vec<types::Query>,
        world: &mut World,
    ) -> Vec<wasmtime::component::Val> {
        let mut system_param: Vec<wasmtime::component::Val> = vec![];

        let type_registry_guard = world.get_resource::<AppTypeRegistry>().unwrap().clone();

        let type_registry = type_registry_guard.read();

        let registry = world
            .get_resource::<WasmComponentRegistry>()
            .unwrap()
            .clone();

        let world_components: HashMap<TypeId, ComponentId> = world
            .components()
            .iter_registered()
            .filter_map(|component_info| {
                if component_info.type_id().is_some() {
                    Some((component_info.type_id().unwrap(), component_info.id()))
                } else {
                    None
                }
            })
            .collect();

        for query in &queries {
            let mut data = QueryBuilder::<FilteredEntityRef>::new(world);
            for component_type_path in &query.components {
                // The component was made in the WASM module
                if let Some(component_id) = registry.get(component_type_path) {
                    data.ref_id(*component_id);
                // The component exists in the host
                } else {
                    let type_id = type_registry
                        .get_with_type_path(component_type_path)
                        .unwrap()
                        .type_id();
                    data.ref_id(*world_components.get(&type_id).unwrap());
                }
            }

            let mut query_state = data.build();
            let data = query_state.iter(world);

            let mut query_rows = vec![];
            for row in data.into_iter() {
                let mut components: Vec<BindingComponent> = vec![];
                for component_type_path in &query.components {
                    // The component was made in the WASM module
                    if let Some(component_id) = registry.get(component_type_path) {
                        let component = unsafe {
                            row.get_by_id(*component_id)
                                .unwrap()
                                .deref::<WasmComponent>()
                        };
                        components.push(BindingComponent {
                            path: component_type_path.to_string(),
                            value: component.serialized_value.clone(),
                        });

                    // The component exists in the host
                    } else {
                        let type_data = type_registry
                            .get_with_type_path(component_type_path)
                            .unwrap();
                        let component_id = world.components().get_id(type_data.type_id()).unwrap();
                        let reflect_from_ptr = type_data.data::<ReflectFromPtr>().unwrap();
                        let value = unsafe {
                            reflect_from_ptr.as_reflect(row.get_by_id(component_id).unwrap())
                        };
                        let serializer = TypedReflectSerializer::new(value, &type_registry);
                        components.push(BindingComponent {
                            path: component_type_path.to_string(),
                            value: serde_json::to_string(&serializer).unwrap(),
                        });
                    }
                }
                query_rows.push(record_from_query_result_entry(QueryResultEntry {
                    entity: row.id().index() as u64,
                    components,
                }));
            }

            system_param.push(wasmtime::component::Val::List(query_rows));
        }

        system_param
    }
}

/// The state object that houses the functionality that is passed to WASM components.
pub struct States<'a> {
    table: ResourceTable,
    ctx: WasiCtx,
    host_ecs: WasmHost<'a>,
}

impl<'a> States<'a> {
    pub fn new(host_ecs: WasmHost<'a>) -> Self {
        let table = ResourceTable::new();
        let ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_network()
            .allow_ip_name_lookup(true)
            .build();
        Self {
            table,
            ctx,
            host_ecs,
        }
    }
}

impl IoView for States<'_> {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasiView for States<'_> {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

pub struct WasvyHostPlugin;

/// Cross engine instatiation of WASM components is not supported.
/// This resources is the global [`Engine`] that is used for instatiation.
///
/// Check the [`Engine`] docs for more information.
#[derive(Resource, Clone, Deref)]
pub struct WasmEngine(Engine);

impl Plugin for WasvyHostPlugin {
    fn build(&self, app: &mut App) {
        let a = Type::of::<Transform>();
        println!("Transform blah: {:?}", a.path());
        app.add_systems(Update, (run_setup, run_systems));

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

    let mut runner = Runner::new(engine.0);
    runner.add_wasi_sync();

    runner.add_functionality(|linker| {
        bindings::wasvy::ecs::functions::add_to_linker(linker, |state: &mut States| {
            &mut state.host_ecs
        })
        .unwrap();
    });

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
        let store = Store::new(&runner.engine, wasi_view);
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

fn record_from_query_result_entry(data: QueryResultEntry) -> wasmtime::component::Val {
    let components: Vec<wasmtime::component::Val> = data
        .components
        .into_iter()
        .map(|component: BindingComponent| {
            wasmtime::component::Val::Record(vec![
                (
                    "path".to_string(),
                    wasmtime::component::Val::String(component.path),
                ),
                (
                    "value".to_string(),
                    wasmtime::component::Val::String(component.value),
                ),
            ])
        })
        .collect();

    wasmtime::component::Val::Record(vec![
        (
            "components".to_string(),
            wasmtime::component::Val::List(components),
        ),
        (
            "entity".to_string(),
            wasmtime::component::Val::U64(data.entity),
        ),
    ])
}

fn run_setup(world: &mut World, mut already_ran: Local<HashSet<AssetId<WasmComponentAsset>>>) {
    #[allow(clippy::type_complexity)]
    let mut system_state: SystemState<(
        EventReader<AssetEvent<WasmComponentAsset>>,
        ResMut<Assets<WasmComponentAsset>>,
    )> = SystemState::new(world);

    let assets_to_setup = {
        let (mut asset_events, assets) = system_state.get_mut(world);

        let mut assets_to_setup: Vec<(AssetId<WasmComponentAsset>, WasmComponentAsset)> = vec![];
        for ev in asset_events.read() {
            match ev {
                AssetEvent::LoadedWithDependencies { id } => {
                    let wasm_asset = assets.get(*id).unwrap();
                    if !already_ran.contains(id) {
                        assets_to_setup.push((*id, wasm_asset.clone()));
                        already_ran.insert(*id);
                    }
                }
                AssetEvent::Modified { id } => {
                    let wasm_asset = assets.get(*id).unwrap();
                    assets_to_setup.push((*id, wasm_asset.clone()));
                }
                _ => {}
            }
        }

        assets_to_setup
    };

    if assets_to_setup.is_empty() {
        return;
    }

    let engine = world.get_resource::<WasmEngine>().unwrap().clone();

    let mut runner = Runner::new(engine.0);
    runner.add_wasi_sync();

    runner.add_functionality(|linker| {
        bindings::wasvy::ecs::functions::add_to_linker(linker, |state: &mut States| {
            &mut state.host_ecs
        })
        .unwrap();
    });

    let params = [];
    let mut results = vec![];
    for (id, asset) in assets_to_setup {
        let wasm_host = WasmHost {
            world,
            wasm_asset_id: id,
        };
        let wasi_view = States::new(wasm_host);
        let store = Store::new(&runner.engine, wasi_view);

        runner.run_function(WasmRunState {
            component: &asset.component,
            function_name: "setup".to_string(),
            store,
            params: &params,
            results: &mut results,
        });
    }
}
