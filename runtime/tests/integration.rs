use memlink_runtime::runtime::{ModuleRuntime, Runtime};
use memlink_runtime::resolver::ModuleRef;
use std::sync::Arc;
use std::path::Path;

#[cfg(target_os = "linux")]
mod test_modules {
    use super::*;
    use std::process::Command;
    use std::io::Write;

    pub fn build_echo_module(path: &Path) -> std::io::Result<()> {
        let mut child = Command::new("cc")
            .args([
                "-shared",
                "-fPIC",
                "-O2",
                "-o",
                path.to_str().unwrap(),
                "-xc",
                "-",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        child.stdin.as_mut().unwrap().write_all(
            br#"
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
            "#,
        )?;
        child.wait()?;
        Ok(())
    }

    pub fn build_panic_module(path: &Path) -> std::io::Result<()> {
        let mut child = Command::new("cc")
            .args([
                "-shared",
                "-fPIC",
                "-O2",
                "-o",
                path.to_str().unwrap(),
                "-xc",
                "-",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        child.stdin.as_mut().unwrap().write_all(
            br#"
            #include <stdint.h>
            #include <string.h>
            #include <stdlib.h>

            __attribute__((visibility("default")))
            int memlink_init(const unsigned char* config, unsigned long config_len) {
                (void)config;
                (void)config_len;
                return 0;
            }

            __attribute__((visibility("default")))
            int memlink_call(unsigned int method_id, const unsigned char* args,
                            unsigned long args_len, unsigned char* output) {
                (void)args;
                (void)args_len;
                (void)output;

                return -1;
            }

            __attribute__((visibility("default")))
            int memlink_shutdown(void) {
                return 0;
            }
            "#,
        )?;
        child.wait()?;
        Ok(())
    }

    pub fn build_counter_module(path: &Path) -> std::io::Result<()> {
        let mut child = Command::new("cc")
            .args([
                "-shared",
                "-fPIC",
                "-O2",
                "-o",
                path.to_str().unwrap(),
                "-xc",
                "-",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        child.stdin.as_mut().unwrap().write_all(
            br#"
            #include <stdint.h>
            #include <string.h>
            #include <stdio.h>

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
            "#,
        )?;
        child.wait()?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
use test_modules::*;

#[cfg(not(target_os = "linux"))]
mod test_modules {
    use super::*;

    pub fn build_echo_module(_path: &Path) -> std::io::Result<()> {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Not supported on this platform"))
    }

    pub fn build_panic_module(_path: &Path) -> std::io::Result<()> {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Not supported on this platform"))
    }

    pub fn build_counter_module(_path: &Path) -> std::io::Result<()> {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Not supported on this platform"))
    }
}

#[cfg(not(target_os = "linux"))]
use test_modules::*;

#[test]
#[cfg_attr(not(target_os = "linux"), ignore = "Test modules only built on Linux")]
fn test_echo_roundtrip() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let module_path = dir.path().join("libecho.so");
    build_echo_module(&module_path).expect("Failed to build echo module");

    let runtime = Runtime::with_local_resolver();
    let reference = ModuleRef::parse(module_path.to_str().unwrap()).unwrap();

    let handle = runtime.load(reference).expect("Failed to load echo module");

    let input = b"hello world";
    let result = runtime.call(handle, "echo", input)
        .expect("Call failed");

    assert_eq!(&result[..input.len()], input);

    runtime.unload(handle).expect("Failed to unload");
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore = "Test modules only built on Linux")]
fn test_counter_state() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let module_path = dir.path().join("libcounter.so");
    build_counter_module(&module_path).expect("Failed to build counter module");

    let runtime = Runtime::with_local_resolver();
    let reference = ModuleRef::parse(module_path.to_str().unwrap()).unwrap();

    let handle = runtime.load(reference).expect("Failed to load counter module");

    let result1 = runtime.call(handle, "increment", b"")
        .expect("Call failed");

    let result2 = runtime.call(handle, "increment", b"")
        .expect("Call failed");

    let output1 = String::from_utf8_lossy(&result1[..result1.iter().position(|&b| b == 0).unwrap_or(result1.len())]);
    let output2 = String::from_utf8_lossy(&result2[..result2.iter().position(|&b| b == 0).unwrap_or(result2.len())]);

    assert_eq!(output1.trim_end_matches('\0'), "1");
    assert_eq!(output2.trim_end_matches('\0'), "2");

    runtime.unload(handle).expect("Failed to unload");
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore = "Test modules only built on Linux")]
fn test_panic_isolation() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let panic_path = dir.path().join("libpanic.so");
    build_panic_module(&panic_path).expect("Failed to build panic module");

    let runtime = Runtime::with_local_resolver();
    let reference = ModuleRef::parse(panic_path.to_str().unwrap()).unwrap();

    let handle = runtime.load(reference).expect("Failed to load panic module");

    let result = runtime.call(handle, "panic_now", b"");

    assert!(result.is_err(), "Panic was not caught!");

    let echo_path = dir.path().join("libecho.so");
    build_echo_module(&echo_path).expect("Failed to build echo module");

    let echo_ref = ModuleRef::parse(echo_path.to_str().unwrap()).unwrap();
    let echo_handle = runtime.load(echo_ref).expect("Failed to load echo after panic");

    let result = runtime.call(echo_handle, "test", b"hello");
    assert!(result.is_ok(), "Runtime not functional after panic: {:?}", result);

    runtime.unload(echo_handle).ok();
    runtime.unload(handle).ok();
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore = "Test modules only built on Linux")]
fn test_concurrent_calls() {
    use std::thread;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let module_path = dir.path().join("libecho.so");
    build_echo_module(&module_path).expect("Failed to build echo module");

    let runtime = Arc::new(Runtime::with_local_resolver());
    let reference = ModuleRef::parse(module_path.to_str().unwrap()).unwrap();

    let handle = runtime.load(reference).expect("Failed to load echo module");

    let mut handles = vec![];

    for thread_id in 0..10 {
        let runtime_clone = Arc::clone(&runtime);
        let handle_clone = handle;

        let h = thread::spawn(move || {
            for i in 0..10 {
                let input = format!("thread{}_msg{}", thread_id, i);
                let result = runtime_clone.call(handle_clone, "echo", input.as_bytes());
                assert!(result.is_ok(), "Call failed in thread {}", thread_id);
            }
        });

        handles.push(h);
    }

    for h in handles {
        h.join().expect("Thread panicked");
    }

    runtime.unload(handle).expect("Failed to unload");
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore = "Test modules only built on Linux")]
fn test_multiple_instances() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let module_path = dir.path().join("libecho.so");
    build_echo_module(&module_path).expect("Failed to build echo module");

