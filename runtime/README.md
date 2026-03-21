# Memlink Runtime

**Dynamic module loading and execution framework for Rust**

<table>
<tr>
  <td><a href="https://crates.io/crates/memlink-runtime"><img src="https://img.shields.io/crates/v/memlink-runtime.svg" alt="Crates.io"/></a></td>
  <td><a href="https://docs.rs/memlink-runtime"><img src="https://docs.rs/memlink-runtime/badge.svg" alt="Docs"/></a></td>
  <td><a href="../LICENSE-APACHE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License"/></a></td>
  <td><a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.70%2B-orange.svg" alt="Rust"/></a></td>
</tr>
<tr>
  <td><a href="docs/PERFORMANCE.md"><img src="https://img.shields.io/badge/benchmarks-included-purple" alt="Benchmarks"/></a></td>
  <td><a href="../shm/README.md"><img src="https://img.shields.io/badge/sibling-shm-blue" alt="SHM"/></a></td>
  <td><a href="examples/README.md"><img src="https://img.shields.io/badge/examples-included-green" alt="Examples"/></a></td>
</tr>
</table>

---

## Overview

Memlink Runtime is a high-performance dynamic module loading system that enables **plugin architectures**, **hot-reloadable code**, and **safe FFI execution**. Load shared libraries at runtime, call methods on them, and unload them without restarting your application.

### Key Features

- 🔌 **Dynamic Loading** - Load `.so`, `.dll`, `.dylib` files at runtime
- 🛡️ **Panic Isolation** - Module panics don't crash your application
- 🔥 **Hot Reload** - Replace modules with zero downtime
- 📊 **Prometheus Metrics** - Built-in observability
- 🧵 **Thread-Safe** - Concurrent module calls from multiple threads
- 📦 **Multi-Module** - Load and manage multiple modules simultaneously

---

## Quick Start

### Basic Example

```rust
use memlink_runtime::runtime::{Runtime, ModuleRuntime};
use memlink_runtime::resolver::ModuleRef;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create runtime
    let runtime = Runtime::with_local_resolver();
    
    // Load a module
    let handle = runtime.load(
        ModuleRef::parse("./my_module.so")?
    )?;
    
    // Call a method
    let result = runtime.call(handle, "process", b"input data")?;
    println!("Result: {:?}", String::from_utf8_lossy(&result));
    
    // Unload when done
    runtime.unload(handle)?;
    
    Ok(())
}
```

---

## Creating Modules

Modules are shared libraries that export three required functions:

### Minimal Module (C)

```c
#include <stdint.h>
#include <string.h>

__attribute__((visibility("default")))
int memlink_init(const unsigned char* config, unsigned long config_len) {
    (void)config;
    (void)config_len;
    return 0;
}

__attribute__((visibility("default")))
int memlink_call(unsigned int method_id, const unsigned char* args,
                unsigned long args_len, unsigned char* output) {
    (void)method_id;
    // Echo input back
    if (args_len > 0 && args != NULL) {
        memcpy(output, args, args_len);
    }
    return 0;
}

__attribute__((visibility("default")))
int memlink_shutdown(void) {
    return 0;
}
```

### Build Commands

| Platform | Command |
|----------|---------|
| Linux | `cc -shared -fPIC -O2 -o my_module.so my_module.c` |
| Windows | `cl /LD my_module.c /Fe:my_module.dll` |
| macOS | `cc -shared -fPIC -O2 -o my_module.dylib my_module.c` |

See [ABI Documentation](docs/abi.md) for full specification.

---

## Advanced Usage

### Loading Multiple Modules

```rust
use memlink_runtime::runtime::{Runtime, ModuleRuntime};
use memlink_runtime::resolver::ModuleRef;
use std::sync::Arc;
use std::thread;

let runtime = Arc::new(Runtime::with_local_resolver());

// Load multiple modules
let math = runtime.load(ModuleRef::parse("./math.so")?)?;
let string = runtime.load(ModuleRef::parse("./string.so")?)?;
let crypto = runtime.load(ModuleRef::parse("./crypto.so")?)?;

// Call concurrently from different threads
let mut handles = vec![];

for (name, module) in [("math", math), ("string", string), ("crypto", crypto)] {
    let rt = Arc::clone(&runtime);
    let h = thread::spawn(move || {
        for i in 0..100 {
            rt.call(module, "process", format!("{}_{}", name, i).as_bytes()).unwrap();
        }
    });
    handles.push(h);
}

for h in handles {
    h.join().unwrap();
}
```

### Hot Reload

```rust
use memlink_runtime::reload::ReloadConfig;
use std::time::Duration;

// Reload a module with new version
let config = ReloadConfig::default()
    .with_drain_timeout(Duration::from_secs(30))
    .with_state_preservation();

let reload_state = runtime.reload_with_config(
    old_handle,
    ModuleRef::parse("./my_module_v2.so")?,
    config
)?;

// Old module drains in-flight calls before unloading
```

### Metrics and Monitoring

