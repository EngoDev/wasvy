pub mod asset;
pub mod component_registry;
pub mod host;
pub mod plugin;
pub mod prelude;
pub mod runner;
pub mod systems;
pub mod state;


mod bindings {
    wasmtime::component::bindgen!("host" in "wit/ecs/ecs.wit");
}
