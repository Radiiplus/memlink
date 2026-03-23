# MemLink

**High-performance IPC and dynamic module loading toolkit for Rust**

<table>
<tr>
  <td><a href="LICENSE-APACHE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License"/></a></td>
  <td><a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.80%2B-orange.svg" alt="Rust"/></a></td>
  <td><a href="https://github.com/memlink/memlink/issues"><img src="https://img.shields.io/github/issues/memlink/memlink" alt="Issues"/></a></td>
</tr>
<tr>
  <td><a href="https://crates.io/crates/memlink-shm"><img src="https://img.shields.io/crates/v/memlink-shm.svg" alt="SHM"/></a></td>
  <td><a href="https://crates.io/crates/memlink-protocol"><img src="https://img.shields.io/crates/v/memlink-protocol.svg" alt="Protocol"/></a></td>
  <td><a href="https://crates.io/crates/memlink-runtime"><img src="https://img.shields.io/crates/v/memlink-runtime.svg" alt="Runtime"/></a></td>
  <td><a href="https://crates.io/crates/memlink-msdk"><img src="https://img.shields.io/crates/v/memlink-msdk.svg" alt="MSDK"/></a></td>
  <td><a href="https://crates.io/crates/memlink-msdk-macros"><img src="https://img.shields.io/crates/v/memlink-msdk-macros.svg" alt="MSDK Macros"/></a></td>
</tr>
</table>

---

## Overview

MemLink is a collection of high-performance Rust libraries for **inter-process communication (IPC)** and **dynamic module loading**. Built for low-latency, high-throughput applications, MemLink provides production-ready solutions for same-machine communication and plugin architectures.

### Core Capabilities

| Capability | Crate | Use Case |
|------------|-------|----------|
| **Binary Protocol** | `memlink-protocol` | Message serialization and version negotiation |
| **Shared Memory IPC** | `memlink-shm` | Ultra-low latency process communication |
| **Dynamic Module Loading** | `memlink-runtime` | Plugin systems, hot-reloadable code |
| **Module SDK** | `memlink-msdk` | Build memlink modules with ease |
| **Proc Macros** | `memlink-msdk-macros` | Automatic code generation for modules |

---

## Crates

### 📦 memlink-protocol

Binary protocol definitions with MessagePack serialization and version negotiation.

**Features:**
- 📦 Fixed 32-byte message headers
- 🔢 Magic numbers and version constants
- 🏷️ Type aliases and enumerations
- ❌ Comprehensive error types
- 🔄 Version negotiation with feature flags
- ⚡ Zero-copy message parsing
- 🎯 Arena-backed slices

**Performance:**

Benchmark results from `cargo bench -p memlink-protocol`:

| Operation | Time | Throughput |
|-----------|------|------------|
| Request serialize | 1.07 µs | 4.59 MiB/s |
| Response deserialize | 491 ns | 10.2 MiB/s |
| Error deserialize | 477 ns | 43.7 MiB/s |
| Header `as_bytes()` | 46 ns | - |
| Header `from_bytes()` | 37 ns | - |

**Quick Start:**
```rust
use memlink_protocol::{MessageHeader, MessageType, Request, Response};
use memlink_protocol::msgpack::MessagePackSerializer;
use memlink_protocol::serializer::Serializer;

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

📖 [Documentation](protocol/README.md) | 📦 [crates.io](https://crates.io/crates/memlink-protocol) | 📚 [API Docs](https://docs.rs/memlink-protocol)

---

### 📦 memlink-shm

Lock-free shared memory IPC with multi-priority message queues.

**Features:**
- 🔒 Lock-free ring buffer design
- 📊 Multi-priority queues (Critical, High, Low)
- ⚡ Futex-based signaling (Linux) / Event-based (Windows)
- 🛡️ Crash recovery and heartbeat monitoring
- 📈 Backpressure control

**Performance:**
| Payload | Latency (p50) | Throughput |
|---------|---------------|------------|
| Empty | 0.8 μs | 850K msg/sec |
| 64 bytes | 1.2 μs | 620K msg/sec |
| 1 KB | 2.8 μs | 280K msg/sec |

**Quick Start:**
```rust
use memlink_shm::buffer::{RingBuffer, Priority};