    let runtime = Runtime::with_local_resolver();
    let reference = ModuleRef::parse(module_path.to_str().unwrap()).unwrap();

    let handle_a = runtime.load(reference.clone()).expect("Failed to load instance A");
    let handle_b = runtime.load(reference).expect("Failed to load instance B");

    assert_ne!(handle_a.0, handle_b.0);

    let result_a = runtime.call(handle_a, "echo", b"instance_a")
        .expect("Call A failed");
    let result_b = runtime.call(handle_b, "echo", b"instance_b")
        .expect("Call B failed");

    assert_eq!(&result_a[..10], b"instance_a");
    assert_eq!(&result_b[..10], b"instance_b");

    runtime.unload(handle_a).expect("Failed to unload A");

    let result_b2 = runtime.call(handle_b, "echo", b"still_working")
        .expect("Call B failed after A unloaded");
    assert_eq!(&result_b2[..13], b"still_working");

    runtime.unload(handle_b).expect("Failed to unload B");
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore = "Test modules only built on Linux")]
fn test_error_propagation() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let module_path = dir.path().join("libecho.so");
    build_echo_module(&module_path).expect("Failed to build echo module");

    let runtime = Runtime::with_local_resolver();
    let reference = ModuleRef::parse(module_path.to_str().unwrap()).unwrap();

    let handle = runtime.load(reference).expect("Failed to load");

    let result = runtime.call(handle, "echo", b"test");
    assert!(result.is_ok());

    runtime.unload(handle).expect("Failed to unload");

    let result = runtime.call(handle, "echo", b"test");
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, memlink_runtime::Error::FileNotFound(_)));
}

#[test]
fn test_runtime_load_call_unload_cycle() {
    let runtime = Runtime::with_local_resolver();

    let reference = ModuleRef::parse("./nonexistent_module.so").unwrap();
    let result = runtime.load(reference);

    assert!(matches!(result, Err(memlink_runtime::Error::FileNotFound(_))));
}
