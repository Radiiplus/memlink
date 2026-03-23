//! Method dispatch table for routing method calls to their handlers.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::context::CallContext;
use crate::error::{ModuleError, Result};

pub type MethodHandler = fn(&CallContext<'_>, &[u8]) -> Result<Vec<u8>>;

static METHOD_TABLE: LazyLock<Mutex<HashMap<u32, MethodHandler>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn register_method(hash: u32, handler: MethodHandler) {
    let mut table = METHOD_TABLE.lock().unwrap();
    table.insert(hash, handler);
}

pub fn dispatch_with_context(ctx: &CallContext<'_>, method_hash: u32, args: &[u8]) -> Result<Vec<u8>> {
    let table = METHOD_TABLE.lock().unwrap();

    match table.get(&method_hash) {
        Some(handler) => handler(ctx, args),
        None => Err(ModuleError::InvalidMethod),
    }
}

pub fn dispatch(method_hash: u32, args: &[u8]) -> Result<Vec<u8>> {
    use crate::exports::get_arena;

    let arena_guard = get_arena();
    if let Some(arena) = arena_guard.as_ref() {
        let ctx = CallContext::new(arena, 0.0, 0, 0, None, None);
        dispatch_with_context(&ctx, method_hash, args)
    } else {
        Err(ModuleError::ServiceUnavailable)
    }
}

pub fn unregister_method(hash: u32) -> bool {
    let mut table = METHOD_TABLE.lock().unwrap();
    table.remove(&hash).is_some()
}

pub fn method_count() -> usize {
    let table = METHOD_TABLE.lock().unwrap();
    table.len()
}

pub fn clear_methods() {
    let mut table = METHOD_TABLE.lock().unwrap();
    table.clear();
}
