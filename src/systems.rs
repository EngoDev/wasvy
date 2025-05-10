use std::{any::TypeId, collections::HashMap};

use bevy::{
    ecs::{component::ComponentId, system::QueryParamBuilder, world::FilteredEntityRef},
    prelude::*,
    reflect::{ReflectFromPtr, TypeRegistry, serde::TypedReflectSerializer},
};
use wasmtime::Store;

use crate::{
    asset::WasmComponentAsset,
    bindings::wasvy::ecs::types::{self, Component as BindingComponent, QueryResultEntry},
    component::{HostWasmComponentResource, WasmComponents},
    component_registry::WasmComponentRegistry,
    plugin::WasmComponent,
    state::States,
};

#[derive(Clone, Component, Reflect)]
pub struct WasmGuestSystem {
    pub name: String,
    #[reflect(ignore)]
    pub queries: wasmtime::component::__internal::Vec<types::Query>,
    pub wasm_asset_id: AssetId<WasmComponentAsset>,
}

/// This struct contains the Query data that will be sent to the WASM guest system.
pub struct WasmSystemParamBuilder {
    pub system: WasmGuestSystem,
    pub system_param: InternalQueriesResult,
}

struct InternalQueryResultEntry {
    entity: types::Entity,
    components: Vec<types::Component>,
}

// struct InternalQueryResult(pub Vec<InternalQueryResultEntry>);

type InternalQueryResult = Vec<InternalQueryResultEntry>;

type InternalQueriesResult = Vec<InternalQueryResult>;

impl WasmSystemParamBuilder {
    pub fn new(guest_system: WasmGuestSystem, world: &mut World) -> Self {
        Self {
            system_param: Self::create_system_param(guest_system.queries.clone(), world),
            system: guest_system,
        }
    }

    pub fn create_system_param(
        // &self,
        queries: wasmtime::component::__internal::Vec<types::Query>,
        world: &mut World,
        // store: Store<States>,
        // ) -> Vec<wasmtime::component::Val> {
    ) -> InternalQueriesResult {
        let type_registry_guard = world.get_resource::<AppTypeRegistry>().unwrap().clone();
        let type_registry = type_registry_guard.read();
        let registry = world
            .get_resource::<WasmComponentRegistry>()
            .unwrap()
            .clone();
        let world_components = Self::get_world_components(world);

        queries
            .iter()
            .map(|query| {
                let mut query_state =
                    Self::build_query_state(query, world, &registry, &world_components);
                let query_rows = Self::process_query_results(
                    query_state.iter(world),
                    query,
                    &registry,
                    &type_registry,
                    &world_components,
                );

                query_rows
                // wasmtime::component::Val::List(query_rows)
            })
            .collect::<InternalQueriesResult>()
    }

    // pub fn build(self, store: &mut Store<States>) -> Vec<wasmtime::component::Val> {
    pub fn build(self, store: &mut Store<States>) -> Vec<types::QueryResult> {
        let mut blah = vec![];

        for query_result in self.system_param {
            let mut results = vec![];
            for result in query_result {
                let mut components: Vec<wasmtime::component::Resource<types::Component>> = vec![];
                for component in result.components {
                    components.push(
                        store
                            .data_mut()
                            .host_ecs
                            .components
                            .table
                            .push(component)
                            .unwrap(),
                    );
                }

                results.push(types::QueryResultEntry {
                    entity: result.entity,
                    components,
                });
            }

            blah.push(results);
        }

        blah
    }

    fn get_world_components(world: &World) -> HashMap<TypeId, ComponentId> {
        world
            .components()
            .iter_registered()
            .filter_map(|component_info| {
                component_info
                    .type_id()
                    .map(|type_id| (type_id, component_info.id()))
            })
            .collect()
    }

