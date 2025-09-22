use bevy::prelude::*;

use crate::state::{WasmHost, Scope};

/// Cross engine instatiation of WASM components is not supported.
/// This resources is the global [`Engine`](wasmtime::Engine) that is used for instatiation.
///
/// Check the wasmtime [`Engine`](wasmtime::Engine) docs for more information.
#[derive(Resource, Clone, Deref)]
pub(crate) struct Engine(wasmtime::Engine);

pub(crate) type Store = wasmtime::Store<WasmHost>;

impl Engine {
    pub(crate) fn new() -> Self {
        let engine = wasmtime::Engine::default();
        Self(engine)
    }

    pub(crate) fn inner(&self) -> wasmtime::Engine {
        self.0.clone()
    }

    pub(crate) fn use_store<'s, F, R>(&self, scope: Scope<'s>, mut f: F) -> R
    where
        F: FnMut(Store) -> R,
    {
        let data = WasmHost::new();
        let _guard = data.scope(scope);

        let store = Store::new(&self.0, data);
        f(store)
    }
}
