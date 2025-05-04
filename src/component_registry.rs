use bevy::{ecs::component::ComponentId, platform::collections::HashMap, prelude::*};

pub type TypePath = String;

#[derive(Default, Clone, Debug, Resource, Deref, DerefMut)]
pub struct WasmComponentRegistry {
    pub data: HashMap<TypePath, ComponentId>,
}
