use super::*;
use anyhow::anyhow;
use bevy::prelude::Update;

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
                let system = table.get_mut(system)?;
                let boxed_system = system
                    .0
                    .take()
                    .ok_or(anyhow!("System was already added to the app"))?;

                let schedule = match schedule {
                    Schedule::Update => Update,
                };

                schedules.add_systems(schedule, boxed_system);
            }

            Ok(())
        })
    }

    fn drop(&mut self, _rep: Resource<App>) -> Result<()> {
        Ok(())
    }
}
