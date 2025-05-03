pub mod asset;
pub mod host;
pub mod plugin;
pub mod prelude;
pub mod runner;

mod bindings {
    wasmtime::component::bindgen!("host" in "wit/ecs/ecs.wit");
}
