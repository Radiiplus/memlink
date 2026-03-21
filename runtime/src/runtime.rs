//! High-level module runtime API.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use dashmap::DashMap;

use crate::error::{Error, Result};
use crate::ffi::loader::ModuleLoader;
use crate::instance::ModuleInstance;
use crate::profile::ModuleProfile;
use crate::reload::{ReloadConfig, ReloadState};
use crate::resolver::{ModuleRef, ModuleResolver, local::LocalResolver};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleHandle(pub u64);

#[derive(Debug, Clone, Default)]
pub struct ModuleUsage {
    pub arena_usage: f32,
    pub arena_bytes: usize,
    pub call_count: u64,
}

struct LoadedModule {
    instance: ModuleInstance,
    profile: ModuleProfile,
    call_count: AtomicU64,
    in_flight: AtomicUsize,
    draining: AtomicBool,
}

pub struct Runtime {
    resolver: Arc<dyn ModuleResolver>,
    loader: ModuleLoader,
    instances: DashMap<ModuleHandle, LoadedModule>,
    handle_counter: AtomicU64,
}

impl Runtime {
    pub fn new(resolver: Arc<dyn ModuleResolver>) -> Self {
        Runtime {
            resolver,
            loader: ModuleLoader::new(),
            instances: DashMap::new(),
            handle_counter: AtomicU64::new(1),
        }
    }

    pub fn with_local_resolver() -> Self {
        Self::new(Arc::new(LocalResolver::new()))
    }

    fn next_handle(&self) -> ModuleHandle {
        let id = self.handle_counter.fetch_add(1, Ordering::Relaxed);
        ModuleHandle(id)
    }

    pub fn loaded_count(&self) -> usize {
        self.instances.len()
    }

    pub fn is_loaded(&self, handle: ModuleHandle) -> bool {
        self.instances.contains_key(&handle)
    }

    pub fn loaded_handles(&self) -> Vec<ModuleHandle> {
        self.instances.iter().map(|r| *r.key()).collect()
    }
}

pub trait ModuleRuntime: Send + Sync {
    fn load(&self, reference: ModuleRef) -> Result<ModuleHandle>;
    fn call(&self, handle: ModuleHandle, method: &str, args: &[u8]) -> Result<Vec<u8>>;
    fn unload(&self, handle: ModuleHandle) -> Result<()>;
    fn get_usage(&self, handle: ModuleHandle) -> Option<ModuleUsage>;
    fn get_profile(&self, handle: ModuleHandle) -> Option<ModuleProfile>;
    fn reload(&self, handle: ModuleHandle, reference: ModuleRef) -> Result<ReloadState>;
    fn reload_with_config(
        &self,
        handle: ModuleHandle,
        reference: ModuleRef,
        config: ReloadConfig,
    ) -> Result<ReloadState>;
}

impl ModuleRuntime for Runtime {
    fn load(&self, reference: ModuleRef) -> Result<ModuleHandle> {
        let artifact = self.resolver.resolve(reference)?;
        let instance = self.loader.load(artifact)?;

        let path_str = instance.path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let profile = ModuleProfile::new(path_str);
        let handle = self.next_handle();

        self.instances.insert(handle, LoadedModule {
            instance,
            profile,
            call_count: AtomicU64::new(0),
            in_flight: AtomicUsize::new(0),
            draining: AtomicBool::new(false),
        });

        Ok(handle)
    }

    fn call(&self, handle: ModuleHandle, method: &str, args: &[u8]) -> Result<Vec<u8>> {
        let module = self.instances.get(&handle)
            .ok_or_else(|| Error::FileNotFound(
                std::path::PathBuf::from(format!("Module handle {}", handle.0))
            ))?;

        if module.draining.load(Ordering::Relaxed) {
            return Err(Error::ReloadInProgress(handle.0));
        }

        module.in_flight.fetch_add(1, Ordering::Relaxed);
        module.call_count.fetch_add(1, Ordering::Relaxed);

        let result = module.instance.call(method, args);

        module.in_flight.fetch_sub(1, Ordering::Relaxed);

        result
    }

    fn unload(&self, handle: ModuleHandle) -> Result<()> {
        let removed = self.instances.remove(&handle);

        if removed.is_some() {
            Ok(())
        } else {
            Err(Error::FileNotFound(
                std::path::PathBuf::from(format!("Module handle {}", handle.0))
            ))
        }
    }

    fn get_usage(&self, handle: ModuleHandle) -> Option<ModuleUsage> {
        let module = self.instances.get(&handle)?;

        let arena = module.instance.arena();
        let usage = arena.usage();
        let arena_bytes = arena.used();
        let call_count = module.call_count.load(Ordering::Relaxed);

        Some(ModuleUsage {
            arena_usage: usage,
            arena_bytes,
            call_count,
        })
    }

    fn get_profile(&self, handle: ModuleHandle) -> Option<ModuleProfile> {
        let module = self.instances.get(&handle)?;
        Some(module.profile.clone())
    }

    fn reload(&self, handle: ModuleHandle, reference: ModuleRef) -> Result<ReloadState> {
        self.reload_with_config(handle, reference, ReloadConfig::default())
    }

    fn reload_with_config(
        &self,
        handle: ModuleHandle,
        reference: ModuleRef,
        config: ReloadConfig,
    ) -> Result<ReloadState> {
        let old_module = self.instances.get(&handle)
            .ok_or_else(|| Error::FileNotFound(
                std::path::PathBuf::from(format!("Module handle {}", handle.0))
            ))?;

        if old_module.draining.load(Ordering::Relaxed) {
            return Err(Error::ReloadInProgress(handle.0));
        }

        let in_flight_count = old_module.in_flight.load(Ordering::Relaxed);
        old_module.draining.store(true, Ordering::Relaxed);

        let new_artifact = self.resolver.resolve(reference)?;
        let new_instance = self.loader.load(new_artifact)?;

        let path_str = new_instance.path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let new_profile = ModuleProfile::new(path_str);

        let new_handle = self.next_handle();

        self.instances.insert(new_handle, LoadedModule {
            instance: new_instance,
            profile: new_profile,
            call_count: AtomicU64::new(0),
            in_flight: AtomicUsize::new(0),
            draining: AtomicBool::new(false),
        });

        let reload_state = ReloadState::new(handle.0, new_handle.0, in_flight_count);

        if in_flight_count > 0 {
            reload_state.wait_for_drain(config.drain_timeout)?;
        }

        self.instances.remove(&handle);

        Ok(reload_state)
    }
}
