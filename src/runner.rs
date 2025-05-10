//! An abstraction to easily run WASM functions

use wasmtime::{
    Engine, Store, WasmParams, WasmResults,
    component::{Component, ComponentNamedList, ComponentType, Func, Lift, Linker, Lower, Val},
};

use crate::bindings::wasvy::ecs::types;

pub struct Runner<T: wasmtime_wasi::WasiView> {
    pub engine: Engine,
    linker: Linker<T>,
}

// Params: ComponentNamedList + Lower,
// Results: ComponentNamedList + Lift,
/// All the necessary data to run a WASM function.
pub struct WasmRunState<
    'a,
    T: wasmtime_wasi::WasiView,
    Params: ComponentNamedList + Lower,
    Results: ComponentNamedList + Lift,
> {
    pub component: &'a Component,
    pub store: Store<T>,
    pub function_name: String,
    //TODO: Hardcoding the param for the guest systems makes it impossible to run any wasm functon
    //using the runner.
    // pub params: Vec<types::QueryResult>,
    pub params: Params,
    // pub results: &'a mut [Val],
    pub results: &'a mut Results,
}

// #[derive(ComponentType)]
// struct Blah {
//     #[component]
//     aa: Vec<types::QueryResult>
// }

// unsafe impl ComponentNamedList for Blah {}

impl<T: wasmtime_wasi::WasiView> Runner<T> {
    pub fn new(engine: Engine) -> Self {
        Self {
            linker: Linker::<T>::new(&engine),
            engine,
        }
    }

    /// Add the sync version of the WASI interfaces
    ///
    /// https://wasi.dev/interfaces
    pub fn add_wasi_sync(&mut self) {
        wasmtime_wasi::add_to_linker_sync(&mut self.linker).expect("Could not add wasi to linker");
    }

    /// Use this function to add custom functionality that will be passed to the WASM module.
    pub fn add_functionality<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Linker<T>),
    {
        f(&mut self.linker);
    }

    pub fn run_function<Params: ComponentNamedList + Lower, Results: ComponentNamedList + Lift>(
        &self,
        mut state: WasmRunState<'_, T, Params, Results>,
    ) {
        let instance = self
            .linker
            .instantiate(&mut state.store, state.component)
            .unwrap();

        let func = instance
            // .get_typed_func::<(Vec<types::QueryResult>, u64), ()>(
            .get_typed_func::<Params, Results>(&mut state.store, state.function_name.clone())
            .expect("WASM function with the given name wasn't found");

        let _ = func.call(state.store, state.params);

        // let typed = func.typed(&state.store);

        // let func.typed(state.store)
        // func.call(state.store, state.params, state.results)
        //     .expect("failed to run the desired function");
    }
}
