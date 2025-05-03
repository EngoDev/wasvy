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

struct GuestComponent;

#[derive(Debug, Serialize, Deserialize)]
pub struct FirstComponent {
    pub first: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SecondComponent {
    pub second: usize,
}

impl Guest for GuestComponent {
    fn hello_world() -> String {
        "Hello, World!".to_string()
    }

    /// The params in this instance will be equal to: `[Query<FirstComponent>]`
    /// due to how the system was registered in `setup`.
    ///
    /// If for example the system was registered with `components: [id1, id2]`
    /// then params would be equal to `[Query<FirstComponent>, Query<SecondComponent>]`
    fn print_first_component_system(params: Vec<bindings::QueryResult>) {
        let first_component_query = params.first().unwrap();
        for row in first_component_query {
            let entity = row.entity;
            println!("Entity: {:?}", entity);
            let component = row.components.first().unwrap();
            let first_component: FirstComponent = serde_json::from_str(&component.value).unwrap();
            println!("Component: {:?}", first_component);
        }
    }

    fn setup() {
        let id1 = wasvy::ecs::functions::register_component("FirstComponent");
        let _id2 = wasvy::ecs::functions::register_component("SecondComponent");

        wasvy::ecs::functions::register_system(
            "print-first-component-system",
            &[Query {
                components: vec![id1],
                with: vec![],
                without: vec![],
            }],
        );

        let serialized = serde_json::to_string(&FirstComponent { first: 18 }).unwrap();
        wasvy::ecs::functions::spawn(&[Component {
            id: id1,
            value: serialized,
        }]);
    }
}

bindings::export!(GuestComponent with_types_in bindings);
