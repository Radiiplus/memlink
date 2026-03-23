//! Panic isolation utilities for memlink modules, catching panics and converting them to errors.

use std::panic::{catch_unwind, UnwindSafe};
use std::string::ToString;

use crate::error::{ModuleError, Result};

pub fn catch_module_panic<F, R>(f: F) -> Result<R>
where
    F: FnOnce() -> R + UnwindSafe,
{
    match catch_unwind(f) {
        Ok(value) => Ok(value),
        Err(payload) => {
            let panic_msg = if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            Err(ModuleError::Panic(panic_msg))
        }
    }
}
