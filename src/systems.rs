use crate::{
    asset::ModAsset,
    engine::Engine,
    state::{Scope, new_store},
};
use bevy::prelude::*;

pub(crate) fn run_setup(
    mut events: MessageReader<AssetEvent<ModAsset>>,
    assets: Res<Assets<ModAsset>>,
    mut schedules: ResMut<Schedules>,
    engine: Res<Engine>,
) {
    let mut updated = Vec::new();
    for event in events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                updated.push(id);
            }
            _ => {}
        }
    }

    if updated.is_empty() {
        return;
    }

    let mut store = new_store(&engine);
    let _guard = store.data().scope(Scope::Setup {
        schedules: &mut schedules,
    });

    for id in updated {
        let asset = assets.get(*id).unwrap();
        asset.setup(&mut store).unwrap();
    }
}
