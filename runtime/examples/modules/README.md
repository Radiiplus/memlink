# Memlink Runtime Example Modules

This directory contains example C modules that demonstrate the memlink module ABI.

## Modules

| Module | Description |
|--------|-------------|
| `math.c` | Basic arithmetic (addition) |
| `string.c` | String manipulation (uppercase) |
| `crypto.c` | Simple hash function (FNV-1a) |
| `echo.c` | Echo input back |

## Building

### Using Node.js Build Script

The build script compiles modules for both Windows (.dll) and Linux (.so):

```bash
# Build for current platform
node build/build.js

# Build for all platforms (Windows + Linux via WSL)
node build/build.js --all

# Build only Linux modules
node build/build.js --linux

# Build only Windows modules
node build/build.js --windows

# Clean built files
node build/build.js --clean
```

### Manual Build

#### Linux (or WSL)
```bash
gcc -shared -fPIC -O2 -o libmath.so math.c
gcc -shared -fPIC -O2 -o libstring.so string.c
gcc -shared -fPIC -O2 -o libcrypto.so crypto.c
gcc -shared -fPIC -O2 -o libecho.so echo.c
```

#### Windows (MSVC)
```cmd
cl /LD math.c /Fe:math.dll
cl /LD string.c /Fe:string.dll
cl /LD crypto.c /Fe:crypto.dll
cl /LD echo.c /Fe:echo.dll
```

#### Windows (MinGW-w64)
```bash
gcc -shared -o math.dll math.c -Wl,--add-stdcall-alias
gcc -shared -o string.dll string.c -Wl,--add-stdcall-alias
gcc -shared -o crypto.dll crypto.c -Wl,--add-stdcall-alias
gcc -shared -o echo.dll echo.c -Wl,--add-stdcall-alias
```

#### macOS
```bash
cc -shared -fPIC -O2 -o libmath.dylib math.c
cc -shared -fPIC -O2 -o libstring.dylib string.c
cc -shared -fPIC -O2 -o libcrypto.dylib crypto.c
cc -shared -fPIC -O2 -o libecho.dylib echo.c
```

## Testing

After building, run the example:

```bash
# From the runtime directory
cargo run --example m2
```

## Module ABI

Each module exports three required functions:

```c
int memlink_init(const unsigned char* config, unsigned long config_len);
int memlink_call(unsigned int method_id, const unsigned char* args,
                unsigned long args_len, unsigned char* output);
int memlink_shutdown(void);
```

See [docs/abi.md](../../docs/abi.md) for full ABI specification.
