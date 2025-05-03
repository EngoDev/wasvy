use bevy::prelude::*;
use bevy::{DefaultPlugins, app::App, asset::AssetPlugin};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use plugin::WasvyHostPlugin;

mod asset;
mod host;
mod plugin;
mod runner;

mod bindings {
    wasmtime::component::bindgen!("host" in "wit/ecs/ecs.wit");
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        watch_for_changes_override: Some(true),
        ..Default::default()
    }));
    app.add_plugins(EguiPlugin {
        enable_multipass_for_primary_context: true,
    });
    app.add_plugins(WorldInspectorPlugin::new());

    app.add_plugins(WasvyHostPlugin);

    app.run();

    Ok(())
}
