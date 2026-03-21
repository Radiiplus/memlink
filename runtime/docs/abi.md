# Memlink Module ABI Specification

**Version:** 1.0  
**Status:** Stable  
**Last Updated:** 2026-03-21

---

## Table of Contents

1. [Overview](#overview)
2. [Required Exports](#required-exports)
3. [Optional Exports](#optional-exports)
4. [ABI Version](#abi-version)
5. [Method Dispatch](#method-dispatch)
6. [Example Module](#example-module)
7. [Building Modules](#building-modules)
8. [Threading](#threading-considerations)
9. [Error Handling](#error-handling)
10. [Memory Management](#memory-management)
11. [Security](#security-notes)

---

## Overview

The Memlink ABI defines the interface between the **memlink-runtime** and dynamically loaded modules. Modules are shared libraries (`.so`, `.dll`, `.dylib`) that export C-compatible functions for lifecycle management and method invocation.

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Memlink Runtime                        │
│  - Loads shared libraries                               │
│  - Validates ABI compatibility                          │
│  - Dispatches method calls                              │
│  - Catches panics and errors                            │
└─────────────────────────────────────────────────────────┘
                          ↓ calls
┌─────────────────────────────────────────────────────────┐
│                    Module (.so/.dll)                    │
│  - memlink_init()    → Initialize                       │
│  - memlink_call()    → Execute method                   │
│  - memlink_shutdown() → Cleanup                         │
└─────────────────────────────────────────────────────────┘
```

---

## Required Exports

All modules **must** export these three functions with exact signatures:

### `memlink_init`

Initializes the module with optional configuration.

```c
int memlink_init(const unsigned char* config, unsigned long config_len);
```

| Parameter | Description |
|-----------|-------------|
| `config` | Pointer to configuration bytes. May be `NULL` if no configuration. |
| `config_len` | Length of configuration data in bytes. |

**Returns:**
- `0` on success
- Non-zero error code on failure

**Lifecycle:**
- Called exactly once after loading
- Module must not receive calls until initialization completes
- Should allocate resources, open files, initialize state

---

### `memlink_call`

Executes a method call on the module.

```c
int memlink_call(
    unsigned int method_id,
    const unsigned char* args,
    unsigned long args_len,
    unsigned char* output
);
```

| Parameter | Description |
|-----------|-------------|
| `method_id` | 32-bit FNV-1a hash of method name |
| `args` | Pointer to input argument bytes. May be `NULL`. |
| `args_len` | Length of input arguments in bytes |
| `output` | Caller-allocated output buffer (typically 4096 bytes) |

**Returns:**
- `0` on success
- Non-zero error code on failure

**Lifecycle:**
- May be called zero or more times
- May be called concurrently from multiple threads
- Output buffer is pre-allocated by runtime; do not free

---

### `memlink_shutdown`

Shuts down the module and releases resources.

```c
int memlink_shutdown(void);
```

**Returns:**
- `0` on success
- Non-zero error code on failure

**Lifecycle:**
- Called exactly once before unloading
- Module will not receive calls after shutdown
- Should release all resources (memory, handles, etc.)

---

## Optional Exports

Modules **may** export these functions for advanced features:

### `memlink_get_state_size`

Returns the size of serialized state.

```c
unsigned long memlink_get_state_size(void);
```

**Returns:** Size in bytes of serialized state, or `0` if unsupported.

---

### `memlink_serialize_state`

Serializes module state for persistence or migration.

```c
int memlink_serialize_state(unsigned char* buffer, unsigned long buffer_size);
```

| Parameter | Description |
|-----------|-------------|
| `buffer` | Output buffer for serialized state |
| `buffer_size` | Size of output buffer in bytes |

**Returns:** `0` on success, non-zero on failure.

---

### `memlink_deserialize_state`

Restores module state from serialized data.

```c
int memlink_deserialize_state(const unsigned char* data, unsigned long data_len);
```

| Parameter | Description |
|-----------|-------------|
| `data` | Pointer to serialized state data |
| `data_len` | Length of serialized data in bytes |

**Returns:** `0` on success, non-zero on failure.

---

## ABI Version

**Current ABI Version: `1`**

The runtime validates ABI compatibility when loading modules:

| Module Version | Runtime Behavior |
|----------------|------------------|
| `1` (current) | ✓ Accepted |
| `< 1` (older) | ⚠ Warning, may work |
| `> 1` (newer) | ✗ Rejected |

---

## Method Dispatch

Methods are identified by 32-bit FNV-1a hashes:

### Hash Algorithm

```c
uint32_t fnv1a_hash(const char* str) {
    uint32_t hash = 2166136261u;  // FNV offset basis
    for (char c : str) {
        hash ^= (uint8_t)c;
        hash *= 16777619u;        // FNV prime
    }
    return hash;
}
```

### Example Hashes

| Method Name | Hash (hex) |
|-------------|------------|
| `"echo"` | `0x8c7a5f3e` |
| `"process"` | `0x4d2b1a9c` |
| `"increment"` | `0x7f3e2d1c` |

*Note: Hashes shown are examples. Use runtime's `fnv1a_hash()` function for actual values.*

---

## Example Module

### Minimal Echo Module

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
    if (args_len > 0 && args != NULL && output != NULL) {
        memcpy(output, args, args_len);
    }
    return 0;
}

__attribute__((visibility("default")))
int memlink_shutdown(void) {
    return 0;
}
```

### Stateful Counter Module

```c
#include <stdint.h>
#include <stdio.h>
#include <string.h>

static uint64_t g_counter = 0;

__attribute__((visibility("default")))
int memlink_init(const unsigned char* config, unsigned long config_len) {
    (void)config;
    (void)config_len;
    g_counter = 0;
    return 0;
}

__attribute__((visibility("default")))
int memlink_call(unsigned int method_id, const unsigned char* args,
                unsigned long args_len, unsigned char* output) {
    (void)method_id;
    (void)args;
    (void)args_len;
    
    g_counter++;
    
    char buffer[32];
    int len = snprintf(buffer, sizeof(buffer), "%lu", (unsigned long)g_counter);
    
    if (len > 0 && len < (int)sizeof(buffer)) {
        memcpy(output, buffer, len);
        return 0;
    }
    return -1;
}

__attribute__((visibility("default")))
int memlink_shutdown(void) {
    g_counter = 0;
    return 0;
}
```

---

## Building Modules

### Linux

```bash
cc -shared -fPIC -O2 -o my_module.so my_module.c
```

### Windows (MSVC)

```cmd
cl /LD my_module.c /Fe:my_module.dll
```

### Windows (MinGW)

```bash
gcc -shared -o my_module.dll my_module.c
```

### macOS

```bash
cc -shared -fPIC -O2 -o my_module.dylib my_module.c
```

### Rust Module

```rust
#[no_mangle]
pub extern "C" fn memlink_init(config: *const u8, config_len: usize) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn memlink_call(
    method_id: u32,
    args: *const u8,
    args_len: usize,
    output: *mut u8
) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn memlink_shutdown() -> i32 {
    0
}
```

Build with:
```bash
rustc --crate-type=cdylib -O my_module.rs
```

---

## Threading Considerations

- **`memlink_call` may be invoked concurrently** from multiple threads
- Modules maintaining internal state **must ensure thread safety**
- Use mutexes, atomics, or thread-local storage as needed
- The runtime does not serialize calls to the same module

### Thread-Safe Counter Example

```c
#include <stdatomic.h>

static atomic_uint_fast64_t g_counter = 0;

int memlink_call(...) {
    uint64_t count = atomic_fetch_add(&g_counter, 1) + 1;
    // ... use count
    return 0;
}
```

---

## Error Handling

### Return Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `-1` | General error |
| `-2` | Invalid arguments |
| `-3` | Internal failure |
| `1+` | Module-defined errors |

### Panic Isolation

The runtime catches panics from modules:

- Rust panics are converted to `Error::ModulePanicked`
- The runtime remains stable after module panics
- Panicking modules should be unloaded and reloaded

---

## Memory Management

### Ownership Rules

| Buffer | Owner | Notes |
|--------|-------|-------|
| `config` | Runtime | Read-only in `memlink_init` |
| `args` | Runtime | Read-only in `memlink_call` |
| `output` | Runtime | Write up to 4096 bytes |
| Internal state | Module | Free in `memlink_shutdown` |

### Guidelines

- Do not free `config`, `args`, or `output` buffers
- Allocate module state internally (malloc, Box, etc.)
- Release all allocations in `memlink_shutdown`
- Use arena allocation for high-performance modules

---

## Security Notes

### Trust Model

- **Modules run in the same process** as the runtime
- A buggy module can crash the entire process
- A malicious module can access all process memory

### Mitigations

| Risk | Mitigation |
|------|------------|
| Panics | Runtime catches and converts to errors |
| Memory corruption | Use safe languages (Rust) when possible |
| Infinite loops | Implement call timeouts in runtime |
| Resource exhaustion | Enforce memory limits per module |

### Untrusted Code

For untrusted modules, consider:

1. Running modules in **separate processes**
2. Using **OS sandboxing** (seccomp, AppContainer, etc.)
3. Implementing **resource quotas** per module

---

## See Also

- [Runtime Documentation](../README.md)
- [API Reference](https://docs.rs/memlink-runtime)
- [Example Modules](../examples/)
