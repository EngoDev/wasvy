use std::{alloc::Layout, borrow::Cow};

use bevy::prelude::*;
use bevy::{
    DefaultPlugins, MinimalPlugins,
    app::App,
    asset::AssetPlugin,
    ecs::{
        component::{Component as BevyComponent, ComponentDescriptor},
        world::{World, WorldId},
    },
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use host::WasmHost;
use host_plugin::HostPlugin;
use serde::{Deserialize, Serialize};
use wasmtime::{
    Engine, Store,
    component::{Component, Func, Linker, Val},
};

use wasmtime::component::bindgen;
use wasmtime_wasi::{self, IoView};

use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

mod asset;
mod host;
mod host_plugin;
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

    app.add_plugins(HostPlugin);

    app.run();

    Ok(())
}
