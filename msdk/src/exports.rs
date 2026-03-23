//! FFI exports for memlink module C ABI, providing memlink_init, memlink_call, and memlink_shutdown.

use std::sync::Mutex;
use std::time::Instant;

use crate::arena::Arena;
use crate::context::CallContext;
use crate::dispatch;
use crate::panic::catch_module_panic;
use crate::request::Response;

pub const INIT_SUCCESS: i32 = 0;
pub const INIT_FAILURE: i32 = -1;
pub const CALL_SUCCESS: i32 = 0;
pub const CALL_FAILURE: i32 = -1;
pub const CALL_BUFFER_TOO_SMALL: i32 = -2;

static MODULE_ARENA: Mutex<Option<Arena>> = Mutex::new(None);

pub fn get_arena() -> std::sync::MutexGuard<'static, Option<Arena>> {
    MODULE_ARENA.lock().unwrap()
}

pub unsafe fn init_arena(base: *mut u8, capacity: usize) -> bool {
    let mut arena_guard = MODULE_ARENA.lock().unwrap();
    if arena_guard.is_some() {
        return false;
    }
    *arena_guard = Some(Arena::new(base, capacity));
    true
}

pub fn reset_arena() {
    if let Some(arena) = MODULE_ARENA.lock().unwrap().as_ref() {
        arena.reset();
    }
}

#[no_mangle]
pub unsafe extern "C" fn memlink_init(config_ptr: *const u8, config_len: usize) -> i32 {
    let _ = config_ptr;
    let _ = config_len;

    INIT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn memlink_call(
    method_hash: u32,
    args_ptr: *const u8,
    args_len: usize,
    out_ptr: *mut u8,
    out_cap: usize,
) -> i32 {
    if args_len > 0 && args_ptr.is_null() {
        return CALL_FAILURE;
    }

    if out_cap > 0 && out_ptr.is_null() {
        return CALL_FAILURE;
    }

    reset_arena();

    let result = catch_module_panic(|| {
        let args = if args_len > 0 {
            std::slice::from_raw_parts(args_ptr, args_len).to_vec()
        } else {
            vec![]
        };

        let arena_guard = get_arena();
        let arena = match arena_guard.as_ref() {
            Some(a) => a,
            None => return CALL_FAILURE,
        };

        let trace_id = 0u128;
        let span_id = 0u64;
        let backpressure = 0.0f32;
        let deadline: Option<Instant> = None;

        let ctx = CallContext::new(arena, backpressure, trace_id, span_id, deadline, None);

        let result_data = dispatch::dispatch_with_context(&ctx, method_hash, &args);

        let response = match result_data {
            Ok(data) => Response::success(data),
            Err(_) => return CALL_FAILURE,
        };

        let response_bytes = match response.to_bytes() {
            Ok(bytes) => bytes,
            Err(_) => return CALL_FAILURE,
        };

        if response_bytes.len() > out_cap {
            return CALL_BUFFER_TOO_SMALL;
        }

        std::ptr::copy_nonoverlapping(
            response_bytes.as_ptr(),
            out_ptr,
            response_bytes.len(),
        );

        CALL_SUCCESS
    });

    match result {
        Ok(code) => code,
        Err(_) => CALL_FAILURE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn memlink_shutdown() {}
