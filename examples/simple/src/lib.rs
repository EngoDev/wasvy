use bevy_transform::components::Transform;

mod bindings {
    // This could be replaced with wit_bindgen::generate!() in the future,
    // since it can resolve the path and world from Cargo.toml
    // But it currently has issues resolving wasvy:ecs...
    // almost like it ignores [package.metadata.component.target.dependencies]
    wit_bindgen::generate!({
        path: ["../../wit/ecs", "./wit"],
        world: "component:simple/example",
        with: {
            "wasvy:ecs/app": generate,
            "wasvy:ecs/types": generate,
            "wasvy:ecs/system-params": generate,
        }
    });
}
use bindings::{
    wasvy::ecs::{
        app::{App, System},
        system_params::{Commands, QueryResults},
        types::{Query, Schedule},
    },
    *,
};

struct GuestComponent;

impl Guest for GuestComponent {
    fn setup() {
        // Define a new system that queries for entities with a Transform and a Marker component
        let my_system = System::new("my_system");
        my_system.add_query(&[
            Query::Mut("bevy_transform::components::Transform".to_string()),
            Query::With("host_example::Marker".to_string()),
        ]);

        // Register the system to run in the Update schedule
        let app = App::new();
        app.add_systems(Schedule::Update, vec![my_system]);
    }

    fn my_system(_commands: Commands, query: QueryResults) -> () {
        loop {
            let results = match query.iter() {
                Some(e) => e,
                None => break,
            };

            let mut transform: Transform = serde_json::from_str(&results[0].get()).unwrap();

            // Simply rotate the entity a bit on each frame
            transform.rotate_x(2.0);

            results[0].set(&serde_json::to_string(&transform).unwrap());
        }
    }
}

export!(GuestComponent);
