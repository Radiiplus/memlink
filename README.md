# MemLink

High-performance inter-process communication (IPC) toolkit for Rust.

## Overview

MemLink provides a collection of optimized IPC mechanisms for building fast, reliable communication between processes on the same machine. The toolkit focuses on:

- **Low latency** - Sub-microsecond message passing
- **High throughput** - Hundreds of thousands of messages per second
- **Cross-platform** - Windows, Linux, and macOS support
- **Memory safety** - Bounds checking, panic guards, and safe abstractions

## Components

### SHM - Shared Memory IPC

The core component of MemLink is the **SHM** crate - a lock-free shared memory IPC library with:

- Multi-priority message queues (Critical, High, Low)
- Daemon-client architecture
- Futex-based signaling for efficient waiting
- Crash recovery and heartbeat monitoring
- Backpressure control

**Get Started with SHM:**
- 📖 [Documentation](shm/README.md)
- 📦 [crates.io](https://crates.io/crates/memlink-shm)
- 📚 [API Docs](https://docs.rs/memlink-shm)

**Quick Example:**

```rust
use memlink_shm::buffer::{RingBuffer, Priority};

let rb = RingBuffer::new(256).unwrap();
rb.write_slot(Priority::High, b"Hello!").unwrap();
let (_, data) = rb.read_slot().unwrap();
```

**Performance:**
| Payload | Latency (p50) | Throughput |
|---------|---------------|------------|
| Empty | 0.8 μs | 850K msg/sec |
| 64 bytes | 1.2 μs | 620K msg/sec |
| 1 KB | 2.8 μs | 280K msg/sec |
| 4 KB | 8.5 μs | 95K msg/sec |

See [SHM README](shm/README.md) for detailed benchmarks and use cases.

## Project Structure

```
memlink/
├── shm/           # Shared memory IPC library (published as separate crate)
│   ├── src/       # Source code
│   ├── README.md  # Detailed documentation
│   └── PUBLISHING.md  # Guide for publishing to crates.io
├── Cargo.toml     # Workspace configuration
└── README.md      # This file
```

## Use Cases

MemLink is ideal for:

- **High-frequency trading** - Ultra-low latency market data distribution
- **Game engines** - Fast communication between subsystems (physics, rendering, AI)
- **Microservices** - Same-machine service communication without network overhead
- **Plugin systems** - Host-plugin communication
- **Data pipelines** - Real-time sensor data ingestion with priority handling

## Installation

### SHM Crate

```toml
[dependencies]
memlink-shm = "0.1.0"
```

## Development

### Prerequisites

- Rust 1.70 or later
- For benchmarks: Python (for criterion plots)

### Building

```bash
# Build the SHM crate
cd shm
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Generate documentation
cargo doc --open
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-features -- -D warnings
```

## Roadmap

Future components planned for MemLink:

- [ ] **Named Pipes** - Cross-platform named pipe abstraction
- [ ] **Unix Domain Sockets** - Optimized socket communication
- [ ] **Message Queues** - POSIX message queue wrapper
- [ ] **Memory Pools** - Lock-free memory pool allocators

## License

MemLink is licensed under the Apache License 2.0 ([LICENSE-APACHE](shm/LICENSE-APACHE)).

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Run `cargo test` and `cargo clippy`
4. Submit a pull request

## Support

- **Issues**: Open an issue on GitHub
- **Documentation**: See individual crate README files
- **API Reference**: https://docs.rs/memlink-shm

## Acknowledgments

- Uses [memmap2](https://crates.io/crates/memmap2) for cross-platform memory mapping
- Futex implementation inspired by Linux kernel design
- Ring buffer design based on lock-free SPSC patterns
