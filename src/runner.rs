use wasmtime::{
    Engine, Store,
    component::{Component, Func, Linker, Val},
};

pub struct Runner<T: wasmtime_wasi::WasiView> {
    pub engine: Engine,
    linker: Linker<T>,
}

pub struct WasmRunState<'a, T: wasmtime_wasi::WasiView> {
    pub component: &'a Component,
    pub store: Store<T>,
    pub function_name: String,
    pub params: &'a [Val],
    pub results: &'a mut [Val],
}

impl<T: wasmtime_wasi::WasiView> Runner<T> {
    pub fn new(engine: Engine) -> Self {
        Self {
            linker: Linker::<T>::new(&engine),
            engine,
        }
    }

    pub fn add_wasi_sync(&mut self) {
        wasmtime_wasi::add_to_linker_sync(&mut self.linker).expect("Could not add wasi to linker");
    }

    pub fn add_functionality_to_linker<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Linker<T>),
    {
        f(&mut self.linker);
    }

    pub fn run_function(&self, mut state: WasmRunState<'_, T>) {
        let instance = self
            .linker
            .instantiate(&mut state.store, state.component)
            .unwrap();

        let func: Func = instance
            .get_func(&mut state.store, state.function_name.clone())
            .expect("`WasmGuestSystem system name` not found");

        func.call(state.store, state.params, state.results);
    }
}

// impl<T: wasmtime_wasi::WasiView> Default for Runner<T> {
//     fn default() -> Self {
//         let engine = Engine::default();
//         Self {
//             linker: Linker::<T>::new(&engine),
//             engine,
//         }
//     }
// }
