//! Multi-module example for Windows (.dll files).

use memlink_runtime::runtime::{ModuleRuntime, Runtime};
use memlink_runtime::resolver::ModuleRef;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MemLink Runtime: Windows DLL Test ===\n");

    let runtime = Arc::new(Runtime::with_local_resolver());

    println!("Step 1: Loading DLL modules...");

    let modules = vec![
        ("math", "./examples/modules/math.dll"),
        ("string", "./examples/modules/string.dll"),
        ("crypto", "./examples/modules/crypto.dll"),
        ("echo", "./examples/modules/echo.dll"),
    ];

    let mut handles = Vec::new();

    for (name, path) in &modules {
        match ModuleRef::parse(path) {
            Ok(reference) => match runtime.load(reference) {
                Ok(handle) => {
                    println!("  ✓ Loaded '{}' from '{}'", name, path);
                    handles.push((name, handle));
                }
                Err(e) => {
                    println!("  ✗ Failed to load '{}': {}", name, e);
                }
            },
            Err(e) => {
                println!("  ✗ Failed to parse path '{}': {}", path, e);
            }
        }
    }

    println!("\nStep 2: Module count = {}", runtime.loaded_count());

    if handles.is_empty() {
        println!("\nNo modules loaded.");
        println!("Build test DLLs with: cd examples/modules && gcc -shared -o math.dll math.c ...");
        return Ok(());
    }

    println!("\nStep 3: Calling modules concurrently...");

    let mut join_handles = Vec::new();

    for (name, handle) in &handles {
        let rt = Arc::clone(&runtime);
        let handle_clone = *handle;
        let name_clone = name.to_string();

        let h = thread::spawn(move || {
            for i in 0..5 {
                let method = "process";
                let args = format!("{}_call_{}", name_clone, i).into_bytes();

                match rt.call(handle_clone, method, &args) {
                    Ok(result) => {
                        println!(
                            "  [Thread {:?}] {} -> {} bytes",
                            thread::current().id(),
                            name_clone,
                            result.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("  [Thread {:?}] {} error: {}", thread::current().id(), name_clone, e);
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        });

        join_handles.push(h);
    }

    for h in join_handles {
        h.join().expect("Thread panicked");
    }

    println!("\nStep 4: Module usage statistics:");

    for (name, handle) in &handles {
        if let Some(usage) = runtime.get_usage(*handle) {
            println!(
                "  {}: {} calls, {:.2}% arena usage, {} bytes",
                name, usage.call_count, usage.arena_usage * 100.0, usage.arena_bytes
            );
        }
    }

    println!("\nStep 5: Unloading modules...");

    for (name, handle) in handles {
        runtime.unload(handle)?;
        println!("  ✓ Unloaded '{}'", name);
    }

    println!("\nStep 6: Final module count = {}", runtime.loaded_count());
    println!("\n=== DLL Test Complete ===");

    Ok(())
}
