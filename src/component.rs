use serde::de::DeserializeSeed;
use serde_json::Deserializer as JsonDeserializer;
use wasmtime_wasi::ResourceTable;

use bevy::{
    platform::collections::HashMap,
    reflect::{
        PartialReflect, Reflect, ReflectFromPtr, ReflectFromReflect, TypeRegistration,
        TypeRegistry, TypeRegistryArc,
        serde::{TypedReflectDeserializer, TypedReflectSerializer},
    },
};

use crate::{bindings::wasvy::ecs::types::HostComponent, plugin::WasmComponent};

// struct WasmComponents<'a, T: Reflect + IsAligned> {
pub struct WasmComponents {
    pub(crate) table: ResourceTable,
    // ptr: PtrMut<'a, T>,
    // ptr: &'a mut T,
    registry: TypeRegistryArc,
    types: HashMap<u32, TypeRegistration>,
}

// #[derive(Debug, Clone)]
pub struct HostWasmComponentResource {
    pub(crate) value: Box<dyn Reflect>,
    pub type_data: TypeRegistration,
}

pub enum WasmComponentResource {
    Host(HostWasmComponentResource),
    Guest(WasmComponent),
}

// impl<'a, T: Reflect + IsAligned> WasmComponent<'a, T> {
impl WasmComponents {
    // pub fn new(component: &'a mut T, registry: &'a TypeRegistry) -> WasmComponent<'a, T> {
    //     // let raw_ptr = component as *mut T as *mut u8;
    //     // let ptr = unsafe { PtrMut::new(std::ptr::NonNull::new(raw_ptr).unwrap()) };
    //     Self {
    //         ptr: component,
    //         registry,
    //     }
    // }
    pub fn new(registry: TypeRegistryArc) -> Self {
        Self {
            table: ResourceTable::new(),
            registry,
            types: HashMap::default(),
        }
    }
}

impl crate::bindings::wasvy::ecs::types::Host for WasmComponents {}

// impl<'a, T: Reflect + IsAligned> HostComponent for WasmComponent<'a, T> {
impl HostComponent for WasmComponents {
    fn new(
        &mut self,
        value: wasmtime::component::__internal::String,
        path: wasmtime::component::__internal::String,
        // ) -> Result<wasmtime::component::Resource<WasmComponentResource>, wasmtime::Error> {
    ) -> wasmtime::component::Resource<WasmComponentResource> {
        let registry = self.registry.read();
        println!("Path: {:?}", path);
        // The component is a host component
        if let Some(type_registration) = registry.get_with_type_path(&path) {
            let mut de = JsonDeserializer::from_str(&value);
            let reflect_deserializer = TypedReflectDeserializer::new(type_registration, &registry);
            let output: Box<dyn PartialReflect> =
                reflect_deserializer.deserialize(&mut de).unwrap();

            let type_id = output.get_represented_type_info().unwrap().type_id();
            let reflect_from_reflect = registry
                .get_type_data::<ReflectFromReflect>(type_id)
                .unwrap();

            let value: Box<dyn Reflect> = reflect_from_reflect
                .from_reflect(output.as_partial_reflect())
                .unwrap();

            let id = self
                .table
                .push(WasmComponentResource::Host(HostWasmComponentResource {
                    value,
                    type_data: type_registration.clone(),
                }))
                .unwrap();

            self.types.insert(id.rep(), type_registration.clone());

            id
        // The component is a guest component
        } else {
            let component = WasmComponent {
                type_path: path,
                value: value.reflect_clone().unwrap(),
            };

            let id = self
                .table
                .push(WasmComponentResource::Guest(component))
                .unwrap();

            id
        }

        // let id = self.table.push(WasmComponent {
        //     val: Box::new(type_registration.data::<ReflectFromPtr>().unwrap().from_ptr(value)),
        //     type_data: type_registration
        // }).unwrap();
    }
    // fn new(&mut self,value:wasmtime::component::__internal::String,path:wasmtime::component::__internal::String,) -> wasmtime::component::Resource<WasmComponent> {
    //     let id = self.table.push(entry)
    // }
    fn set(
        &mut self,
        resource: wasmtime::component::Resource<WasmComponentResource>,
        value: wasmtime::component::__internal::String,
        // ) -> Result<(), wasmtime::Error> {
    ) {
        let value = if let Some(registration) = self.types.get(&resource.rep()) {
            let registry = self.registry.read();
            // let registration = self.registry.get(TypeId::of::<T>()).unwrap();
            // let reflect_from_ptr = registration.data::<ReflectFromPtr>().unwrap();
            let mut de = JsonDeserializer::from_str(&value);
            let reflect_deserializer = TypedReflectDeserializer::new(registration, &registry);
            let output: Box<dyn PartialReflect> =
                reflect_deserializer.deserialize(&mut de).unwrap();

            let type_id = output.get_represented_type_info().unwrap().type_id();
            let reflect_from_reflect = registry
                .get_type_data::<ReflectFromReflect>(type_id)
                .unwrap();
            let value: Box<dyn Reflect> = reflect_from_reflect
                .from_reflect(output.as_partial_reflect())
                .unwrap();

            value
            // let raw_ptr = value.set(value)
        } else {
            value.reflect_clone().unwrap()
        };

        match self.table.get_mut(&resource).unwrap() {
            WasmComponentResource::Host(component) => component.value.set(value),
            WasmComponentResource::Guest(component) => component.value.set(value),
        };

        // self.ptr.set(value);
        // resource.
    }
    fn get(
        &mut self,
        resource: wasmtime::component::Resource<WasmComponentResource>,
        // ) -> wasmtime::Result<wasmtime::component::__internal::String> {
    ) -> wasmtime::component::__internal::String {
        let registry = self.registry.read();
        // let registration = self.types.get(&resource.rep()).unwrap();
        match self.table.get(&resource).unwrap() {
            WasmComponentResource::Host(component) => {
                // let data = &self.table.get(&resource).unwrap().value;
                let serializer =
                    TypedReflectSerializer::new(component.value.as_reflect(), &registry);
                serde_json::to_string(&serializer).unwrap()
            }

            WasmComponentResource::Guest(component) => component
                .value
                .downcast_ref::<String>()
                .unwrap()
                .to_string(),
        }
    }
    //
    // fn get(
    //     &mut self,
    //     self_: wasmtime::component::Resource<crate::bindings::wasvy::ecs::types::Component>,
    // ) -> wasmtime::component::__internal::String {
    // }

    fn drop(
        &mut self,
        resource: wasmtime::component::Resource<WasmComponentResource>,
    ) -> wasmtime::Result<()> {
        self.table.delete(resource)?;
        // drop(self.ptr);

        Ok(())
    }
}
