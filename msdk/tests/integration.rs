//! Integration tests for memlink SDK.

use memlink_msdk::prelude::*;

fn echo(_ctx: &CallContext<'_>, input: String) -> Result<String> {
    Ok(input)
}

async fn async_echo(_ctx: &CallContext<'_>, input: Vec<u8>) -> Result<Vec<u8>> {
    Ok(input)
}

fn use_arena(ctx: &CallContext<'_>) -> Result<u64> {
    let x = ctx
        .arena()
        .alloc::<u64>()
        .ok_or(ModuleError::QuotaExceeded)?;
    unsafe {
        std::ptr::write(x, 42);
        Ok(*x)
    }
}

fn add(_ctx: &CallContext<'_>, a: u32, b: u32) -> Result<u32> {
    Ok(a + b)
}

fn panic_test(_ctx: &CallContext<'_>) -> Result<()> {
    panic!("intentional panic for testing");
}

fn error_test(_ctx: &CallContext<'_>) -> Result<()> {
    Err(ModuleError::ServiceUnavailable)
}

fn record_metric_test(_ctx: &CallContext<'_>) -> Result<()> {
    record_metric("test_counter", MetricValue::Counter(1));
    Ok(())
}

fn log_test(_ctx: &CallContext<'_>) -> Result<()> {
    info("Test log message", &[("test", "integration")]);
    Ok(())
}

fn backpressure_test(ctx: &CallContext<'_>) -> Result<f32> {
    Ok(ctx.backpressure())
}

fn trace_id_test(ctx: &CallContext<'_>) -> Result<u128> {
    Ok(ctx.trace_id())
}

fn deadline_test(ctx: &CallContext<'_>) -> Result<Option<u64>> {
    Ok(ctx.remaining_time().map(|d| d.as_millis() as u64))
}

fn create_test_context(
    backpressure: f32,
    trace_id: u128,
    span_id: u64,
) -> CallContext<'static> {
    let arena = Box::leak(Box::new(unsafe {
        let buffer = vec![0u8; 8192].into_boxed_slice();
        let ptr = Box::into_raw(buffer) as *mut u8;
        Arena::new(ptr, 8192)
    }));

    CallContext::new(
        arena,
        backpressure,
        trace_id,
        span_id,
        None,
        None,
    )
}

#[test]
fn test_module_compiles() {
    let ctx = create_test_context(0.0, 0, 0);
    let result = add(&ctx, 1, 2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);
}

#[test]
fn test_module_lifecycle() {
    let ctx = create_test_context(0.0, 0, 0);
    assert!(add(&ctx, 10, 20).is_ok());
    assert!(echo(&ctx, "hello".to_string()).is_ok());
    assert!(use_arena(&ctx).is_ok());
}

#[test]
fn test_echo_roundtrip() {
    let ctx = create_test_context(0.0, 0, 0);

    let test_cases = vec![
        "".to_string(),
        "hello".to_string(),
        "hello, world!".to_string(),
        "🦀 Rust".to_string(),
        "a".repeat(1000),
    ];

    for input in test_cases {
        let result = echo(&ctx, input.clone());
        assert!(result.is_ok(), "Echo failed for: {}", input);
        assert_eq!(result.unwrap(), input);
    }
}

#[tokio::test]
async fn test_async_echo() {
    let ctx = create_test_context(0.0, 0, 0);

    let test_cases = vec![
        vec![],
        vec![1, 2, 3],
        vec![0u8; 1024],
    ];

    for input in test_cases {
        let result = async_echo(&ctx, input.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), input);
    }
}

#[test]
fn test_arena_allocation() {
    let ctx = create_test_context(0.0, 0, 0);
    let result = use_arena(&ctx);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_multiple_arena_allocations() {
    let ctx = create_test_context(0.0, 0, 0);

    let a = ctx.arena().alloc::<u32>().unwrap();
    unsafe { std::ptr::write(a, 100); }

    let b = ctx.arena().alloc::<u32>().unwrap();
    unsafe { std::ptr::write(b, 200); }

    let c = ctx.arena().alloc::<u32>().unwrap();
    unsafe { std::ptr::write(c, 300); }

    assert_eq!(*a, 100);
    assert_eq!(*b, 200);
    assert_eq!(*c, 300);
}

#[test]
fn test_arena_reset() {
    let ctx = create_test_context(0.0, 0, 0);

    let _ = ctx.arena().alloc::<[u8; 1000]>().unwrap();
    let used_before = ctx.arena().used();
    assert!(used_before > 0);

    ctx.arena().reset();
    let used_after = ctx.arena().used();
    assert_eq!(used_after, 0);

    let _ = ctx.arena().alloc::<[u8; 1000]>().unwrap();
}

#[test]
fn test_panic_isolation() {
    use memlink_msdk::panic::catch_module_panic;

    let ctx = create_test_context(0.0, 0, 0);

    let result = catch_module_panic(|| {
        panic_test(&ctx)
    });

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ModuleError::Panic(_)));
}

#[test]
fn test_call_after_panic() {
    use memlink_msdk::panic::catch_module_panic;

    let ctx = create_test_context(0.0, 0, 0);

    let _ = catch_module_panic(|| panic_test(&ctx));

    let result = echo(&ctx, "after panic".to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "after panic");
}

#[test]
fn test_error_propagation() {
    let ctx = create_test_context(0.0, 0, 0);

    let result = error_test(&ctx);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ModuleError::ServiceUnavailable));
}

#[test]
fn test_add_function() {
    let ctx = create_test_context(0.0, 0, 0);

    assert_eq!(add(&ctx, 0, 0).unwrap(), 0);
    assert_eq!(add(&ctx, 1, 2).unwrap(), 3);
    assert_eq!(add(&ctx, 100, 200).unwrap(), 300);
    assert_eq!(add(&ctx, u32::MAX, 0).unwrap(), u32::MAX);
}

