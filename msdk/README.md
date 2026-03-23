# memlink-msdk

<table>
<tr>
  <td><a href="https://crates.io/crates/memlink-msdk"><img src="https://img.shields.io/crates/v/memlink-msdk.svg" alt="Crates.io"/></a></td>
  <td><a href="https://docs.rs/memlink-msdk"><img src="https://docs.rs/memlink-msdk/badge.svg" alt="Docs"/></a></td>
  <td><a href="../LICENSE-APACHE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License"/></a></td>
  <td><a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.70%2B-orange.svg" alt="Rust"/></a></td>
</tr>
</table>

SDK for building memlink modules with automatic serialization, FFI exports, panic isolation, and arena-based memory management.

## Features

- **`#[memlink_export]` Macro**: Automatic wrapper generation for module methods
- **Automatic Serialization**: MessagePack (rmp-serde) for efficient IPC
- **Panic Isolation**: Built-in panic catching at FFI boundary
- **Arena Allocation**: Bump-pointer arena for bounded temporary allocations
- **Nested Calls**: Module-to-module communication support
- **Backpressure Control**: Flow control signals from daemon
- **Deadline Tracking**: Timeout and deadline support for calls
- **Structured Logging**: Log export to daemon for aggregation
- **Metrics API**: Counter, gauge, and histogram support

Note: `memlink-msdk-macros` is automatically included as a dependency.

## Quick Start

### Basic Function Export

```rust
use memlink_msdk::prelude::*;

#[memlink_export]
pub fn echo(_ctx: &CallContext, input: String) -> Result<String> {
    Ok(input)
}

#[memlink_export]
pub fn add(_ctx: &CallContext, a: u32, b: u32) -> Result<u32> {
    Ok(a + b)
}
```

### Async Functions

```rust
#[memlink_export]
pub async fn async_process(_ctx: &CallContext, data: Vec<u8>) -> Result<Vec<u8>> {
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    Ok(data)
}
```

### Arena Allocation

```rust
#[memlink_export]
pub fn use_arena(ctx: &CallContext) -> Result<u64> {
    let slot = ctx.arena().alloc::<u64>().ok_or(ModuleError::QuotaExceeded)?;
    unsafe { std::ptr::write(slot, 42); }
    Ok(*slot)
}
```

### Custom Data Structures

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserData {
    pub id: u32,
    pub name: String,
    pub score: u64,
}

#[memlink_export]
pub fn process_user(_ctx: &CallContext, user: UserData) -> Result<UserData> {
    Ok(UserData {
        id: user.id,
        name: user.name.to_uppercase(),
        score: user.score * 2,
    })
}
```

### Nested Module Calls

```rust
#[memlink_export]
pub async fn call_other(ctx: &CallContext, module: String) -> Result<Vec<u8>> {
    let caller = ctx.module_caller().ok_or(ModuleError::ServiceUnavailable)?;
    caller.call(&module, "method", b"args").await
}
```

### Backpressure and Deadlines

```rust
#[memlink_export]
pub fn check_pressure(ctx: &CallContext) -> Result<f32> {
    Ok(ctx.backpressure())
}

#[memlink_export]
pub fn check_deadline(ctx: &CallContext) -> Result<Option<u64>> {
    Ok(ctx.remaining_time().map(|d| d.as_millis() as u64))
}
```

### Logging and Metrics

```rust
#[memlink_export]
pub fn log_and_measure(_ctx: &CallContext) -> Result<()> {
    info("Processing request", &[("user", "alice")]);
    
    increment_counter("requests_total", 1);
    observe_histogram("request_latency", 42.0);
    
    Ok(())
}
```

## Module Structure

| Module | Description |
|--------|-------------|
| `arena` | Bump-pointer arena allocator for temporary allocations |
| `caller` | Module caller for nested invocations |
| `context` | Call context with execution environment |
| `dispatch` | Method dispatch table |
| `error` | Error types (ModuleError, Result) |
| `exports` | FFI exports (memlink_init, memlink_call) |
| `log` | Structured logging API |
| `macros` | Proc macro re-exports |
| `metrics` | Metrics recording API |
| `panic` | Panic isolation utilities |
| `ref` | Persistent arena references |
| `request` | Request/Response types |
| `serialize` | Serialization trait and BincodeSerializer |

## Testing

Test your exported functions using the SDK's test utilities:

```rust
#[cfg(test)]
mod tests {
    use memlink_msdk::prelude::*;
    use crate::echo;

