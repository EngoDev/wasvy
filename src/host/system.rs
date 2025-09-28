use std::mem::replace;

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
        world::{FromWorld, World},
    },
    log::trace,
    prelude::{Assets, Res},
};

pub struct System {
    name: String,
    params: Vec<Param>,
    built: bool,
}

enum Param {
    Commands,
}

impl System {
    pub(crate) fn build(
        &mut self,
        mut world: &mut World,
        mod_name: &str,
        asset_id: &AssetId<ModAsset>,
        asset_version: &Tick,
    ) -> Result<BoxedSystem> {
        if self.built {
            bail!("System was already added to the app");
        }
        self.built = true;

        let system = (
            LocalBuilder(Input {
                mod_name: mod_name.to_string(),
                system_name: self.name.clone(),
                asset_id: asset_id.clone(),
                asset_version: asset_version.clone(),
                params: replace(&mut self.params, Vec::new()),
            }),
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
            // TODO: FilteredResourcesMutParamBuilder::new(|builder| {}),
            // TODO: QueryParamBuilder::new_box(|builder| {}),
        )
            .build_state(&mut world)
            .build_system(system_runner)
            .with_name(format!("wasvy[{mod_name}]::{}", self.name));

        let boxed_system = Box::new(IntoSystem::into_system(system));

        Ok(boxed_system)
    }
}

#[derive(FromWorld)]
struct Input {
    mod_name: String,
    system_name: String,
    asset_id: AssetId<ModAsset>,
    asset_version: Tick,
    params: Vec<Param>,
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

    let mut runner = Runner::new(&engine);

    // Setup system param resources
    let params: Vec<_> = input
        .params
        .iter()
        .map(|param| match param {
            Param::Commands => runner.new_resource(Commands),
        })
        .map(|resource| Val::Resource(resource))
        .collect();

    trace!(
        "Running system \"{}\" from \"{}\"",
        input.system_name, input.mod_name
    );
    let result = asset.run_system(
        &mut runner,
        &input.system_name,
        ConfigRunSystem {
            commands: &mut commands,
        },
        &params,
    );
    trace!("got result {:?}", result);
}

impl HostSystem for WasmHost {
    fn new(&mut self, name: String) -> Result<Resource<System>> {
        let State::Setup { table, .. } = self.access() else {
            bail!("Systems can only be instantiated in a setup function")
        };

        Ok(table.push(System {
            built: false,
            name,
            params: Vec::new(),
        })?)
    }

    fn add_commands(&mut self, system: Resource<System>) -> Result<()> {
        let State::Setup { table, .. } = self.access() else {
            bail!("Systems can only be modified in a setup function")
        };

        let system = table.get_mut(&system)?;
        system.params.push(Param::Commands);

        Ok(())
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
