pub mod asset;
pub mod component;
pub mod component_registry;
pub mod host;
pub mod plugin;
pub mod prelude;
pub mod runner;
pub mod state;
pub mod systems;

pub struct Test;

mod bindings {
    // wasmtime::component::bindgen!("host" in "wit/ecs/ecs.wit");
    wasmtime::component::bindgen!({
        world: "host",
        path: "wit/ecs/ecs.wit",
        with: {
            // "wasvy:ecs/types/component": crate::component::WasmComponents,
            "wasvy:ecs/types/component": crate::component::WasmComponentResource,
        },
    });
}
