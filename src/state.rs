use wasmtime_wasi::{
    ResourceTable, WasiCtx, WasiCtxBuilder, WasiView, IoView,
};

use crate::host::WasmHost;

/// The state object that houses the functionality that is passed to WASM components.
pub struct States<'a> {
    table: ResourceTable,
    ctx: WasiCtx,
    pub host_ecs: WasmHost<'a>,
}

impl<'a> States<'a> {
    pub fn new(host_ecs: WasmHost<'a>) -> Self {
        let table = ResourceTable::new();
        let ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_network()
            .allow_ip_name_lookup(true)
            .build();
        Self {
            table,
            ctx,
            host_ecs,
        }
    }
}

impl IoView for States<'_> {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasiView for States<'_> {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}