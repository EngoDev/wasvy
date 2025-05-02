use bevy::{
    ecs::{component::ComponentId, world::FilteredEntityRef},
    prelude::*,
};
use wasmtime::{
    Engine, Store,
    component::{Component, Func, Linker},
};
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

use crate::{
    bindings::{
        self,
        component::protocol::types::{Component as BindingComponent, QueryData},
    },
    host::{WasmComponent, WasmGuestSystem, WasmHost},
};

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

impl<'a> IoView for States<'a> {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl<'a> WasiView for States<'a> {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

pub struct HostPlugin;

impl Plugin for HostPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, run_setup);
        app.add_systems(Update, run_systems);
    }
}

fn run_systems(world: &mut World) {
    // / let mut entities = world.query::<(Entity, &Order, &Label)>()
    let wasm_systems: Vec<WasmGuestSystem> = world
        .query::<&WasmGuestSystem>()
        .iter(world)
        .cloned()
        .collect();

    for wasm_system in wasm_systems.iter() {
        // let mut args: Vec<wasmtime::component::Val> = vec![];
        let mut system_param: Vec<wasmtime::component::Val> = vec![];

        for query in &wasm_system.queries {
            let mut data = QueryBuilder::<FilteredEntityRef>::new(world);
            // let mut access = FilteredAccess::default();
            for component_index in &query.components {
                data.ref_id(ComponentId::new(*component_index as usize));
                // println!(
                //     "Adding component: {:?}, {:?}",
                //     component_index,
                //     world
                //         .components()
                //         .get_name(ComponentId::new(*component_index as usize))
                //         .unwrap()
                //         .clone()
                // );
            }

            let mut query_state = data.build();
            let data = query_state.iter(world);

            let mut query_rows = vec![];
            for row in data.into_iter() {
                // println!("Row: {:?}", row.id());
                let mut components: Vec<BindingComponent> = vec![];
                for component_index in &query.components {
                    let component_a = unsafe {
                        row.get_by_id(ComponentId::new(*component_index as usize))
                            .unwrap()
                            .deref::<WasmComponent>()
                    };
                    components.push(BindingComponent {
                        id: *component_index,
                        value: component_a.serialized_value.clone(),
                    });
                    // let a = serde_json::to_string(&component_a);
                }
                query_rows.push(record_from_query_data(QueryData {
                    entity: row.id().index() as u64,
                    components,
                }));
            }

            system_param.push(wasmtime::component::Val::List(query_rows));
        }

        let wasm_host = WasmHost { world };
        let engine = Engine::default();
        let module = Component::from_file(&engine, "guest3.wasm").unwrap();

        let wasi_view = States::new(wasm_host);
        let mut store = Store::new(&engine, wasi_view);
        let mut linker = Linker::new(&engine);

        wasmtime_wasi::add_to_linker_sync(&mut linker).expect("Could not add wasi to linker");
        bindings::component::protocol::host_ecs::add_to_linker(
            &mut linker,
            |state: &mut States| &mut state.host_ecs,
        )
        .unwrap();

        let instance = linker.instantiate(&mut store, &module).unwrap();
        let func: Func = instance
            .get_func(&mut store, wasm_system.name.to_string())
            .expect("`WasmGuestSystem system name` not found");

        let mut results = vec![];
        // println!("Args: {:?}", args);
        // if args.len() == 0 {
        //     continue;
        // }
        func.call(
            store,
            &[wasmtime::component::Val::List(system_param)],
            &mut results,
        )
        .unwrap();
    }
}

fn record_from_query_data(data: QueryData) -> wasmtime::component::Val {
    let components: Vec<wasmtime::component::Val> = data
        .components
        .into_iter()
        .map(|component: BindingComponent| {
            wasmtime::component::Val::Record(vec![
                (
                    "id".to_string(),
                    wasmtime::component::Val::U64(component.id),
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

fn run_setup(world: &mut World) {
    let wasm_host = WasmHost { world };
    let engine = Engine::default();
    let module = Component::from_file(&engine, "guest3.wasm").unwrap();

    let wasi_view = States::new(wasm_host);
    let mut store = Store::new(&engine, wasi_view);
    let mut linker = Linker::new(&engine);

    wasmtime_wasi::add_to_linker_sync(&mut linker).expect("Could not add wasi to linker");
    bindings::component::protocol::host_ecs::add_to_linker(&mut linker, |state: &mut States| {
        &mut state.host_ecs
    })
    .unwrap();

    let instance = linker.instantiate(&mut store, &module).unwrap();

    let func: Func = instance
        .get_func(&mut store, "setup")
        .expect("`setup` not found");

    let args = [];
    // let mut results = vec![Val::String("".to_string())];
    let mut results = vec![];
    func.call(store, &args, &mut results).unwrap();

    // println!("Params: {:?}, Results: {:?}", ty.params(), ty.results());
    // println!("Results: {:?}", results);
}
