use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};

use crate::{asset::ModAsset, mods::Mod};

/// Group all the system params we neeed to allow shared access from one &mut world
#[derive(SystemParam)]
pub struct Setup<'w, 's> {
    events: MessageReader<'w, 's, AssetEvent<ModAsset>>,
    assets: ResMut<'w, Assets<ModAsset>>,
    mods: Query<'w, 's, (Entity, Option<&'static Name>, &'static Mod)>,
}

pub(crate) fn run_setup(mut world: &mut World, param: &mut SystemState<Setup>) {
    let Setup {
        mut events,
        mut assets,
        mods,
    } = param.get_mut(world);

    let mut setup = Vec::new();
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            let Some(asset) = assets.get_mut_untracked(*id).map(ModAsset::take) else {
                continue;
            };

            // Find the mod entity matching this asset
            let Some((entity, name, _)) = mods.iter().find(|&(_, _, m)| m.asset.id() == *id) else {
                warn!(
                    "Loaded wasm mod asset, but missing its entity. Did you accidentally load a wasm asset?"
                );
                continue;
            };

            let name = name
                .and_then(|name| Some(name.as_str()))
                .unwrap_or("unknown")
                .to_string();

            setup.push((asset, *id, entity, name));
        }
    }

    for (asset, asset_id, entity, name) in setup {
        let result = asset.setup(&mut world, &asset_id, &name);

        let mut assets = world.get_resource_mut::<Assets<ModAsset>>().unwrap();
        match result {
            Ok(asset) => {
                info!("Successfully loaded mod \"{}\"", name);

                assets.get_mut(asset_id).unwrap().put(asset);
            }
            Err(err) => {
                error!("Error loading mod \"{}\":\n{:?}", name, err);

                assets.remove(asset_id);
                drop(assets);
                world.despawn(entity);
            }
        }
    }
}
