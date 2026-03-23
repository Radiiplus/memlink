# memlink-protocol

**Binary protocol definitions for MemLink IPC**

<table>
<tr>
  <td><a href="../LICENSE-APACHE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License"/></a></td>
  <td><a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.70%2B-orange.svg" alt="Rust"/></a></td>
  <td><a href="https://crates.io/crates/memlink-protocol"><img src="https://img.shields.io/crates/v/memlink-protocol.svg" alt="Version"/></a></td>
</tr>
</table>

---

## Overview

`memlink-protocol` provides the core protocol definitions for MemLink IPC, including fixed 32-byte message headers, version negotiation, feature flags, and MessagePack serialization.

**Key Features:**
- 📦 Fixed 32-byte message headers with packed representation
- 🔢 Magic numbers and version constants for protocol identification
- 🏷️ Type aliases for consistent sizing across platforms
- ❌ Comprehensive error types for all failure modes
- 🔄 Version negotiation with feature flag support
- ⚡ Zero-copy message parsing from shared memory
- 🎯 Arena-backed slices for efficient memory management
- 🔐 Stream handles for large payload transfers

---

### Features

| Feature | Description |
|---------|-------------|
| `std` | Enable std library support (for threading tests) |
| `shm` | Enable integration with `memlink-shm` crate |

---

## Quick Start

```rust
use memlink_protocol::{MessageHeader, MessageType, Request, Response};
use memlink_protocol::msgpack::MessagePackSerializer;
use memlink_protocol::serializer::Serializer;

// Create a request header
let header = MessageHeader::new(
    MessageType::Request,
    1,           // request_id
    42,          // module_id
    0x12345678,  // method_hash
    256,         // payload_len
);

// Validate the header
assert!(header.validate().is_ok());

// Convert to bytes for transmission
let bytes = header.as_bytes();

// Parse from bytes
let parsed = MessageHeader::from_bytes(&bytes)?;

// Create and serialize a request
let request = Request::new(
    1,
    memlink_protocol::Priority::Normal,
    "calculator",
    "add",
    vec![1, 2, 3, 4],
);

let bytes = MessagePackSerializer.serialize_request(&request)?;
let parsed = MessagePackSerializer.deserialize_request(&bytes)?;
```

📚 [More examples](examples/basic.rs)

---

## Wire Format

All multi-byte fields use **big-endian** byte order for network transmission:

| Offset | Size | Field        | Type  | Description                    |
|--------|------|--------------|-------|--------------------------------|
| 0      | 4    | magic        | u32   | Magic number (0x4D4C4E4B)      |
| 4      | 1    | version      | u8    | Protocol version               |
| 5      | 1    | msg_type     | u8    | Message type                   |
| 6      | 2    | features     | u16   | Feature flags (big-endian)     |
| 8      | 8    | request_id   | u64   | Request identifier             |
| 16     | 8    | module_id    | u64   | Module identifier              |
| 24     | 4    | method_hash  | u32   | Method name hash               |
| 28     | 4    | payload_len  | u32   | Payload size in bytes          |

**Total: 32 bytes** (exactly)

---

## Protocol Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MEMLINK_MAGIC` | `0x4D4C4E4B` | "MLNK" in ASCII, big-endian |
| `PROTOCOL_VERSION` | `1` | Current protocol version |
| `HEADER_SIZE` | `32` | Header size in bytes |
| `MAX_PAYLOAD_SIZE` | `64 MiB` | Maximum payload (67,108,864 bytes) |
| `CONTROL_REGION_SIZE` | `4 KiB` | SHM control region (4,096 bytes) |

---

## Type Aliases

| Type | Alias | Description |
|------|-------|-------------|
| `RequestId` | `u64` | Unique identifier for request/response correlation |
| `ModuleId` | `u64` | Identifier for modules or services |
| `MethodHash` | `u32` | FNV-1a hash of method name for dispatch |
| `TraceId` | `u128` | Distributed tracing identifier |
| `SpanId` | `u64` | Span identifier within a trace |

---

## Message Types

| Type | Value | Description |
|------|-------|-------------|
| `Request` | 0 | Initiates an operation (expects response) |
| `Response` | 1 | Completes an operation |
| `Error` | 2 | Indicates failure |
| `StreamHandle` | 3 | Stream handle for streaming operations |
| `HealthCheck` | 16 | Health check request/response |
| `LoadModule` | 32 | Load a new module |
| `UnloadModule` | 33 | Unload a module |
| `Stats` | 48 | Statistics request/response |
| `Event` | 96 | Event notification |

---

## Status Codes

| Status | Value | Description |
|--------|-------|-------------|
| `Success` | 0 | Operation completed successfully |
| `ModuleNotFound` | 1 | The specified module was not found |
| `MethodNotFound` | 2 | The specified method was not found |
| `ExecutionError` | 3 | Execution failed with an error |
| `Timeout` | 4 | Operation timed out |
| `QuotaExceeded` | 5 | Resource quota exceeded |
| `BackpressureRejection` | 6 | Request rejected due to backpressure |

---

## Feature Flags

| Feature | Bit | Description |
|---------|-----|-------------|
| `STREAMING` | 0 | Chunked transfer support |
| `BATCHING` | 1 | Batch message grouping |
| `PRIORITY_DEGRADATION` | 2 | Priority-based fallback |

