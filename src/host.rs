use std::alloc::Layout;
use std::any::TypeId;
use std::borrow::Cow;

use bevy::asset::AssetId;
use bevy::ecs::component::{
    Component as BevyComponent, ComponentDescriptor as BevyComponentDescriptor, ComponentId,
};
use bevy::ecs::name::Name;
use bevy::ecs::reflect::{AppTypeRegistry, ReflectCommandExt};
use bevy::ecs::world::World;
use bevy::reflect::prelude::*;
use bevy::reflect::serde::{ReflectDeserializer, TypedReflectDeserializer};
use bevy::reflect::{PartialReflect, Type, TypeRegistration, TypeRegistry};
use serde::de::DeserializeSeed;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer as JsonDeserializer;

use crate::asset::WasmComponentAsset;
use crate::bindings::wasvy::ecs::types;
use crate::component_registry::WasmComponentRegistry;

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

pub fn type_id_for_path(registry: &TypeRegistry, path: &str) -> Option<TypeId> {
    // Try to find a registration by the full, stable type path
    registry
        .get_with_type_path(path)
        .map(|registration: &TypeRegistration| registration.type_id())
}

impl crate::bindings::wasvy::ecs::functions::Host for WasmHost<'_> {
    fn register_component(
        &mut self,
        path: wasmtime::component::__internal::String,
    ) -> types::ComponentId {
        let type_registry = self
            .world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .clone();

        // This is a known type by the host
        if let Some(id) = type_id_for_path(&type_registry.read(), &path) {
            return self.world.components().get_id(id).expect("if the value exists in the type registry it should also exist in the world components.").index() as u64;
        }

        let id = self
            .world
            .register_component_with_descriptor(create_component_descriptor(Cow::from(
                path.clone(),
            )));

        self.world
            .get_resource_mut::<WasmComponentRegistry>()
            .unwrap()
            .insert(path, id);

        id.index() as u64
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
        let type_registry_guard = self
            .world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .clone();

        let type_registry = type_registry_guard.read();

        let registry = self
            .world
            .get_resource::<WasmComponentRegistry>()
            .unwrap()
            .clone();

        let mut commands = self.world.commands();

        // type_registry.write().add_registration

        // type_registry.0.read().into

        let mut entity = commands.spawn_empty();
        for component in components {
            // The component is a WASM component
            if let Some(component_id) = registry.get(&component.path) {
                unsafe {
                    entity.insert_by_id(
                        ComponentId::new(component_id.index() as usize),
                        WasmComponent {
                            serialized_value: component.value,
                        },
                    );
                };
            // The component exists in the host.
            } else {
                let type_registration = type_registry.get_with_type_path(&component.path).unwrap();
                println!("Deserializing component: {:?}", component.value);
                let mut de = JsonDeserializer::from_str(&component.value);
                let reflect_deserializer =
                    TypedReflectDeserializer::new(type_registration, &type_registry);
                let output: Box<dyn PartialReflect> =
                    reflect_deserializer.deserialize(&mut de).unwrap();

                let type_id = output.get_represented_type_info().unwrap().type_id();
                let reflect_from_reflect = type_registry
                    .get_type_data::<ReflectFromReflect>(type_id)
                    .unwrap();
                let value: Box<dyn Reflect> = reflect_from_reflect
                    .from_reflect(output.as_partial_reflect())
                    .unwrap();
                entity.insert_reflect(value);
                // let boxed_reflect: Bpx<dyn Reflect> = ReflectDeserializer::new(&registry);
                // let blah = type_registration.try_into
            }
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
