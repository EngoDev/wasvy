use bevy::prelude::*;

/// Cross engine instatiation of WASM components is not supported.
/// This resources is the global [`Engine`](wasmtime::Engine) that is used for instatiation.
///
/// Check the wasmtime [`Engine`](wasmtime::Engine) docs for more information.
#[derive(Resource, Clone, Deref)]
pub(crate) struct Engine(wasmtime::Engine);

impl Engine {
    pub(crate) fn new() -> Self {
        let engine = wasmtime::Engine::default();
        Self(engine)
    }

    pub(crate) fn inner(&self) -> &wasmtime::Engine {
        &self.0
    }
}
