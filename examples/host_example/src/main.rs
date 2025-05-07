use bevy::prelude::*;
use bevy::{DefaultPlugins, app::App, asset::AssetPlugin};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use wasmtime::component::Component;
use wasmtime::{Engine, Store};
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView, add_only_http_to_linker_async};
use wasvy::asset::WasmComponentAsset;
use wasvy::plugin::WasvyHostPlugin;
use wasvy::runner::WasmRunState;

pub struct Stat {
    table: ResourceTable,
    ctx: WasiCtx,
    http: WasiHttpCtx,
}

impl IoView for Stat {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasiView for Stat {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

impl WasiHttpView for Stat {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
}

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
    let simple_handle = asset_server.load::<WasmComponentAsset>("simple.wasm");

    // It takes a few seconds for the python WASM to load.
    // If there are no errors at runtime it means it's working, just give it 10-20 seconds.
    let python_handle = asset_server.load::<WasmComponentAsset>("python.wasm");

    commands.insert_resource(WasmAssets {
        assests: vec![simple_handle, python_handle],
    });
}

fn run_blah() {
    let engine = Engine::default();
    let mut runner = wasvy::runner::Runner::new(engine.clone());
    runner.add_wasi_sync();

    runner.add_functionality(|linker| {
        add_only_http_to_linker_async(linker);
    });

    let mut results = vec![wasmtime::component::Val::String("".to_string())];
    let component = Component::from_file(
                &engine,
                "/home/engodev/projects/wasm/wasvy/examples/host_example/assets/net.wasm",
            )
            .unwrap();
    loop {
        println!("Store start");
        let store = Store::new(
            &engine,
            Stat {
                table: ResourceTable::new(),
                ctx: WasiCtxBuilder::new()
                    .inherit_stdio()
                    .inherit_network()
                    .allow_ip_name_lookup(true)
                    .build(),
                http: WasiHttpCtx::new(),
            },
        );
        println!("Store end");
        runner.run_function(WasmRunState {
            component: &component,
            function_name: "run".to_string(),
            store,
            params: &[wasmtime::component::Val::String(
                "https://wtfismyip.com/json".to_string(),
            )],
            results: &mut results,
        });
        println!("Results: {:?}", results);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn main() {
    let mut app = App::new();
    run_blah();
    return;

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