    fn create_test_context() -> CallContext<'static> {
        let arena = Box::leak(Box::new(unsafe {
            let buf = vec![0u8; 8192].into_boxed_slice();
            let ptr = Box::into_raw(buf) as *mut u8;
            Arena::new(ptr, 8192)
        }));
        CallContext::new(arena, 0.0, 0, 0, None, None)
    }

    #[test]
    fn test_echo() {
        let ctx = create_test_context();
        let result = echo(&ctx, "hello".to_string()).unwrap();
        assert_eq!(result, "hello");
    }
}
```

## Function Requirements

Functions annotated with `#[memlink_export]` must:

1. **First Parameter**: `&CallContext` or `CallContext`
2. **Return Type**: `Result<T>` where `T: Serialize + DeserializeOwned`
3. **Sync or Async**: Both are supported

## Error Handling

The SDK provides comprehensive error handling:

```rust
use memlink_msdk::prelude::*;

#[memlink_export]
pub fn may_fail(_ctx: &CallContext, value: u32) -> Result<u32> {
    if value > 100 {
        return Err(ModuleError::QuotaExceeded);
    }
    Ok(value * 2)
}
```

### Error Types

| Error | Description |
|-------|-------------|
| `QuotaExceeded` | Arena or resource quota exceeded |
| `CallFailed(String)` | Nested call failed |
| `Timeout(Duration)` | Operation timed out |
| `Panic(String)` | Module panicked |
| `Serialize(String)` | Serialization error |
| `ServiceUnavailable` | Service not available |
| `InvalidMethod` | Unknown method hash |
| `ModuleNotFound(String)` | Target module not found |
| `MaxCallDepthExceeded` | Nested call depth limit reached |

## Performance

Benchmark results from `cargo bench -p memlink-msdk`:

### Function Call Overhead

| Benchmark | Time | Throughput |
|-----------|------|------------|
| `echo_empty` | 1.14 ns | 874.6 M calls/sec |
| `echo_small` ("hello") | 98.8 ns | 10.1 M calls/sec |

### Arena Allocation

| Benchmark | Time | Throughput |
|-----------|------|------------|
| `alloc_u64` | 2.41 ns | 414.8 M allocs/sec |
| `alloc_with` (u64) | 3.31 ns | 301.8 M allocs/sec |

*Note: Benchmarks run on Windows x86_64. Actual performance varies by system configuration and payload size. Run `cargo bench` for measurements on your system.*

## Examples

See the [examples](examples/) directory for complete working examples:

- `basic.rs` - Basic function exports
- `async_echo.rs` - Async function handling
- `arena_usage.rs` - Arena allocation patterns
- `nested_calls.rs` - Module-to-module communication

## Integration with memlink-runtime

Modules built with memlink-msdk can be loaded dynamically:

```rust
use memlink_runtime::runtime::{Runtime, ModuleRuntime};
use memlink_runtime::resolver::ModuleRef;

let runtime = Runtime::with_local_resolver();
let handle = runtime.load(ModuleRef::parse("./my_module.so")?)?;
let result = runtime.call(handle, "echo", b"hello")?;
```

## Troubleshooting

### Common Issues

**"First parameter must be &CallContext"**
```rust
// Wrong
#[memlink_export]
pub fn bad(input: String) -> Result<String> { Ok(input) }

// Correct
#[memlink_export]
pub fn good(ctx: &CallContext, input: String) -> Result<String> { Ok(input) }
```

**"Return type must be Result<T>"**
```rust
// Wrong
#[memlink_export]
pub fn bad(ctx: &CallContext) -> String { "hello".to_string() }

// Correct
#[memlink_export]
pub fn good(ctx: &CallContext) -> Result<String> { Ok("hello".to_string()) }
```

## License

Licensed under the Apache License 2.0. See [LICENSE-APACHE](../LICENSE-APACHE) for details.

## Contributing

Contributions are welcome! Please submit issues and pull requests to the [main repository](https://github.com/Radiiplus/memlink).
