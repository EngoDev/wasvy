pub(crate) use crate::{
    bindings::wasvy::ecs::app::*,
    state::{State, WasmHost},
};
pub(crate) use anyhow::bail;
pub(crate) use wasmtime::{Result, component::Resource};

mod app;
mod commands;
mod component;
mod query;
mod system;

pub use app::*;
pub use commands::*;
pub use component::*;
pub use query::*;
pub use system::*;

impl Host for WasmHost {}
