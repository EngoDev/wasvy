#[allow(warnings)]
mod bindings;

use bindings::{
    Guest,
    wasvy::{
        self,
        ecs::types::{Component, Query},
    },
};
use serde::{Deserialize, Serialize};

use bevy::{prelude::*, reflect::Type};

struct GuestComponent;

#[derive(Debug, Reflect, Serialize, Deserialize)]
pub struct FirstComponent {
    pub first: usize,
}

#[derive(Debug, Reflect, Serialize, Deserialize)]
pub struct SecondComponent {
    pub second: usize,
}

impl Guest for GuestComponent {
    fn hello_world() -> String {
        "Hello, World!".to_string()
    }

    /// The params in this instance will be equal to: `[Query<&FirstComponent>]`
    /// due to how the system was registered in `setup`.
    ///
    /// If for example the system was registered with
    /// `components: [simple::FirstComponent, simple::SecondComponent]`
    /// then params would be equal to `[Query<(&FirstComponent, &SecondComponent)>]`
    fn print_first_component_system(params: Vec<bindings::QueryResult>, blah: u64) {
        let first_component_query = params.first().unwrap();
        for row in first_component_query {
            let entity = row.entity;
            println!("Entity: {:?}", entity);
            let component = row.components.first().unwrap();
            let first_component: FirstComponent = serde_json::from_str(&component.get()).unwrap();
            println!("Component: {:?}", first_component);
        }
    }

    fn two_components_in_a_query(params: Vec<bindings::QueryResult>, blah: u64) {
        let query = params.first().unwrap();
        for row in query {
            let second_component_serialized = row.components.first().unwrap();
            let second_component: SecondComponent =
                serde_json::from_str(&second_component_serialized.get()).unwrap();

            let transform_component_serialized = &row.components[1];
            let mut transform_component: Transform =
                serde_json::from_str(&transform_component_serialized.get()).unwrap();

            transform_component.translation.x += 1.0;

            transform_component_serialized
                .set(&serde_json::to_string(&transform_component).unwrap());

            println!(
                "Second Component: {:?}, Transform: {:?}",
                second_component, transform_component
            );
        }
    }

    fn setup() {
        let first_component_type_path = Type::of::<FirstComponent>().path();
        let second_component_type_path = Type::of::<SecondComponent>().path();
        let transform_type_path = Type::of::<Transform>().path();

        let _id1 = wasvy::ecs::functions::register_component(first_component_type_path);
        let _id2 = wasvy::ecs::functions::register_component(second_component_type_path);

        wasvy::ecs::functions::register_system(
            "print-first-component-system",
            &[Query {
                components: vec![first_component_type_path.to_string()],
                with: vec![],
                without: vec![],
            }],
        );

        wasvy::ecs::functions::register_system(
            "two-components-in-a-query",
            &[Query {
                components: vec![
                    second_component_type_path.to_string(),
                    transform_type_path.to_string(),
                ],
                with: vec![],
                without: vec![],
            }],
        );

        let first_serialized = serde_json::to_string(&FirstComponent { first: 18 }).unwrap();
        let second_serialized = serde_json::to_string(&SecondComponent { second: 18 }).unwrap();
        let transform_serialized = serde_json::to_string(
            &Transform::default().with_translation(Vec3::new(10.0, 20.0, 30.0)),
        )
        .unwrap();

        wasvy::ecs::functions::spawn(vec![Component::new(
            &first_serialized,
            first_component_type_path,
        )]);

        wasvy::ecs::functions::spawn(vec![
            Component::new(&second_serialized, second_component_type_path),
            Component::new(&transform_serialized, transform_type_path),
        ]);

        println!("Finished setup!");
    }
}

bindings::export!(GuestComponent with_types_in bindings);
