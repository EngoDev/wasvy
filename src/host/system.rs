use crate::{asset::ModAsset, engine::Engine};

use super::*;
use bevy::{
    ecs::system::{BoxedSystem, IntoSystem},
    prelude::{Assets, Res, info},
};

pub struct System(pub(crate) Option<BoxedSystem>);

impl HostSystem for HostState {
    fn new(&mut self, name: String) -> Result<Resource<System>> {
        self.access(move |state| {
            let State::Setup {
                table,
                mod_name,
                asset_id,
                asset_version,
                ..
            } = state
            else {
                bail!("Systems can only be instantiated in a setup function")
            };

            let mod_name = mod_name.to_string();
            let system_name = name.clone();
            let asset_id = asset_id.clone();
            let asset_version = asset_version.clone();

            let boxed_system = Box::new(IntoSystem::into_system(
                move |engine: Res<Engine>, assets: Res<Assets<ModAsset>>| {
                    // Skip no longer loaded mods
                    let Some(asset) = assets.get(asset_id) else {
                        return;
                    };

                    // Skip mismatching system versions
                    if asset.version != asset_version {
                        return;
                    }

                    info!("Running system \"{}\" from \"{}\"", system_name, mod_name);
                    let result = asset.run_system(&engine, &system_name);
                    info!("got result {:?}", result);
                },
            ));

            Ok(table.push(System(Some(boxed_system)))?)
        })
    }

    fn add_commands(&mut self, _self: Resource<System>) -> Result<()> {
        bail!("Unimplemented")
    }

    fn add_query(&mut self, _self: Resource<System>, _query: Vec<QueryFor>) -> Result<()> {
        bail!("Unimplemented")
    }

    fn before(&mut self, _self: Resource<System>, _other: Resource<System>) -> Result<()> {
        bail!("Unimplemented")
    }

    fn after(&mut self, _self: Resource<System>, _other: Resource<System>) -> Result<()> {
        bail!("Unimplemented")
    }

    fn drop(&mut self, _rep: Resource<System>) -> Result<()> {
        Ok(())
    }
}
