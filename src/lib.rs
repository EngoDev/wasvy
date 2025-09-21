pub mod asset;
pub mod component_registry;
pub mod engine;
pub mod mods;
pub mod plugin;
pub mod prelude;
pub mod state;

mod bindings {
    wasmtime::component::bindgen!({
        path: "wit/ecs/ecs.wit",
        world: "host",
        // Interactions with `ResourceTable` can possibly trap so enable the ability
        // to return traps from generated functions.
        imports: { default: trappable },
    });
}
