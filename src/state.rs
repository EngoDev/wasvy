use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::ecs::schedule::Schedules;
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::send_sync_ptr::SendSyncPtr;

pub(crate) struct HostState {
    /// The lifetime of a [`wasmtime::Store`] is bound to a 'static lifetime, which is problematic for us
    /// since we want to pass references to system params which have shorter lifetimes.
    ///
    /// Thus we use this Mutex and a container to hold pointers.
    ///
    /// A guard holds access and removes the references held in inner when they go out of scope
    inner: Arc<Mutex<Inner>>,
    table: ResourceTable,
    ctx: WasiCtx,
}

impl HostState {
    pub(crate) fn new() -> Self {
        let table = ResourceTable::new();
        let ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_network()
            .allow_ip_name_lookup(true)
            .build();

        Self {
            inner: Arc::new(Mutex::new(Inner::Uninitialized)),
            table,
            ctx,
        }
    }

    pub(crate) fn scope<'s>(&self, scope: Scope<'s>) -> ScopeGuard<'s> {
        let mut inner = self.inner.lock().unwrap();
        assert!(
            matches!(*inner, Inner::Uninitialized),
            "State is already scoped. The ScopeGuard must be dropped first."
        );

        *inner = match scope {
            Scope::Setup { schedules } => Inner::Setup {
                schedules: SendSyncPtr::new(schedules.into()),
                app: None,
            },
            Scope::RunSystem => Inner::RunSystem,
        };

        ScopeGuard {
            inner: Arc::clone(&self.inner),
            referencing: PhantomData,
        }
    }

    pub(crate) fn access<F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(State<'_>) -> R,
    {
        let table = &mut self.table;
        let mut inner = self.inner.lock().unwrap();
        let state = match &mut *inner {
            Inner::Setup { schedules, app } => State::Setup {
                // Safety: Always contains a reference to an initialized value, and borrow_mut ensures this is the only borrow
                schedules: unsafe { schedules.as_mut() },
                app,
                table,
            },
            Inner::RunSystem => State::RunSystem,
            Inner::Uninitialized => panic!("Attempting to get state from unscoped HostState"),
        };
        f(state)
    }
}

pub(crate) enum State<'s> {
    Setup {
        schedules: &'s mut Schedules,
        table: &'s mut ResourceTable,
        app: &'s mut Option<u32>,
    },
    RunSystem,
}

enum Inner {
    Uninitialized,
    Setup {
        schedules: SendSyncPtr<Schedules>,
        app: Option<u32>,
    },
    RunSystem,
}

pub(crate) enum Scope<'s> {
    Setup { schedules: &'s mut Schedules },
    RunSystem,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

/// A guard that ensures that the lifetimes of pointers stored in [`Inner`] are respected
pub struct ScopeGuard<'a> {
    inner: Arc<Mutex<Inner>>,
    referencing: PhantomData<&'a ()>,
}

impl Drop for ScopeGuard<'_> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        *inner = Inner::Uninitialized;
    }
}
