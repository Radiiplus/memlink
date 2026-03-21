//! Hot-reload support for module reloading.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use crate::error::{Error, Result};
use crate::instance::ModuleInstance;

pub const DEFAULT_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub struct ReloadState {
    pub old_handle: u64,
    pub new_handle: u64,
    pub draining: bool,
    in_flight: Arc<AtomicUsize>,
    drain_complete: Arc<AtomicBool>,
}

impl ReloadState {
    pub fn new(old_handle: u64, new_handle: u64, in_flight_count: usize) -> Self {
        let in_flight = Arc::new(AtomicUsize::new(in_flight_count));
        let drain_complete = Arc::new(AtomicBool::new(false));

        if in_flight_count == 0 {
            drain_complete.store(true, Ordering::Relaxed);
        }

        ReloadState {
            old_handle,
            new_handle,
            draining: true,
            in_flight,
            drain_complete,
        }
    }

    pub fn in_flight_count(&self) -> usize {
        self.in_flight.load(Ordering::Relaxed)
    }

    pub fn is_drain_complete(&self) -> bool {
        self.drain_complete.load(Ordering::Relaxed)
    }

    pub fn mark_call_started(&self) {
        self.in_flight.fetch_add(1, Ordering::Relaxed);
        self.drain_complete.store(false, Ordering::Relaxed);
    }

    pub fn mark_call_completed(&self) {
        if self.in_flight.fetch_sub(1, Ordering::Relaxed) == 1 {
            self.drain_complete.store(true, Ordering::Relaxed);
        }
    }

    pub fn wait_for_drain(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if self.is_drain_complete() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        Err(Error::ReloadTimeout(self.in_flight_count()))
    }
}

#[derive(Debug, Clone)]
pub struct ModuleState {
    pub data: Vec<u8>,
    pub abi_version: u32,
}

impl ModuleState {
    pub fn new(data: Vec<u8>, abi_version: u32) -> Self {
        ModuleState { data, abi_version }
    }
}

pub trait StatefulModule {
    fn serialize_state(&self) -> Result<Vec<u8>>;
    fn restore_state(&self, _state: &[u8]) -> Result<()>;
}

impl StatefulModule for ModuleInstance {
    fn serialize_state(&self) -> Result<Vec<u8>> {
        Err(Error::UnsupportedReference)
    }

    fn restore_state(&self, _state: &[u8]) -> Result<()> {
        Err(Error::UnsupportedReference)
    }
}

#[derive(Debug, Clone)]
pub struct ReloadConfig {
    pub drain_timeout: Duration,
    pub preserve_state: bool,
    pub force_unload_on_timeout: bool,
}

impl Default for ReloadConfig {
    fn default() -> Self {
        ReloadConfig {
            drain_timeout: DEFAULT_DRAIN_TIMEOUT,
            preserve_state: false,
            force_unload_on_timeout: true,
        }
    }
}

impl ReloadConfig {
    pub fn with_drain_timeout(mut self, timeout: Duration) -> Self {
        self.drain_timeout = timeout;
        self
    }

    pub fn with_state_preservation(mut self) -> Self {
        self.preserve_state = true;
        self
    }

    pub fn with_force_unload(mut self, force: bool) -> Self {
        self.force_unload_on_timeout = force;
        self
    }
}
