use bevy::prelude::*;
use bevy::{DefaultPlugins, app::App, asset::AssetPlugin};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use wasvy::asset::WasmComponentAsset;
use wasvy::plugin::WasvyHostPlugin;

/// Bevy drops assets if there are no active handles
/// so this resource exists to keep the handles alive.
#[derive(Resource)]
struct WasmAssets {
    #[allow(dead_code)]
    pub assests: Vec<Handle<WasmComponentAsset>>,
}

struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_wasm_modules);
    }
}

/// Before running the example build either `simple` or `python_example` from the examples folder
/// and put the `.wasm` file in the host_example assets folder.
///
/// You can build either by using `just` (Checkout the `justfile` in the root of the repo)
fn load_wasm_modules(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Remember to modify simple.wasm to the wasm file you copied to the assets folder.
    let handle = asset_server.load::<WasmComponentAsset>("simple.wasm");

    commands.insert_resource(WasmAssets {
        assests: vec![handle],
    });
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        watch_for_changes_override: Some(true),
        ..Default::default()
    }));
    app.add_plugins(EguiPlugin {
        enable_multipass_for_primary_context: true,
    });
    app.add_plugins(WorldInspectorPlugin::new());

    // Adding the [`WasvyHostPlugin`] is all you need ;)
    app.add_plugins(WasvyHostPlugin);

    app.add_plugins(ExamplePlugin);

    app.run();
}
