use crate::{asset::ModAsset, engine::Engine, mods::Mod};
use bevy::prelude::*;

pub(crate) fn run_setup(
    mut events: MessageReader<AssetEvent<ModAsset>>,
    assets: Res<Assets<ModAsset>>,
    mut schedules: ResMut<Schedules>,
    engine: Res<Engine>,
    mut commands: Commands,
    mut mods: Query<(Entity, Option<&Name>, &Mod)>,
) {
    for event in events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                let asset = assets.get(*id).unwrap();

                // Find the mod entity matching this asset
                let Some((entity, name, _)) =
                    mods.iter_mut().find(|&(_, _, m)| m.asset.id() == *id)
                else {
                    warn!(
                        "Loaded wasm mod, but missing it's entity. Did you accidentally load a wasm asset?"
                    );
                    continue;
                };

                let name = name
                    .and_then(|name| Some(name.as_str()))
                    .unwrap_or("unknown");

                match asset.setup(&engine, &mut schedules) {
                    Ok(()) => info!("Successfully loaded mod \"{}\"", name),
                    Err(err) => {
                        commands.entity(entity).despawn();
                        error!("Error loading mod \"{}\":\n{:?}", name, err)
                    }
                }
            }
            _ => {}
        }
    }
}