#[test]
fn test_backpressure_access() {
    let ctx = create_test_context(0.75, 0, 0);

    let result = backpressure_test(&ctx);
    assert!(result.is_ok());
    assert!((result.unwrap() - 0.75).abs() < 0.001);
}

#[test]
fn test_trace_id_propagation() {
    let ctx = create_test_context(0.0, 123456789u128, 0);

    let result = trace_id_test(&ctx);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 123456789u128);
}

#[test]
fn test_deadline_tracking() {
    use std::time::{Duration, Instant};

    let arena = Box::leak(Box::new(unsafe {
        let buffer = vec![0u8; 8192].into_boxed_slice();
        let ptr = Box::into_raw(buffer) as *mut u8;
        Arena::new(ptr, 8192)
    }));

    let deadline = Instant::now() + Duration::from_millis(500);
    let ctx = CallContext::new(arena, 0.0, 0, 0, Some(deadline), None);

    let result = deadline_test(&ctx);
    assert!(result.is_ok());
    let remaining = result.unwrap().unwrap();
    assert!(remaining <= 500);
    assert!(remaining > 0);
}

#[test]
fn test_no_deadline() {
    let ctx = create_test_context(0.0, 0, 0);

    let result = deadline_test(&ctx);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_metrics_recording() {
    let ctx = create_test_context(0.0, 0, 0);

    let result = record_metric_test(&ctx);
    assert!(result.is_ok());
}

#[test]
fn test_logging() {
    let ctx = create_test_context(0.0, 0, 0);

    let result = log_test(&ctx);
    assert!(result.is_ok());
}

#[test]
fn test_serialization_roundtrip() {
    use memlink_msdk::serialize::{default_serializer, Serializer};

    let serializer = default_serializer();

    let input = "hello, world!".to_string();
    let bytes = serializer.serialize(&input).unwrap();
    let output: String = serializer.deserialize(&bytes).unwrap();
    assert_eq!(input, output);

    let input = vec![1u8, 2, 3, 4, 5];
    let bytes = serializer.serialize(&input).unwrap();
    let output: Vec<u8> = serializer.deserialize(&bytes).unwrap();
    assert_eq!(input, output);

    let input = 42u32;
    let bytes = serializer.serialize(&input).unwrap();
    let output: u32 = serializer.deserialize(&bytes).unwrap();
    assert_eq!(input, output);

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestData {
        name: String,
        value: u32,
        items: Vec<u8>,
    }

    let input = TestData {
        name: "test".to_string(),
        value: 100,
        items: vec![1, 2, 3],
    };
    let bytes = serializer.serialize(&input).unwrap();
    let output: TestData = serializer.deserialize(&bytes).unwrap();
    assert_eq!(input, output);
}

#[test]
fn test_dispatch_registration() {
    use memlink_msdk::dispatch::{clear_methods, dispatch_with_context, register_method};

    clear_methods();

    fn test_handler(_ctx: &CallContext<'_>, args: &[u8]) -> Result<Vec<u8>> {
        Ok(args.to_vec())
    }

    register_method(0x12345678, test_handler);

    let ctx = create_test_context(0.0, 0, 0);
    let result = dispatch_with_context(&ctx, 0x12345678, &[1, 2, 3]);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_dispatch_unknown_method() {
    use memlink_msdk::dispatch::{clear_methods, dispatch_with_context};

    clear_methods();

    let ctx = create_test_context(0.0, 0, 0);
    let result = dispatch_with_context(&ctx, 0xFFFFFFFF, &[]);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ModuleError::InvalidMethod));
}

#[test]
fn test_nested_call_structure() {
    use memlink_msdk::caller::ModuleCaller;

    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let caller = ModuleCaller::new(tx, 0, 12345u128, 67890u64);

    assert_eq!(caller.depth(), 0);
    assert_eq!(caller.trace_id(), 12345u128);

    let nested = caller.for_nested_call(11111u64);
    assert_eq!(nested.depth(), 1);
    assert_eq!(nested.trace_id(), 12345u128);
    assert_eq!(nested.span_id(), 11111u64);
}

#[tokio::test]
async fn test_max_call_depth() {
    use memlink_msdk::caller::{ModuleCaller, MAX_CALL_DEPTH};

    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let caller = ModuleCaller::new(tx, MAX_CALL_DEPTH, 0, 0);

    let result = caller.call("test", "method", &[]).await;
    assert!(matches!(result, Err(ModuleError::MaxCallDepthExceeded)));
}

#[test]
fn test_request_serialization() {
    use memlink_msdk::request::Request;

    let request = Request::new(0x12345678, vec![1, 2, 3])
        .with_trace_id(12345u128)
        .with_deadline(1000000000);

    let bytes = request.to_bytes().unwrap();
    let recovered = Request::from_bytes(&bytes).unwrap();

    assert_eq!(request.method_hash, recovered.method_hash);
    assert_eq!(request.args, recovered.args);
    assert_eq!(request.trace_id, recovered.trace_id);
    assert_eq!(request.deadline_ns, recovered.deadline_ns);
}

#[test]
fn test_response_serialization() {
    use memlink_msdk::request::Response;

    let response = Response::success(vec![1, 2, 3]);
    let bytes = response.to_bytes().unwrap();
    let recovered = Response::from_bytes(&bytes).unwrap();
    assert_eq!(response, recovered);

    let response = Response::error(-1);
    let bytes = response.to_bytes().unwrap();
    let recovered = Response::from_bytes(&bytes).unwrap();
    assert_eq!(response, recovered);
}
