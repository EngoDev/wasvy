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

use crate::bindings::wasvy::ecs::types::HostComponent;

// struct WasmComponents<'a, T: Reflect + IsAligned> {
pub struct WasmComponents {
    pub(crate) table: ResourceTable,
    // ptr: PtrMut<'a, T>,
    // ptr: &'a mut T,
    registry: TypeRegistryArc,
    types: HashMap<u32, TypeRegistration>,
}

// #[derive(Debug, Clone)]
pub struct WasmComponentResource {
    pub(crate) value: Box<dyn Reflect>,
    pub type_data: TypeRegistration,
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

// impl<'a, T: Reflect + IsAligned> HostComponent for WasmComponent<'a, T> {
impl HostComponent for WasmComponents {
    fn new(
        &mut self,
        value: wasmtime::component::__internal::String,
        path: wasmtime::component::__internal::String,
    ) -> Result<wasmtime::component::Resource<WasmComponentResource>, wasmtime::Error> {
        let registry = self.registry.read();
        let type_registration = registry.get_with_type_path(&path).unwrap();

        let mut de = JsonDeserializer::from_str(&value);
        let reflect_deserializer = TypedReflectDeserializer::new(type_registration, &registry);
        let output: Box<dyn PartialReflect> = reflect_deserializer.deserialize(&mut de).unwrap();

        let type_id = output.get_represented_type_info().unwrap().type_id();
        let reflect_from_reflect = registry
            .get_type_data::<ReflectFromReflect>(type_id)
            .unwrap();

        let value: Box<dyn Reflect> = reflect_from_reflect
            .from_reflect(output.as_partial_reflect())
            .unwrap();

        let id = self
            .table
            .push(WasmComponentResource {
                value,
                type_data: type_registration.clone(),
            })
            .unwrap();

        self.types.insert(id.rep(), type_registration.clone());

        // let id = self.table.push(WasmComponent {
        //     val: Box::new(type_registration.data::<ReflectFromPtr>().unwrap().from_ptr(value)),
        //     type_data: type_registration
        // }).unwrap();

        Ok(id)
    }
    // fn new(&mut self,value:wasmtime::component::__internal::String,path:wasmtime::component::__internal::String,) -> wasmtime::component::Resource<WasmComponent> {
    //     let id = self.table.push(entry)
    // }
    fn set(
        &mut self,
        resource: wasmtime::component::Resource<WasmComponentResource>,
        value: wasmtime::component::__internal::String,
    ) -> Result<(), wasmtime::Error> {
        let registration = self.types.get(&resource.rep()).unwrap();
        let registry = self.registry.read();
        // let registration = self.registry.get(TypeId::of::<T>()).unwrap();
        // let reflect_from_ptr = registration.data::<ReflectFromPtr>().unwrap();
        let mut de = JsonDeserializer::from_str(&value);
        let reflect_deserializer = TypedReflectDeserializer::new(registration, &registry);
        let output: Box<dyn PartialReflect> = reflect_deserializer.deserialize(&mut de).unwrap();

        let type_id = output.get_represented_type_info().unwrap().type_id();
        let reflect_from_reflect = registry
            .get_type_data::<ReflectFromReflect>(type_id)
            .unwrap();
        let value: Box<dyn Reflect> = reflect_from_reflect
            .from_reflect(output.as_partial_reflect())
            .unwrap();
        // let raw_ptr = value.set(value)

        self.table.get_mut(&resource).unwrap().value.set(value);
        // self.ptr.set(value);
        // resource.
        Ok(())
    }
    fn get(
        &mut self,
        resource: wasmtime::component::Resource<WasmComponentResource>,
    ) -> wasmtime::Result<wasmtime::component::__internal::String> {
        let registry = self.registry.read();
        // let registration = self.types.get(&resource.rep()).unwrap();
        let data = &self.table.get(&resource).unwrap().value;

        let serializer = TypedReflectSerializer::new(data.as_reflect(), &registry);
        Ok(serde_json::to_string(&serializer).unwrap())
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
