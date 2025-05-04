use bevy::{ecs::component::ComponentId, platform::collections::HashMap, prelude::*};

pub type TypePath = String;

/// Registry for storing the components that are registered from WASM assets.
#[derive(Default, Clone, Debug, Resource, Deref, DerefMut)]
pub struct WasmComponentRegistry(pub HashMap<TypePath, ComponentId>);
