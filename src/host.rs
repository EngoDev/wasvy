use std::alloc::Layout;
use std::any::TypeId;
use std::borrow::Cow;

use bevy::{
    asset::AssetId,
    ecs::{
        component::{Component, ComponentDescriptor as BevyComponentDescriptor, ComponentId},
        name::Name,
        reflect::{AppTypeRegistry, ReflectCommandExt},
        system::EntityCommands,
        world::World,
    },
    reflect::{
        PartialReflect, TypeRegistration, TypeRegistry, prelude::*, serde::TypedReflectDeserializer,
    },
};
use serde::de::DeserializeSeed;
use serde_json::Deserializer as JsonDeserializer;

use crate::component_registry::WasmComponentRegistry;
use crate::{asset::WasmComponentAsset, plugin::WasmComponent, systems::WasmGuestSystem};
use crate::{bindings::wasvy::ecs::types, component::WasmComponents};

/// The implemenation of the ECS host functions that the WASM components use for interacting with
/// Bevy.
pub struct WasmHost<'a> {
    pub world: &'a mut World,
    /// The WASM component this host is going to be used on.
    pub wasm_asset_id: AssetId<WasmComponentAsset>,
    pub components: WasmComponents,
}

impl crate::bindings::wasvy::ecs::functions::Host for WasmHost<'_> {
    fn register_component(
        &mut self,
        path: wasmtime::component::__internal::String,
    ) -> Result<types::ComponentId, wasmtime::Error> {
        let type_registry = self
            .world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .clone();

        // This is a known type by the hosto no need to register it.
        if let Some(id) = type_id_for_path(&type_registry.read(), &path) {
            return Ok(self.world.components().get_id(id).expect("if the value exists in the type registry it should also exist in the world components.").index() as u64);
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

        Ok(id.index() as u64)
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
    ) -> Result<Option<types::ComponentId>, wasmtime::Error> {
        for component_info in self.world.components().iter_registered() {
            if *component_info.name() == path {
                return Ok(Some(component_info.id().index() as u64));
            }
        }

        Ok(None)
    }

    fn spawn(
        &mut self,
        components: std::vec::Vec<wasmtime::component::Resource<types::Component>>,
    ) -> Result<types::Entity, wasmtime::Error> {
        let type_registry = self.get_type_registry();
        let type_registry = type_registry.read();
        let registry = self.get_component_registry();

        let mut commands = self.world.commands();
        let mut entity = commands.spawn_empty();

        for component in components {
            insert_component(
                &mut entity,
                self.components.table.get(&component).unwrap().clone(),
                &type_registry,
                &registry,
            );
        }

        Ok(entity.id().index() as u64)
    }

    fn this_function_does_nothing(
        &mut self,
        _entry: crate::bindings::wasvy::ecs::functions::QueryResultEntry,
        _query_result: crate::bindings::wasvy::ecs::functions::QueryResult,
    ) {
    }
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
    registry
        .get_with_type_path(path)
        .map(|registration: &TypeRegistration| registration.type_id())
}

//TODO: Continue to fix
// Here is an example: https://docs.wasmtime.dev/api/wasmtime/component/bindgen_examples/_4_imported_resources/index.html
fn insert_component(
    entity: &mut EntityCommands,
    component: &types::Component,
    type_registry: &TypeRegistry,
    registry: &WasmComponentRegistry,
) {
    if let Some(component_id) = registry.get(&component.path) {
        insert_wasm_component(entity, component_id, component.value);
    } else {
        insert_host_component(entity, component, type_registry);
    }
}

fn insert_wasm_component(entity: &mut EntityCommands, component_id: &ComponentId, value: String) {
    unsafe {
        entity.insert_by_id(
            ComponentId::new(component_id.index()),
            WasmComponent {
                serialized_value: value,
            },
        );
    }
}

fn insert_host_component(
    entity: &mut EntityCommands,
    component: types::Component,
    type_registry: &TypeRegistry,
) {
    let type_registration = type_registry.get_with_type_path(&component.path).unwrap();

    let mut de = JsonDeserializer::from_str(&component.value);
    let reflect_deserializer = TypedReflectDeserializer::new(type_registration, type_registry);
    let output: Box<dyn PartialReflect> = reflect_deserializer.deserialize(&mut de).unwrap();

    let type_id = output.get_represented_type_info().unwrap().type_id();
    let reflect_from_reflect = type_registry
        .get_type_data::<ReflectFromReflect>(type_id)
        .unwrap();
    let value: Box<dyn Reflect> = reflect_from_reflect
        .from_reflect(output.as_partial_reflect())
        .unwrap();

    entity.insert_reflect(value);
}

impl WasmHost<'_> {
    fn get_type_registry(&self) -> AppTypeRegistry {
        self.world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .clone()
    }

    fn get_component_registry(&self) -> WasmComponentRegistry {
        self.world
            .get_resource::<WasmComponentRegistry>()
            .unwrap()
            .clone()
    }
}