let rb = RingBuffer::new(256).unwrap();
rb.write_slot(Priority::High, b"Hello!").unwrap();
let (_, data) = rb.read_slot().unwrap();
```

📖 [Documentation](shm/README.md) | 📦 [crates.io](https://crates.io/crates/memlink-shm) | 📚 [API Docs](https://docs.rs/memlink-shm)

---

### 🔧 memlink-msdk

SDK for building memlink modules with automatic serialization, FFI exports, and panic isolation.

**Features:**
- 🎯 `#[memlink_export]` proc macro for easy method exports
- 🔄 Automatic serialization/deserialization (MessagePack)
- 🛡️ Built-in panic isolation at FFI boundary
- 📊 Arena allocation for bounded memory management
- 🔗 Nested module-to-module calls
- 📈 Backpressure and deadline tracking
- 📝 Structured logging and metrics APIs

**Performance:**

Benchmark results from `cargo bench -p memlink-msdk`:

| Benchmark | Time | Throughput |
|-----------|------|------------|
| Function call (empty) | 1.14 ns | 874.6 M/sec |
| Function call + serialization | 98.8 ns | 10.1 M/sec |
| Arena allocation (u64) | 2.41 ns | 414.8 M/sec |
| Arena alloc + init | 3.31 ns | 301.8 M/sec |

**Quick Start:**
```rust
use memlink_msdk::prelude::*;

#[memlink_export]
pub fn echo(ctx: &CallContext, input: String) -> Result<String> {
    Ok(input)
}

#[memlink_export]
pub async fn async_echo(ctx: &CallContext, data: Vec<u8>) -> Result<Vec<u8>> {
    Ok(data)
}

#[memlink_export]
pub fn use_arena(ctx: &CallContext) -> Result<u64> {
    let x = ctx.arena().alloc::<u64>().ok_or(ModuleError::QuotaExceeded)?;
    unsafe { std::ptr::write(x, 42); }
    Ok(*x)
}
```