---

## Version Negotiation

```rust
use memlink_protocol::{negotiate_version, validate_version, V1_0, V1_1, V1_2};

// Negotiate between client and server versions
let result = negotiate_version(&V1_2, &V1_0);
assert!(result.is_ok());
assert_eq!(result.unwrap().minor, 0); // Negotiates to lowest common

// Validate version support
assert!(validate_version(&V1_0).is_ok());
assert!(validate_version(&V1_2).is_ok());
```

---

## Performance

Benchmark results from `cargo bench -p memlink-protocol`:

### Serialization Performance

| Operation | Time | Throughput |
|-----------|------|------------|
| Request serialize | 1.07 µs | 4.59 MiB/s |
| Request deserialize | 1.11 µs | 4.44 MiB/s |
| Response serialize | 1.17 µs | 4.16 MiB/s |
| Response deserialize | 491 ns | 10.2 MiB/s |
| Error serialize | 1.00 µs | 20.8 MiB/s |
| Error deserialize | 477 ns | 43.7 MiB/s |

### Large Payload Performance

| Payload | Serialize | Deserialize |
|---------|-----------|-------------|
| 1 KB | 18.2 µs (55.5 MiB/s) | 12.0 µs (83.3 MiB/s) |
| 10 KB | 109 µs (89.4 MiB/s) | 121 µs (81.0 MiB/s) |
| 100 KB | 1.30 ms (75.4 MiB/s) | 1.55 ms (62.9 MiB/s) |

### Header Operations

| Operation | Time |
|-----------|------|
| `as_bytes()` | 46 ns |
| `from_bytes()` | 37 ns |
| `validate()` | <1 ns |

📊 [Run benchmarks](#development)

---

## Error Handling

```rust
use memlink_protocol::ProtocolError;

match header.validate() {
    Ok(()) => { /* valid header */ }
    Err(ProtocolError::InvalidMagic(magic)) => { /* wrong magic */ }
    Err(ProtocolError::UnsupportedVersion(ver)) => { /* unsupported version */ }
    Err(ProtocolError::PayloadTooLarge(size, max)) => { /* payload too big */ }
    Err(ProtocolError::SerializationFailed(msg)) => { /* serialization error */ }
    Err(ProtocolError::BufferOverflow { required, available }) => { /* buffer full */ }
}
```

### Error Recovery

| Error Type | Recoverable | Description |
|------------|-------------|-------------|
| `Timeout` | ✅ Yes | Retry with backoff |
| `BufferOverflow` | ✅ Yes | Apply backpressure |
| `QuotaExceeded` | ✅ Yes | Wait for resources |
| `InvalidMagic` | ❌ No | Data corruption |
| `UnsupportedVersion` | ❌ No | Protocol mismatch |

---

## Design Principles

- **Binary-only**: No JSON or text-based protocols for maximum performance
- **Fixed header size**: Exactly 32 bytes for predictable memory layout
- **Platform-independent**: Fixed-width types (u32, u64) and big-endian wire format
- **Explicit packing**: `#[repr(C, packed)]` to prevent padding
- **Safe conversion**: No `std::mem::transmute` for byte conversion

---

## Safety

This crate follows strict safety guidelines:

- ✅ No `unsafe` code in core implementation
- ✅ No `transmute` for byte conversion (uses explicit `to_be_bytes`)
- ✅ Compile-time size assertions for header structure
- ✅ Comprehensive validation before parsing
- ✅ Bounds checking on all buffer operations

---

## Development

### Prerequisites

- Rust 1.70 or later

### Building

```bash
# Build the crate
cargo build -p memlink-protocol

# Build with SHM integration
cargo build -p memlink-protocol --features shm
```

### Testing

```bash
# Run all tests
cargo test -p memlink-protocol --features std

# Run integration tests
cargo test -p memlink-protocol --features std --test integration

# Run doc tests
cargo test -p memlink-protocol --doc
```

### Benchmarks

```bash
# Run all benchmarks
cargo bench -p memlink-protocol --features std

# Run specific benchmark
cargo bench -p memlink-protocol --features std -- serialization
```

### Code Quality

```bash
# Format code
cargo fmt -p memlink-protocol

# Run clippy
cargo clippy -p memlink-protocol --features std -- -D warnings
```

### Examples

```bash
# Run the basic example
cargo run -p memlink-protocol --example basic --features std
```

---

## Related Crates

| Crate | Description |
|-------|-------------|
| [memlink-shm](../shm) | Shared memory IPC library |
| [memlink-runtime](../runtime) | Dynamic module loading |
| [memlink-msdk](../msdk) | Module SDK |
| [memlink-msdk-macros](../msdk-macros) | Proc macros |

---

## License

Apache-2.0 license.

See [LICENSE-APACHE](../LICENSE-APACHE) for the full license text.

---

## Contributing

Contributions are welcome! Please follow the project guidelines:

1. Add tests for new functionality
2. Ensure `cargo clippy` passes with no warnings
3. Update documentation for API changes
4. Follow the existing code style

---

*memlink-protocol - Fast, reliable binary protocol for Rust IPC*
