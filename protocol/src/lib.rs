#[allow(warnings)]
mod bindings;

// use std::collections::HashMap;

use std::time::Duration;

use bindings::{
    Guest,
    component::protocol::{
        host_ecs,
        types::{Component, ComponentId, Query},
    },
};
use serde::{Deserialize, Serialize};

struct GuestComponent {
    // components: HashMap<String, ComponentId>,
}

pub trait Test {}

impl Test for Query {}

#[derive(Debug, Serialize, Deserialize)]
pub struct FirstComponent {
    pub first: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SecondComponent {
    pub second: usize,
}

impl Guest for GuestComponent {
    /// Say hello!
    fn hello_world() -> String {
        "Hello, World!".to_string()
    }

    fn sum(params: Vec<Vec<bindings::QueryData>>) {
        let first_component_query = params.first().unwrap();
        for row in first_component_query {
            let entity = row.entity;
            println!("Entity: {:?}", entity);
            let component = row.components.first().unwrap();
            let first_component: FirstComponent = serde_json::from_str(&component.value).unwrap();
            println!("Component: {:?}", first_component);
        }

        // fetch_my_ip();
    }

    fn setup() {
        let id1 = host_ecs::register_component("FirstComponent");
        let id2 = host_ecs::register_component("SecondComponent");

        host_ecs::register_system(
            "sum",
            &[Query {
                components: vec![id1],
            }],
        );

        let serialized = serde_json::to_string(&FirstComponent { first: 12 }).unwrap();
        host_ecs::spawn(&[Component {
            id: id1,
            value: serialized,
        }]);
    }
}

pub fn fetch_my_ip() {
    let request = ehttp::Request::get("https://icanhazip.com");
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        println!("Body: {}", result.unwrap().text().unwrap());
    });
    // let body = reqwest::blocking::get("https://icanhazip.com")
    //     .unwrap()
    //     .text()
    //     .unwrap();
    //
    // body.to_string()
    // let client = reqwest::blocking::Client::builder()
    //     .timeout(Duration::from_secs(5))
    //     .build()
    //     .unwrap();
    //
    // let body = client
    //     .get("https://icanhazip.com")
    //     .send()
    //     .unwrap()
    //     .text()
    //     .unwrap();
    //
    // // Trim trailing newline and return
    // body.trim().to_string()
}

bindings::export!(GuestComponent with_types_in bindings);
