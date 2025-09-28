use crate::{
    asset::ModAsset,
    engine::Engine,
    runner::{ConfigRunSystem, Runner},
};

use super::*;
use bevy::{
    asset::AssetId,
    ecs::{
        component::Tick,
        system::{
            BoxedSystem, Commands, IntoSystem, Local, LocalBuilder, ParamBuilder,
            SystemParamBuilder,
        },
        world::FromWorld,
    },
    log::trace,
    prelude::{Assets, Res},
};

pub struct System(pub(crate) Option<BoxedSystem>);

#[derive(FromWorld)]
struct Input {
    mod_name: String,
    system_name: String,
    asset_id: AssetId<ModAsset>,
    asset_version: Tick,
}

fn system_runner(
    input: Local<Input>,
    assets: Res<Assets<ModAsset>>,
    engine: Res<Engine>,
    mut commands: Commands,
    // TODO: mut resources: FilteredResourcesMut,
    // TODO: mut query: Query<FilteredEntityMut>,
) {
    // Skip no longer loaded mods
    let Some(asset) = assets.get(input.asset_id) else {
        return;
    };

    // Skip mismatching system versions
    if asset.version() != Some(input.asset_version) {
        return;
    }

    trace!(
        "Running system \"{}\" from \"{}\"",
        input.system_name, input.mod_name
    );
    let mut runner = Runner::new(&engine);
    let result = asset.run_system(
        &mut runner,
        &input.system_name,
        ConfigRunSystem {
            commands: &mut commands,
        },
    );
    trace!("got result {:?}", result);
}

impl HostSystem for WasmHost {
    fn new(&mut self, system_name: String) -> Result<Resource<System>> {
        let State::Setup {
            table,
            mod_name,
            asset_id,
            asset_version,
            mut world,
            ..
        } = self.access()
        else {
            bail!("Systems can only be instantiated in a setup function")
        };

        let system = (
            LocalBuilder(Input {
                mod_name: mod_name.to_string(),
                system_name: system_name.clone(),
                asset_id: asset_id.clone(),
                asset_version: asset_version.clone(),
            }),
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
            // TODO: FilteredResourcesMutParamBuilder::new(|builder| {}),
            // TODO: QueryParamBuilder::new_box(|builder| {}),
        )
            .build_state(&mut world)
            .build_system(system_runner)
            .with_name(format!("wasvy[{mod_name}]::{system_name}"));

        let boxed_system = Box::new(IntoSystem::into_system(system));

        Ok(table.push(System(Some(boxed_system)))?)
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
