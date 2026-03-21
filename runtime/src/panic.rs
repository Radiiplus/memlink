//! Panic isolation for safe FFI calls.

use std::panic::{self, AssertUnwindSafe, UnwindSafe};
use std::sync::Once;

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct PanicError {
    pub message: String,
}

impl std::fmt::Display for PanicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Module panicked: {}", self.message)
    }
}

impl std::error::Error for PanicError {}

pub fn setup_panic_hook() {
    static SETUP: Once = Once::new();
    SETUP.call_once(|| {
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            eprintln!("[RUNTIME PANIC] {}", format_panic_info(panic_info));
            default_hook(panic_info);
        }));
    });
}

#[allow(clippy::incompatible_msrv)]
fn format_panic_info(panic_info: &panic::PanicHookInfo<'_>) -> String {
    let location = panic_info
        .location()
        .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
        .unwrap_or_else(|| "unknown location".to_string());

    let payload = panic_info
        .payload()
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| panic_info.payload().downcast_ref::<String>().cloned())
        .unwrap_or_else(|| format!("{:?}", panic_info.payload()));

    format!("{} at {}", payload, location)
}

pub fn safe_call<F, R>(f: F) -> Result<R>
where
    F: FnOnce() -> R + UnwindSafe,
{
    match panic::catch_unwind(f) {
        Ok(result) => Ok(result),
        Err(panic_payload) => {
            let message = extract_panic_message(&panic_payload);
            Err(Error::ModulePanicked(message))
        }
    }
}

pub fn safe_call_unchecked<F, R>(f: F) -> Result<R>
where
    F: FnOnce() -> R,
{
    safe_call(AssertUnwindSafe(f))
}

fn extract_panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        format!("{:?}", payload)
    }
}