    fn build_query_state<'w>(
        query: &types::Query,
        world: &mut World,
        registry: &WasmComponentRegistry,
        world_components: &HashMap<TypeId, ComponentId>,
    ) -> QueryState<FilteredEntityRef<'w>> {
        let type_registry = world.get_resource::<AppTypeRegistry>().unwrap().clone();
        let type_registry = type_registry.read();

        let mut data = QueryBuilder::<FilteredEntityRef<'w>>::new(world);
        for component_type_path in &query.components {
            if let Some(component_id) = registry.get(component_type_path) {
                data.ref_id(*component_id);
            } else {
                let type_data = type_registry
                    .get_with_type_path(component_type_path)
                    .unwrap();
                let component_id = world_components.get(&type_data.type_id()).unwrap();
                data.ref_id(*component_id);
            }
        }
        data.build()
    }

    fn process_query_results<'w>(
        query_results: impl Iterator<Item = FilteredEntityRef<'w>>,
        query: &types::Query,
        registry: &WasmComponentRegistry,
        type_registry: &TypeRegistry,
        world_components: &HashMap<TypeId, ComponentId>,
        // ) -> Vec<wasmtime::component::Val> {
    ) -> Vec<InternalQueryResultEntry> {
        query_results
            .map(|row| {
                let components = query
                    .components
                    .iter()
                    .map(|component_type_path| {
                        Self::create_binding_component(
                            &row,
                            component_type_path,
                            registry,
                            type_registry,
                            world_components,
                        )
                    })
                    .collect();

                InternalQueryResultEntry {
                    entity: row.id().index() as u64,
                    components,
                }
                // record_from_query_result_entry(QueryResultEntry {
                //     entity: row.id().index() as u64,
                //     components,
                // })
            })
            .collect()
    }

    fn create_binding_component(
        row: &FilteredEntityRef,
        component_type_path: &str,
        registry: &WasmComponentRegistry,
        type_registry: &TypeRegistry,
        world_components: &HashMap<TypeId, ComponentId>,
    ) -> BindingComponent {
        // This is a guest component
        if let Some(component_id) = registry.get(component_type_path) {
            let component = unsafe {
                row.get_by_id(*component_id)
                    .unwrap()
                    .deref::<WasmComponent>()
            };
            BindingComponent::Guest(WasmComponent {
                type_path: component.type_path.clone(),
                // type_data: type_registry
                //     .get_with_type_path(&component.type_path)
                //     .unwrap()
                //     .clone(),
                // type_path: component_type_path.to_string(),
                value: component.value.reflect_clone().unwrap(),
                // value: component
            })
        // This is host component
        } else {
            let type_data = type_registry
                .get_with_type_path(component_type_path)
                .unwrap();
            let component_id = world_components.get(&type_data.type_id()).unwrap();
            let reflect_from_ptr = type_data.data::<ReflectFromPtr>().unwrap();
            let reflected_component =
                unsafe { reflect_from_ptr.as_reflect(row.get_by_id(*component_id).unwrap()) };
            // let serializer = TypedReflectSerializer::new(reflected_component, type_registry);

            BindingComponent::Host(HostWasmComponentResource {
                type_data: type_data.clone(),
                value: reflected_component.reflect_clone().unwrap(),
                // value: serde_json::to_string(&serializer).unwrap(),
            })
        }
    }
}

// fn record_from_query_result_entry(
//     data: QueryResultEntry,
//     store: Store<States>,
// ) -> wasmtime::component::Val {
//     let components: Vec<wasmtime::component::Val> = data
//         .components
//         .into_iter()
//         .map(|component| {
//             // wasmtime::component::Val::Resource(
//             //     wasm_components
//             //         .table
//             //         .push(component)
//             //         .unwrap()
//             //         .try_into_resource_any(store),
//             // )
//             wasmtime::component::Val::Resource(component.try_into_resource_any(store).unwrap())
//         })
//         .collect();
//
//     wasmtime::component::Val::Record(vec![
//         (
//             "components".to_string(),
//             wasmtime::component::Val::List(components),
//         ),
//         (
//             "entity".to_string(),
//             wasmtime::component::Val::U64(data.entity),
//         ),
//     ])
// }
