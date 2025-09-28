use std::alloc::Layout;

use bevy::{
    ecs::{
        component::{ComponentDescriptor, ComponentId},
        reflect::ReflectCommandExt,
    },
    platform::collections::HashMap,
    prelude::*,
    reflect::serde::TypedReflectDeserializer,
};
use serde::de::DeserializeSeed;

pub type TypePath = String;

/// Registry for storing the components that are registered from WASM assets.
#[derive(Default, Clone, Debug, Resource, Deref, DerefMut)]
pub struct WasmComponentRegistry(pub HashMap<TypePath, ComponentId>);

/// This component is the wrapper component for all the Bevy components that are registered in a
/// WASM.
///
/// # Description
///
/// When you call the spawn method in WASM you need to provide a component id, that id is used to
/// add a new [`WasmComponent`] under that id with the `serialized_value` that is given.
///
/// This approach makes it possible to register components that don't exist in Rust.
#[derive(Component, Reflect)]
pub struct WasmComponent {
    pub serialized_value: String,
}

/// A command that registers and adds a component to an entity
struct RegisterWasmComponent {
    entity: Entity,
    serialized_value: String,
    type_path: String,
}

impl Command for RegisterWasmComponent {
    fn apply(self, world: &mut World) {
        let value = WasmComponent {
            serialized_value: self.serialized_value,
        };

        // Avoid duplicate registrations
        let component_registry = world.get_resource::<WasmComponentRegistry>().unwrap();
        let component_id = if let Some(id) = component_registry.get(&self.type_path) {
            id.clone()
        } else {
            // Safety:
            // - the drop fn is usable on this component type
            // - the component is safe to access from any thread
            let descriptor = unsafe {
                ComponentDescriptor::new_with_layout(
                    self.type_path.clone(),
                    WasmComponent::STORAGE_TYPE,
                    Layout::new::<WasmComponent>(),
                    Some(|ptr| {
                        ptr.drop_as::<WasmComponent>();
                    }),
                    true,
                    WasmComponent::clone_behavior(),
                )
            };

            let id = world.register_component_with_descriptor(descriptor);

            let mut component_registry = world.get_resource_mut::<WasmComponentRegistry>().unwrap();
            component_registry.insert(self.type_path, id);

            id
        };

        let mut commands = world.commands();
        let mut entity_commands = commands.entity(self.entity);

        // Safety:
        // - ComponentId is from the same world as self.
        // - T has the same layout as the one passed during component_id creation.
        unsafe { entity_commands.insert_by_id(component_id, value) };
    }
}

pub(crate) fn insert_component(
    commands: &mut Commands,
    type_registry: &AppTypeRegistry,
    component_registry: &WasmComponentRegistry,
    entity: Entity,
    type_path: String,
    serialized_value: String,
) {
    let type_registry = type_registry.read();

    // Insert types that are known by bevy (inserted as concrete types)
    if let Some(type_registration) = type_registry.get_with_type_path(&type_path) {
        let mut de = serde_json::Deserializer::from_str(&serialized_value);
        let reflect_deserializer = TypedReflectDeserializer::new(type_registration, &type_registry);
        let output: Box<dyn PartialReflect> = reflect_deserializer.deserialize(&mut de).unwrap();

        commands.entity(entity).insert_reflect(output);
    }
    // Handle guest types (inserted as json strings)
    else if let Some(component_id) = component_registry.get(&type_path) {
        let value = WasmComponent { serialized_value };
        let mut entity_commands = commands.entity(entity);

        // Safety:
        // - ComponentId must be from the same world as self.
        // - T must have the same layout as the one passed during component_id creation.
        unsafe { entity_commands.insert_by_id(component_id.clone(), value) };
    }
    // Finally, for guest types that are not registered, we can register and insert them via a command
    else {
        commands.queue(RegisterWasmComponent {
            entity,
            serialized_value,
            type_path,
        });
    }
}
