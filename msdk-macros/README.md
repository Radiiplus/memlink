# memlink-msdk-macros

<table>
<tr>
  <td><a href="https://crates.io/crates/memlink-msdk-macros"><img src="https://img.shields.io/crates/v/memlink-msdk-macros.svg" alt="Crates.io"/></a></td>
  <td><a href="https://docs.rs/memlink-msdk-macros"><img src="https://docs.rs/memlink-msdk-macros/badge.svg" alt="Docs"/></a></td>
  <td><a href="../LICENSE-APACHE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License"/></a></td>
  <td><a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.70%2B-orange.svg" alt="Rust"/></a></td>
</tr>
</table>

Procedural macros for the memlink SDK. Provides automatic code generation for exporting Rust functions as memlink module methods with serialization, FFI bindings, and panic isolation.

## Features

- **`#[memlink_export]` Macro**: Automatic wrapper generation for module methods
- **Compile-Time Hashing**: FNV-1a method hash computation at compile time
- **Automatic Serialization**: MessagePack serialization for arguments and return values
- **FFI Export Generation**: C-compatible extern functions with panic isolation
- **Async Support**: Handles both sync and async functions
- **Custom Method Names**: Optional `name` attribute for method hash customization

## Quick Start

### Basic Usage

```rust
use memlink_msdk::prelude::*;

#[memlink_export]
pub fn echo(ctx: &CallContext, input: String) -> Result<String> {
    Ok(input)
}

#[memlink_export]
pub fn add(ctx: &CallContext, a: u32, b: u32) -> Result<u32> {
    Ok(a + b)
}
```

### Async Functions

```rust
#[memlink_export]
pub async fn async_process(ctx: &CallContext, data: Vec<u8>) -> Result<Vec<u8>> {
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    Ok(data)
}
```

### Custom Method Names

```rust
#[memlink_export(name = "custom_name")]
pub fn my_function(ctx: &CallContext, value: u32) -> Result<u32> {
    Ok(value * 2)
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

### Nested Module Calls

```rust
#[memlink_export]
pub async fn call_other(ctx: &CallContext, module: String) -> Result<Vec<u8>> {
    let caller = ctx.module_caller().ok_or(ModuleError::ServiceUnavailable)?;
    caller.call(&module, "method", b"args").await
}
```

## What the Macro Generates

When you annotate a function with `#[memlink_export]`, the macro generates:

1. **Args Struct**: Serialization struct for function parameters
2. **Wrapper Function**: Handles serialization/deserialization
3. **FFI Export**: C-compatible extern function with panic isolation
4. **Registration Code**: Method dispatch table registration

### Example Expansion

```rust
// Your code
#[memlink_export]
pub fn echo(ctx: &CallContext, input: String) -> Result<String> {
    Ok(input)
}

// Macro generates (simplified):
pub fn echo(ctx: &CallContext, input: String) -> Result<String> {
    Ok(input)
}

struct __echoArgs {
    pub input: String,
}

fn __echo_wrapper(ctx: &CallContext, args_bytes: &[u8]) -> Result<Vec<u8>> {
    let args: __echoArgs = deserialize(args_bytes)?;
    let result = echo(ctx, args.input)?;
    serialize(&result)
}

#[no_mangle]
pub unsafe extern "C" fn __echo_ffi(...) -> i32 {
    // FFI boundary with panic isolation
}
```

## Function Requirements

Functions annotated with `#[memlink_export]` must:

1. **First Parameter**: `&CallContext` or `CallContext`
2. **Return Type**: `Result<T>` where `T: Serialize + DeserializeOwned`
3. **Sync or Async**: Both are supported

## Supported Types

The macro supports any type that implements `serde::Serialize` and `serde::de::DeserializeOwned`:

- Primitives: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `bool`, `char`
- Strings: `String`, `&str`
- Collections: `Vec<T>`, `Option<T>`, `HashMap<K, V>`
- Tuples: `(T1, T2, ...)` where each T implements the traits
- Custom structs: Derive `Serialize` and `Deserialize`

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MyData {
    pub name: String,
    pub value: u32,
}

#[memlink_export]
pub fn process_data(ctx: &CallContext, data: MyData) -> Result<MyData> {
    Ok(MyData {
        name: data.name.to_uppercase(),
        value: data.value * 2,
    })
}
```

## Error Handling

The macro automatically handles:

- **Serialization Errors**: Converted to `ModuleError::Serialize`
- **Panic Isolation**: Panics caught and converted to `ModuleError::Panic`
- **FFI Safety**: Null pointer checks and buffer validation

## Performance

| Operation | Overhead |
|-----------|----------|
| Macro expansion | Compile-time only |
| Serialization | ~100-500 ns per KB |
| FFI boundary | ~50 ns |
| Panic isolation | ~10 ns |

## Integration with memlink-msdk

The macros are re-exported, so you can use them directly:

```rust
use memlink_msdk::prelude::*;

#[memlink_export]  // From memlink-msdk-macros
pub fn my_method(ctx: &CallContext, arg: u32) -> Result<u32> {
    Ok(arg)
}
```

## Examples

See the [examples](examples/) directory for complete working examples:

- `basic.rs` - Basic function exports
- `async_echo.rs` - Async function handling
- `arena_usage.rs` - Arena allocation patterns
- `nested_calls.rs` - Module-to-module communication

## Testing

Test your exported functions using the SDK's test utilities:

```rust
#[cfg(test)]
mod tests {
    use memlink_msdk::prelude::*;
    use crate::echo;

    fn create_test_context() -> CallContext<'static> {
        // Create test context with leaked arena
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
