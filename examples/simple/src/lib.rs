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

use bevy::{
    prelude::*,
    reflect::{
        Type,
        serde::{ReflectDeserializer, ReflectSerializer},
    },
};

struct GuestComponent;

#[derive(Debug, Reflect, Serialize, Deserialize)]
pub struct FirstComponent {
    pub first: usize,
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct SecondComponent {
    pub second: usize,
}

impl Guest for GuestComponent {
    fn hello_world() -> String {
        "Hello, World!".to_string()
    }

    /// The params in this instance will be equal to: `[Query<(&FirstComponent, &Transform)>]`
    /// due to how the system was registered in `setup`.
    ///
    /// If for example the system was registered with `components: [id1, id2]`
    /// then params would be equal to `[Query<(&FirstComponent, &Transform)>]`
    fn print_first_component_system(params: Vec<bindings::QueryResult>) {
        let query = params.first().unwrap();
        for row in query {
            let entity = row.entity;
            println!("Row: {:?}", row);
            let first_component_serialized = row.components.first().unwrap();
            let first_component: FirstComponent =
                serde_json::from_str(&first_component_serialized.value).unwrap();

            let transform_component_serialized = &row.components[1];
            let transform_component: Transform =
                serde_json::from_str(&transform_component_serialized.value).unwrap();

            println!(
                "First Component: {:?}, Transform: {:?}",
                first_component, transform_component
            );
        }
    }

    fn setup() {
        println!("asdfasd");
        let first_component_type_path = Type::of::<FirstComponent>().path();
        let second_component_type_path = Type::of::<SecondComponent>().path();
        let transform_type_path = Type::of::<Transform>().path();

        let id1 = wasvy::ecs::functions::register_component(first_component_type_path);
        let _id2 = wasvy::ecs::functions::register_component(second_component_type_path);
        let transform_id = wasvy::ecs::functions::get_component_id(transform_type_path).unwrap();

        wasvy::ecs::functions::register_system(
            "print-first-component-system",
            &[Query {
                components: vec![
                    first_component_type_path.to_string(),
                    transform_type_path.to_string(),
                ],
                with: vec![],
                without: vec![],
            }],
        );

        let serialized = serde_json::to_string(&FirstComponent { first: 18 }).unwrap();
        // let transform_serialized = ron::ser::to_string_pretty(
        //     &Transform::default().with_translation(Vec3::new(10.0, 20.0, 30.0)),
        //     PrettyConfig::default().struct_names(true),
        // )
        // .unwrap();
        let transform_serialized = serde_json::to_string(
            &Transform::default().with_translation(Vec3::new(10.0, 20.0, 30.0)),
        )
        .unwrap();
        println!("Serialized transform: {:?}", transform_serialized);

        wasvy::ecs::functions::spawn(&[
            Component {
                path: first_component_type_path.to_string(),
                // id: id1,
                value: serialized,
            },
            Component {
                path: transform_type_path.to_string(),
                value: transform_serialized,
            },
        ]);
    }
}

bindings::export!(GuestComponent with_types_in bindings);
