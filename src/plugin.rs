use crate::{
    asset::{ModAsset, ModAssetLoader},
    component_registry::WasmComponentRegistry,
    engine::Engine,
    systems::run_setup,
};
use bevy::prelude::*;

/// This plugin adds Wasvy modding support to [`App`]
///
/// ```rust
///  App::new()
///    .add_plugins(DefaultPlugins)
///    .add_plugins(ModloaderPlugin)
///    // etc
/// ```
///
/// Looking for next steps? See: [`Mods`](crate::mods::Mods)
/// ```
pub struct ModloaderPlugin;

impl Plugin for ModloaderPlugin {
    fn build(&self, app: &mut App) {
        let engine = Engine::new();

        app.init_asset::<ModAsset>()
            .register_asset_loader(ModAssetLoader::new(&engine));

        app.insert_resource(engine)
            .init_resource::<WasmComponentRegistry>();

        app.add_systems(PreUpdate, run_setup);

        let asset_plugins = app.get_added_plugins::<AssetPlugin>();
        let asset_plugin = asset_plugins
            .get(0)
            .expect("ModloaderPlugin requires AssetPlugin to be loaded.");

        // Warn a user running the App in debug; they probably want hot-reloading
        if cfg!(debug_assertions) {
            let user_overrode_watch_setting = asset_plugin.watch_for_changes_override.is_some();
            let resolved_watch_setting = app
                .world()
                .get_resource::<AssetServer>()
                .unwrap()
                .watching_for_changes();

            if !user_overrode_watch_setting && !resolved_watch_setting {
                warn!(
                    "Enable Bevy's watch feature to enable hot-reloading Wasvy mods.\
                You can do this by running the command `cargo run --features bevy/file_watcher`.\
                In order to hide this message, set the `watch_for_changes_override` to\
                `Some(true)` or `Some(false)` in the AssetPlugin."
                );
            }
        }
    }
}
