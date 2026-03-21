//! Module symbols - type-safe function pointers for module exports.

use libloading::Symbol;
use std::marker::PhantomData;
use std::sync::Arc;

pub type ModuleInitFn = unsafe extern "C" fn(config: *const u8, config_len: usize) -> i32;

pub type ModuleCallFn =
    unsafe extern "C" fn(method_id: u32, args: *const u8, args_len: usize, output: *mut u8) -> i32;

pub type ModuleShutdownFn = unsafe extern "C" fn() -> i32;

#[derive(Clone)]
pub struct LibHandle {
    inner: Arc<libloading::Library>,
}

impl LibHandle {
    pub(crate) fn new(inner: libloading::Library) -> Self {
        LibHandle {
            inner: Arc::new(inner),
        }
    }

    pub(crate) fn inner(&self) -> &libloading::Library {
        &self.inner
    }
}

pub struct ModuleSymbols<'lib> {
    pub memlink_init: Symbol<'lib, ModuleInitFn>,
    pub memlink_call: Symbol<'lib, ModuleCallFn>,
    pub memlink_shutdown: Symbol<'lib, ModuleShutdownFn>,
    _marker: PhantomData<&'lib LibHandle>,
}

impl<'lib> ModuleSymbols<'lib> {
    pub fn new(
        memlink_init: Symbol<'lib, ModuleInitFn>,
        memlink_call: Symbol<'lib, ModuleCallFn>,
        memlink_shutdown: Symbol<'lib, ModuleShutdownFn>,
    ) -> Self {
        ModuleSymbols {
            memlink_init,
            memlink_call,
            memlink_shutdown,
            _marker: PhantomData,
        }
    }
}
