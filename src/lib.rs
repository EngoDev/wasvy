pub mod asset;
pub mod component_registry;
pub mod host;
pub mod mods;
pub mod plugin;
pub mod prelude;
pub mod runner;
pub mod state;
pub mod systems;

mod bindings {
    wasmtime::component::bindgen!("host" in "wit/ecs/ecs.wit");
}
