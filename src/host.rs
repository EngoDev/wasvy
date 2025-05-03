use std::alloc::Layout;
use std::borrow::Cow;

use bevy::asset::{AssetId, Handle};
use bevy::ecs::bundle::{Bundle, DynamicBundle};
use bevy::ecs::component::{
    Component as BevyComponent, ComponentDescriptor as BevyComponentDescriptor, ComponentId,
    StorageType,
};
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Commands;
use bevy::ecs::world::World;
use bevy::ptr::OwningPtr;
// use crate::bindings::component::protocol::host_ecs;
use serde::{Deserialize, Serialize};

use crate::asset::WasmComponentAsset;
// use crate::bindings::component::host_ecsprotocol::host_ecs;
use crate::bindings::wasvy::ecs::types;

pub struct WasmHost<'a> {
    pub world: &'a mut World,
    pub wasm_asset_id: AssetId<WasmComponentAsset>,
}

#[derive(BevyComponent, Serialize, Deserialize)]
pub struct WasmComponent {
    // serialized_value: serde_json::Value,
    pub serialized_value: String,
}

#[derive(Clone, BevyComponent)]
pub struct WasmGuestSystem {
    pub name: String,
    pub queries: wasmtime::component::__internal::Vec<types::Query>,
    pub wasm_asset_id: AssetId<WasmComponentAsset>,
}

fn create_component_descriptor(name: impl Into<Cow<'static, str>>) -> BevyComponentDescriptor {
    unsafe {
        BevyComponentDescriptor::new_with_layout(
            name,
            WasmComponent::STORAGE_TYPE,
            Layout::new::<WasmComponent>(),
            None,
            false,
            WasmComponent::clone_behavior(),
        )
    }
}

impl crate::bindings::wasvy::ecs::functions::Host for WasmHost<'_> {
    fn register_component(
        &mut self,
        path: wasmtime::component::__internal::String,
        // descriptor: host_ecs::ComponentDescriptor,
    ) -> types::ComponentId {
        self.world
            .register_component_with_descriptor(create_component_descriptor(Cow::from(path)))
            .index() as u64
    }

    fn register_system(
        &mut self,
        name: wasmtime::component::__internal::String,
        query: wasmtime::component::__internal::Vec<types::Query>,
    ) {
        self.world.spawn(WasmGuestSystem {
            name,
            queries: query,
            wasm_asset_id: self.wasm_asset_id.clone(),
        });
    }

    fn get_component_id(
        &mut self,
        path: wasmtime::component::__internal::String,
    ) -> Option<types::ComponentId> {
        for component_info in self.world.components().iter_registered() {
            if *component_info.name().to_string() == path.to_string() {
                return Some(component_info.id().index() as u64);
            }
        }

        None
    }

    fn spawn(
        &mut self,
        components: wasmtime::component::__internal::Vec<types::Component>,
    ) -> types::Entity {
        // let component_ids: Vec<ComponentId> = components
        //     .iter()
        //     .map(|component_id| ComponentId::new(*component_id as usize))
        //     .collect();

        // let bundle_info = self.world.register_dynamic_bundle(&component_ids);
        // let spawner = BundleSpaw

        let mut commands = self.world.commands();

        let mut entity = commands.spawn_empty();
        // .commands()
        // .get_entity(Entity::PLACEHOLDER)
        // .unwrap();
        for component in components {
            unsafe {
                entity.insert_by_id(
                    ComponentId::new(component.id as usize),
                    WasmComponent {
                        serialized_value: component.value,
                    },
                );
            };
        }

        entity.id().index() as u64
        //insert_by_id(component_id, value)
    }

    fn this_function_does_nothing(
        &mut self,
        entry: crate::bindings::wasvy::ecs::types::QueryResultEntry,
        query_result: crate::bindings::wasvy::ecs::types::QueryResult,
    ) {
    }
}

// unsafe impl Bundle for GuestComponents {
//     fn get_component_ids(components: &bevy::ecs::component::Components, ids: &mut impl FnMut(Option<ComponentId>)) {
//
//     }
//
//     fn register_required_components(
//             _components: &mut bevy::ecs::component::ComponentsRegistrator,
//             _required_components: &mut bevy::ecs::component::RequiredComponents,
//         ) {
//
//     }
//
//     fn component_ids(components: &mut ComponentsRegistrator, ids: &mut impl FnMut(ComponentId)) {
//     }
// }
//
// impl DynamicBundle for GuestComponents {
//
//     fn get_components(self, func: &mut impl FnMut(StorageType, OwningPtr<'_>)) -> Self::Effect {
//     }
// }
