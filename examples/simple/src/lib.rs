use bevy_transform::components::Transform;
use wit_bindgen::rt::async_support;

mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/ecs/ecs.wit",
        world: "guest",
    });
}
use bindings::{wasvy::ecs::api::*, *};

struct GuestComponent;

impl Guest for GuestComponent {
    fn setup() {
        // Define a new system that queries for entities with a Transform and a Marker component
        let my_system = SystemBuilder::new("my_system");
        my_system.add_query(&[
            Query::Mut("bevy_transform::components::Transform".to_string()),
            Query::With("host_example::Marker".to_string()),
        ]);
        let (my_system, mut stream) = my_system.build();

        // Register the system to run in the Update schedule
        let m = Mod::new("simple");
        m.add_systems(Schedule::Update, vec![my_system]);

        // Start listening to the event stream
        async_support::spawn(async move {
            loop {
                let input = match stream.next().await {
                    Some(e) => e,
                    None => break,
                };

                let query = input.next().as_query();
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
        });
    }
}

export!(GuestComponent);
