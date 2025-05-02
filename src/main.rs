use std::{alloc::Layout, borrow::Cow};

use bevy::{
    DefaultPlugins, MinimalPlugins,
    app::App,
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

mod host;
mod host_plugin;

mod bindings {
    wasmtime::component::bindgen!("ecs" in "./protocol/wit/world.wit");
    // wasmtime::component::bindgen!({
    //     paths: "./protocol/wit/world.wit",
    //     world: "ecs",
    //     async: false
    // })
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(EguiPlugin {
        enable_multipass_for_primary_context: true,
    });
    app.add_plugins(WorldInspectorPlugin::new());

    app.add_plugins(HostPlugin);

    app.run();

    Ok(())
}
