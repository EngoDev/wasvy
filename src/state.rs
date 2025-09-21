use anyhow::bail;
use wasmtime::{
    Result,
    component::{HasData, Resource},
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};

use crate::{
    bindings::wasvy::ecs::app::{
        App, Commands, Component, Host, HostApp, HostCommands, HostComponent, HostQuery,
        HostSystem, Query, QueryFor, Schedule, SerializedComponent, System,
    },
    engine::Engine,
};

pub(crate) type Store = wasmtime::Store<State>;

pub(crate) fn new_store(engine: &Engine) -> Store {
    Store::new(
        engine,
        State {
            wasi_ctx: WasiCtx::builder().build(),
            resource_table: ResourceTable::new(),
        },
    )
}

pub(crate) struct State {
    wasi_ctx: WasiCtx,
    resource_table: ResourceTable,
}

impl HasData for State {
    type Data<'a> = WasiCtxView<'a>;
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

impl Host for State {}

impl HostApp for State {
    fn new(&mut self) -> Result<Resource<App>> {
        bail!("Unimplemented")
    }

    fn add_systems(
        &mut self,
        _self: Resource<App>,
        _schedule: Schedule,
        _systems: Vec<Resource<System>>,
    ) -> Result<()> {
        Ok(())
    }

    fn drop(&mut self, _rep: Resource<App>) -> Result<()> {
        Ok(())
    }
}

impl HostSystem for State {
    fn new(&mut self, _name: String) -> Result<Resource<System>> {
        bail!("Unimplemented")
    }

    fn add_commands(&mut self, _self: Resource<System>) -> Result<()> {
        Ok(())
    }

    fn add_query(&mut self, _self: Resource<System>, _query: Vec<QueryFor>) -> Result<()> {
        Ok(())
    }

    fn before(&mut self, _self: Resource<System>, _other: Resource<System>) -> Result<()> {
        Ok(())
    }

    fn after(&mut self, _self: Resource<System>, _other: Resource<System>) -> Result<()> {
        Ok(())
    }

    fn drop(&mut self, _rep: Resource<System>) -> Result<()> {
        Ok(())
    }
}

impl HostCommands for State {
    fn spawn(
        &mut self,
        _self: Resource<Commands>,
        _components: Vec<Resource<Component>>,
    ) -> Result<()> {
        bail!("Unimplemented")
    }

    fn drop(&mut self, _rep: Resource<Commands>) -> Result<()> {
        Ok(())
    }
}

impl HostQuery for State {
    fn iter(&mut self, __self: Resource<Query>) -> Result<Option<Vec<Resource<Component>>>> {
        bail!("Unimplemented")
    }

    fn drop(&mut self, _rep: Resource<Query>) -> Result<()> {
        Ok(())
    }
}

impl HostComponent for State {
    fn get(&mut self, _self: Resource<Component>) -> Result<SerializedComponent> {
        bail!("Unimplemented")
    }

    fn set(&mut self, _self: Resource<Component>, _value: SerializedComponent) -> Result<()> {
        Ok(())
    }

    fn drop(&mut self, _rep: Resource<Component>) -> Result<()> {
        Ok(())
    }
}
