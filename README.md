# MemLink

**High-performance IPC and dynamic module loading toolkit for Rust**

<table>
<tr>
  <td><a href="LICENSE-APACHE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License"/></a></td>
  <td><a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.70%2B-orange.svg" alt="Rust"/></a></td>
  <td><a href="https://github.com/memlink/memlink/issues"><img src="https://img.shields.io/github/issues/memlink/memlink" alt="Issues"/></a></td>
</tr>
<tr>
  <td><a href="https://crates.io/crates/memlink-shm"><img src="https://img.shields.io/crates/v/memlink-shm.svg" alt="SHM"/></a></td>
  <td><a href="https://crates.io/crates/memlink-runtime"><img src="https://img.shields.io/crates/v/memlink-runtime.svg" alt="Runtime"/></a></td>
</tr>
</table>

---

## Overview

MemLink is a collection of high-performance Rust libraries for **inter-process communication (IPC)** and **dynamic module loading**. Built for low-latency, high-throughput applications, MemLink provides production-ready solutions for same-machine communication and plugin architectures.

### Core Capabilities

| Capability | Crate | Use Case |
|------------|-------|----------|
| **Shared Memory IPC** | `memlink-shm` | Ultra-low latency process communication |
| **Dynamic Module Loading** | `memlink-runtime` | Plugin systems, hot-reloadable code |

---

## Crates

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

📖 [Documentation](runtime/README.md) | 📚 [API Docs](https://docs.rs/memlink-runtime) | 📊 [Benchmarks](runtime/docs/PERFORMANCE.md)

---

## Project Structure

```
memlink/
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
├── Cargo.toml            # Workspace configuration
└── README.md             # This file
```

---

## Installation

### memlink-shm

```toml
[dependencies]
memlink-shm = "0.1.0"
```

### memlink-runtime

```toml
[dependencies]
memlink-runtime = "0.1.0"
```

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

| Platform | memlink-shm | memlink-runtime |
|----------|-------------|-----------------|
| Linux | ✅ Tested | ✅ Tested |
| Windows | ✅ Tested | ✅ Tested |
| macOS | ✅ Tested | ✅ Tested |

---

## Development

### Prerequisites

- Rust 1.70 or later
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
cargo test -p memlink-shm
cargo test -p memlink-runtime
```

### Benchmarks

```bash
# Run benchmarks
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
```

---

## Performance Summary

| Crate | Operation | Latency | Throughput |
|-------|-----------|---------|------------|
| **memlink-shm** | Empty message | 0.8 μs | 850K/sec |
| **memlink-shm** | 1 KB payload | 2.8 μs | 280K/sec |
| **memlink-runtime** | Module load | 92 μs | - |
| **memlink-runtime** | Method call | 210 ns | 4.7M/sec |

See individual crate documentation for detailed benchmarks:
- [SHM Benchmarks](shm/README.md#performance)
- [Runtime Benchmarks](runtime/docs/PERFORMANCE.md)

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