```rust
// Get usage statistics
let usage = runtime.get_usage(handle).unwrap();
println!("Calls: {}", usage.call_count);
println!("Arena: {} bytes ({:.2}%)", 
    usage.arena_bytes, 
    usage.arena_usage * 100.0
);

// Export Prometheus metrics
let metrics = RuntimeMetrics::new();
// ... after operations ...
println!("{}", metrics.prometheus_export());
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Your Application                      │
│                     (Memlink Runtime)                    │
├─────────────────────────────────────────────────────────┤
│  Runtime                                                │
│  ├─ Resolver (locate modules)                           │
│  ├─ Loader (load shared libs)                           │
│  ├─ Instance Manager (track loaded modules)             │
│  ├─ Panic Handler (catch module panics)                 │
│  └─ Metrics (Prometheus-compatible)                     │
└─────────────────────────────────────────────────────────┘
         │
         ├─→ [module_a.so] ── memlink_init/call/shutdown
         ├─→ [module_b.dll] ── memlink_init/call/shutdown
         └─→ [module_c.dylib] ── memlink_init/call/shutdown
```

### Components

| Component | Description |
|-----------|-------------|
| **Runtime** | High-level API for module management |
| **Resolver** | Locates and validates module files |
| **Loader** | Loads shared libraries and resolves symbols |
| **Instance** | Represents a loaded module |
| **Arena** | Fast bump allocator for module memory |
| **Metrics** | Collects and exports runtime statistics |

---

## Performance

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Module load | 92 μs | - |
| Method call (64 bytes) | 210 ns | 4.7M calls/sec (single thread) |
| Module unload | 52 μs | - |
| Hot reload | 237 μs | - |
| Concurrent calls (8 threads) | - | 2.5M calls/sec |
| Memory overhead | 0.7 MB/module | - |

**Key Highlights:**
- ✅ Sub-microsecond call latency
- ✅ Linear scalability up to 8 threads
- ✅ Fast hot reload with zero downtime
- ✅ Low memory footprint

See [Performance Benchmarks](docs/perf.md) for detailed methodology, full results, and optimization recommendations.

---

## Use Cases

### Plugin Systems

Load user-created plugins without recompiling your application:

```rust
// Load all plugins from a directory
for entry in std::fs::read_dir("./plugins")? {
    let path = entry?.path();
    if path.extension() == Some("so".as_ref()) {
        runtime.load(ModuleRef::parse(path.to_str().unwrap())?)?;
    }
}
```

### Hot-Reloadable Business Logic

Update logic in production without downtime:

```rust
// Watch for file changes
notify::Watcher::new(move |event| {
    if event.path.ends_with("business_logic.so") {
        runtime.reload(handle, ModuleRef::parse("./business_logic.so")?)?;
    }
})?;
```

### Sandboxed Execution

Isolate risky code that might panic:

```rust
// Module panics are caught and converted to errors
match runtime.call(handle, "risky_operation", data) {
    Ok(result) => println!("Success"),
    Err(Error::ModulePanicked(msg)) => eprintln!("Module panicked: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

---

## API Reference

### Core Traits

| Trait | Purpose |
|-------|---------|
| `ModuleRuntime` | Main interface for module operations |
| `ModuleResolver` | Resolve module references to artifacts |

### Key Types

| Type | Description |
|------|-------------|
| `Runtime` | Default runtime implementation |
| `ModuleHandle` | Opaque handle to loaded module |
| `ModuleRef` | Module reference (path or registry) |
| `ModuleUsage` | Usage statistics per module |
| `ReloadState` | Tracks hot-reload operations |

[Full API Documentation](https://docs.rs/memlink-runtime)

---

## Examples

Run the included examples:

```bash
# Multi-module concurrent calls
cargo run --example m2

# Build test modules first (Linux/WSL)
cd examples/modules/build
node build.js --linux
```

See [examples/modules/README.md](examples/modules/README.md) for module documentation.

---

## Platform Support

| Platform | Status | Extensions |
|----------|--------|------------|
| Linux | ✅ Tested | `.so` |
| Windows | ✅ Tested | `.dll` |
| macOS | ✅ Tested | `.dylib` |

---

## License

Memlink Runtime is licensed under the Apache License 2.0 ([LICENSE-APACHE](../LICENSE-APACHE)).

---

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Run `cargo test` and `cargo clippy`
4. Submit a pull request

### Development

```bash
# Build
cargo build

# Test
cargo test

# Lint
cargo clippy --all-targets

# Format
cargo fmt --all

# Build docs
cargo doc --open
```

---

## Related Crates

- [memlink-shm](../shm/README.md) - Shared memory IPC
- [libloading](https://crates.io/crates/libloading) - Underlying library loading
- [dashmap](https://crates.io/crates/dashmap) - Concurrent hash map

---

## Support

- **Issues**: [GitHub Issues](https://github.com/memlink/memlink/issues)
- **Documentation**: [docs.rs](https://docs.rs/memlink-runtime)
- **ABI Spec**: [docs/abi.md](docs/abi.md)