📖 [Documentation](msdk/README.md) | 📚 [API Docs](https://docs.rs/memlink-msdk) | 🧪 [Examples](msdk/examples/)

---

### 🔮 memlink-msdk-macros

Procedural macros for automatic code generation in memlink modules.

**Features:**
- 🎯 `#[memlink_export]` attribute macro for effortless method exports
- ⚡ Compile-time FNV-1a method hash computation
- 🔄 Automatic MessagePack serialization/deserialization
- 🛡️ FFI boundary generation with built-in panic isolation
- 🔀 Support for both sync and async functions
- 🏷️ Custom method name attribution

**Quick Start:**
```rust
use memlink_msdk::prelude::*;

#[memlink_export]
pub fn echo(ctx: &CallContext, input: String) -> Result<String> {
    Ok(input)
}

#[memlink_export]
pub async fn async_process(ctx: &CallContext, data: Vec<u8>) -> Result<Vec<u8>> {
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    Ok(data)
}
```

**What the Macro Generates:**
- Args struct for parameter serialization
- Wrapper function for serialization handling
- FFI export function with panic isolation
- Method dispatch table registration

📖 [Documentation](msdk-macros/README.md) | 📚 [API Docs](https://docs.rs/memlink-msdk-macros) | 🧪 [Examples](msdk-macros/examples/)

---

### 🔌 memlink-runtime

Dynamic module loading framework for plugin architectures and hot-reloadable code.

**Features:**
- 🔌 Load `.so`, `.dll`, `.dylib` at runtime
- 🛡️ Panic isolation - module crashes don't affect host
- 🔥 Hot reload with zero downtime
- 📊 Prometheus-compatible metrics
- 🧵 Thread-safe concurrent module calls
- 📦 Multi-module management

**Performance:**
| Operation | Latency | Throughput |
|-----------|---------|------------|
| Module load | 92 μs | - |
| Method call | 210 ns | 4.7M calls/sec |
| Hot reload | 237 μs | - |

**Quick Start:**
```rust
use memlink_runtime::runtime::{Runtime, ModuleRuntime};
use memlink_runtime::resolver::ModuleRef;

let runtime = Runtime::with_local_resolver();
let handle = runtime.load(ModuleRef::parse("./plugin.so")?)?;
let result = runtime.call(handle, "process", b"input")?;
```

📖 [Documentation](runtime/README.md) | 📚 [API Docs](https://docs.rs/memlink-runtime) | 📊 [Benchmarks](runtime/docs/perf.md)

---

## Project Structure

```
memlink/
├── protocol/              # Binary protocol definitions
│   ├── src/              # Source code
│   ├── examples/         # Usage examples
│   ├── benches/          # Performance benchmarks
│   └── README.md         # Detailed documentation
├── shm/                    # Shared memory IPC library
│   ├── src/               # Source code
│   ├── examples/          # Usage examples
│   ├── benches/           # Performance benchmarks
│   └── README.md          # Detailed documentation
├── runtime/               # Dynamic module loading
│   ├── src/              # Source code
│   ├── examples/         # Example modules and demos
│   ├── docs/             # ABI spec and benchmarks
│   └── README.md         # Detailed documentation
├── msdk/                  # Module SDK
│   ├── src/              # SDK source code
│   ├── tests/            # Integration tests
│   └── tests/integration.rs  # All-in-one integration test suite
├── msdk-macros/           # Proc macros for msdk
│   └── src/              # Macro implementations
├── Cargo.toml            # Workspace configuration
└── README.md             # This file
```

---

## Installation

### memlink-protocol

```toml
[dependencies]
memlink-protocol = "0.1.0"

# With shared memory integration
memlink-protocol = { version = "0.1.0", features = ["shm"] }
```

### memlink-shm

```toml
[dependencies]
memlink-shm = "0.1.3"
```

### memlink-runtime

```toml
[dependencies]
memlink-runtime = "0.1.2"
```

### memlink-msdk

```toml
[dependencies]
memlink-msdk = "0.1.2"
```

### memlink-msdk-macros

```toml
[dependencies]
memlink-msdk-macros = "0.1.2"
```

Note: `memlink-msdk-macros` is automatically included when you install `memlink-msdk`.

---

## Use Cases

### High-Frequency Trading
Ultra-low latency market data distribution between processes.

```rust
// SHM: Sub-microsecond tick data distribution
let buffer = RingBuffer::new(1024).unwrap();
buffer.write_slot(Priority::Critical, tick_data)?;
```

### Plugin Systems
Load user-created plugins without recompiling the host application.

```rust
// Runtime: Dynamic plugin loading
for plugin in std::fs::read_dir("./plugins")? {
    runtime.load(ModuleRef::parse(plugin.path())?)?;
}
```

### Hot-Reloadable Business Logic
Update rules and logic in production without downtime.

```rust
// Runtime: Zero-downtime reload
runtime.reload_with_config(handle, new_ref, reload_config)?;
```

### Game Engine Subsystems
Fast communication between physics, rendering, and AI systems.

```rust
// SHM: Multi-priority message passing
physics_queue.write(Priority::High, physics_state);
render_queue.write(Priority::Low, render_commands);
```

### Microservices Communication
Same-machine service communication without network overhead.

```rust
// SHM: Replace network calls with shared memory
let response = shm_client.request(&request)?;  // ~1 μs vs ~100 μs for TCP
```

---

## Platform Support

| Platform | memlink-protocol | memlink-shm | memlink-runtime |
|----------|------------------|-------------|-----------------|
| Linux | ✅ Tested | ✅ Tested | ✅ Tested |
| Windows | ✅ Tested | ✅ Tested | ✅ Tested |
| macOS | ✅ Tested | ✅ Tested | ✅ Tested |

---

## Development

### Prerequisites

- Rust 1.80 or later
- C compiler (for runtime module examples)
- Node.js 18+ (for runtime build scripts)
- WSL2 (for Linux builds on Windows)

### Building

```bash
# Build all crates
cargo build --workspace

# Build specific crate
cargo build -p memlink-shm
cargo build -p memlink-runtime
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Test specific crate
cargo test -p memlink-protocol
cargo test -p memlink-shm
cargo test -p memlink-runtime
cargo test -p memlink-msdk

# Run integration tests only
cargo test --test integration -p memlink-protocol
cargo test --test integration -p memlink-msdk

# Run doc tests only
cargo test --doc -p memlink-protocol
cargo test --doc -p memlink-msdk
```

### Benchmarks

```bash
# Run benchmarks
cargo bench -p memlink-protocol
cargo bench -p memlink-shm
cargo bench -p memlink-runtime

# Build test modules first (for runtime)
cd runtime/examples/modules/build
node build.js --linux
```

### Code Quality

```bash
# Format all code
cargo fmt --all

# Run clippy lints
cargo clippy --workspace --all-targets -- -D warnings

# Check msdk specifically
cargo clippy -p memlink-msdk
cargo clippy -p memlink-msdk-macros
```

---

## Performance Summary

| Crate | Operation | Latency | Throughput |
|-------|-----------|---------|------------|
| **memlink-protocol** | Request serialize | 1.07 µs | 4.59 MiB/s |
| **memlink-protocol** | Response deserialize | 491 ns | 10.2 MiB/s |
| **memlink-protocol** | Error deserialize | 477 ns | 43.7 MiB/s |
| **memlink-protocol** | Header `as_bytes()` | 46 ns | - |
| **memlink-shm** | Empty message | 0.8 μs | 850K/sec |
| **memlink-shm** | 1 KB payload | 2.8 μs | 280K/sec |
| **memlink-msdk** | Function call (empty) | 1.14 ns | 874.6 M/sec |
| **memlink-msdk** | Function call + serialization | 98.8 ns | 10.1 M/sec |
| **memlink-msdk** | Arena allocation | 2.41 ns | 414.8 M/sec |
| **memlink-runtime** | Module load | 92 μs | - |
| **memlink-runtime** | Method call | 210 ns | 4.7M/sec |

See individual crate documentation for detailed benchmarks:
- [Protocol Benchmarks](protocol/README.md#performance)
- [SHM Benchmarks](shm/README.md#performance)
- [Runtime Benchmarks](runtime/docs/PERFORMANCE.md)
- [MSDK Benchmarks](msdk/README.md#performance)

---

## Roadmap

### memlink-shm
- [ ] Zero-copy message serialization
- [ ] Multi-producer multi-consumer (MPMC) queues
- [ ] Persistent shared memory segments

### memlink-runtime
- [ ] WebAssembly module support
- [ ] Module sandboxing with seccomp/AppContainer
- [ ] Automatic module discovery and loading

### memlink-msdk
- [ ] Full `#[memlink_export]` macro implementation
- [ ] Zero-copy arena-based argument passing
- [ ] Built-in metrics and logging exporters
- [ ] Module hot-reload support

### New Components
- [ ] `memlink-pipe` - Cross-platform named pipe abstraction
- [ ] `memlink-pool` - Lock-free memory pool allocator

---

## License

MemLink is licensed under the Apache License 2.0.

See [LICENSE-APACHE](shm/LICENSE-APACHE) for the full license text.

---

## Contributing

Contributions are welcome! Please follow these steps:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/my-feature`)
3. **Implement** your changes with tests
4. **Verify** code quality:
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets
   cargo test --workspace
   ```
5. **Submit** a pull request

### Development Guidelines

- Follow Rust API Guidelines
- Add tests for new functionality
- Update documentation for API changes
- Include benchmarks for performance-sensitive code

---

## Support

| Resource | Link |
|----------|------|
| **Issues** | [GitHub Issues](https://github.com/memlink/memlink/issues) |
| **Documentation** | See individual crate README files |
| **API Reference** | [docs.rs/memlink-shm](https://docs.rs/memlink-shm), [docs.rs/memlink-runtime](https://docs.rs/memlink-runtime) |
| **Discussions** | [GitHub Discussions](https://github.com/memlink/memlink/discussions) |

---

## Acknowledgments

- **[memmap2](https://crates.io/crates/memmap2)** - Cross-platform memory mapping
- **[libloading](https://crates.io/crates/libloading)** - Dynamic library loading
- **[dashmap](https://crates.io/crates/dashmap)** - Concurrent hash maps
- Linux kernel futex implementation
- Lock-free SPSC ring buffer patterns

---

*MemLink - Building fast, reliable Rust applications together.*
