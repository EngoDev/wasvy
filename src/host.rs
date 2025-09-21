use anyhow::bail;
use bevy::{app::Update, log::info};
use wasmtime::{Result, component::Resource};

use crate::{
    bindings::wasvy::ecs::app::{
        Host, HostApp, HostCommands, HostComponent, HostQuery, HostSystem, QueryFor, Schedule,
        SerializedComponent,
    },
    state::{HostState, State},
};

impl Host for HostState {}

pub struct App;

impl HostApp for HostState {
    fn new(&mut self) -> Result<Resource<App>> {
        self.access(|state| {
            let State::Setup { table, app, .. } = state else {
                bail!("App can only be instantiated in a setup function")
            };

            if app.is_some() {
                bail!("App can only be instantiated once")
            }

            let app_res = table.push(App)?;
            *app = Some(app_res.rep());

            Ok(app_res)
        })
    }

    fn add_systems(
        &mut self,
        _self: Resource<App>,
        schedule: Schedule,
        systems: Vec<Resource<System>>,
    ) -> Result<()> {
        self.access(move |state| {
            let State::Setup {
                table, schedules, ..
            } = state
            else {
                unreachable!()
            };

            for system in systems.iter() {
                let system = table.get(system)?;
                let name = system.name.clone();

                let schedule = match schedule {
                    Schedule::Update => Update,
                };
                schedules.add_systems(schedule, move || {
                    info!("TODO: Run system {}", name);
                });
            }

            Ok(())
        })
    }

    fn drop(&mut self, _rep: Resource<App>) -> Result<()> {
        Ok(())
    }
}

pub struct System {
    name: String,
}

impl HostSystem for HostState {
    fn new(&mut self, name: String) -> Result<Resource<System>> {
        self.access(move |state| {
            let State::Setup { table, .. } = state else {
                bail!("Systems can only be instantiated in a setup function")
            };

            let name = name.clone();
            Ok(table.push(System { name })?)
        })
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

pub struct Commands;

impl HostCommands for HostState {
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

pub struct Query;

impl HostQuery for HostState {
    fn iter(&mut self, __self: Resource<Query>) -> Result<Option<Vec<Resource<Component>>>> {
        bail!("Unimplemented")
    }

    fn drop(&mut self, _rep: Resource<Query>) -> Result<()> {
        Ok(())
    }
}

pub struct Component;

impl HostComponent for HostState {
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
