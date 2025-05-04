use bevy::reflect::serde::ReflectDeserializerProcessor;

pub struct WasmDeserializerProcessor {
    pub type_path: String,
}

impl ReflectDeserializerProcessor for WasmDeserializerProcessor {
    fn try_deserialize<'de, D>(
        &mut self,
        registration: &bevy::reflect::TypeRegistration,
        registry: &bevy::reflect::TypeRegistry,
        deserializer: D,
    ) -> Result<Result<Box<dyn bevy::reflect::PartialReflect>, D>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Err(deserializer))
    }
}
