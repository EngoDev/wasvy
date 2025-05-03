use std::alloc::Layout;
use std::borrow::Cow;

use bevy::asset::AssetId;
use bevy::ecs::component::{
    Component as BevyComponent, ComponentDescriptor as BevyComponentDescriptor, ComponentId,
};
use bevy::ecs::name::Name;
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};

use crate::asset::WasmComponentAsset;
use crate::bindings::wasvy::ecs::types;

/// The implemenation of the ECS host functions that the WASM components use for interacting with
/// Bevy.
pub struct WasmHost<'a> {
    pub world: &'a mut World,
    /// The WASM component this host is going to be used on.
    pub wasm_asset_id: AssetId<WasmComponentAsset>,
}

/// This component is the wrapper component for all the Bevy components that are registered in a
/// WASM.
///
/// # Description
///
/// When you call the spawn method in WASM you need to provide a component id, that id is used to
/// add a new [`WasmComponent`] under that id with the `serialized_value` that is given.
///
/// This approach makes it possible to register components that don't exist in Rust.
#[derive(BevyComponent, Serialize, Deserialize)]
pub struct WasmComponent {
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
        self.world.spawn((
            Name::new("WasvySystem"),
            WasmGuestSystem {
                name,
                queries: query,
                wasm_asset_id: self.wasm_asset_id,
            },
        ));
    }

    fn get_component_id(
        &mut self,
        path: wasmtime::component::__internal::String,
    ) -> Option<types::ComponentId> {
        for component_info in self.world.components().iter_registered() {
            if *component_info.name() == path {
                return Some(component_info.id().index() as u64);
            }
        }

        None
    }

    fn spawn(
        &mut self,
        components: wasmtime::component::__internal::Vec<types::Component>,
    ) -> types::Entity {
        let mut commands = self.world.commands();

        let mut entity = commands.spawn_empty();
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
    }

    fn this_function_does_nothing(
        &mut self,
        _entry: crate::bindings::wasvy::ecs::types::QueryResultEntry,
        _query_result: crate::bindings::wasvy::ecs::types::QueryResult,
    ) {
    }
}
