//! Crash recovery, stale slot cleanup, and daemon liveness monitoring.
//! Uses slot state machine, timestamps, heartbeats, and PID files.

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread;
use std::fs;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    Empty = 0,
    Writing = 1,
    Ready = 2,
    Reading = 3,
    Done = 4,
}

impl SlotState {
    fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(SlotState::Empty),
            1 => Some(SlotState::Writing),
            2 => Some(SlotState::Ready),
            3 => Some(SlotState::Reading),
            4 => Some(SlotState::Done),
            _ => None,
        }
    }

    fn as_u8(self) -> u8 {
        self as u8
    }
}

pub struct AtomicSlotState {
    inner: AtomicU8,
}

impl AtomicSlotState {
    pub fn new(state: SlotState) -> Self {
        Self {
            inner: AtomicU8::new(state.as_u8()),
        }
    }

    pub fn load(&self) -> SlotState {
        SlotState::from_u8(self.inner.load(Ordering::Acquire))
            .unwrap_or(SlotState::Empty)
    }

    pub fn store(&self, state: SlotState) {
        self.inner.store(state.as_u8(), Ordering::Release);
    }

    pub fn compare_exchange(
        &self,
        current: SlotState,
        new: SlotState,
    ) -> Result<SlotState, SlotState> {
        match self.inner.compare_exchange(
            current.as_u8(),
            new.as_u8(),
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(val) => Ok(SlotState::from_u8(val).unwrap_or(SlotState::Empty)),
            Err(val) => Err(SlotState::from_u8(val).unwrap_or(SlotState::Empty)),
        }
    }
}

#[repr(C)]
pub struct SlotMetadata {
    pub state: AtomicSlotState,
    pub timestamp: AtomicU64,
    pub sequence: AtomicU64,
}

impl SlotMetadata {
    pub fn new() -> Self {
        Self {
            state: AtomicSlotState::new(SlotState::Empty),
            timestamp: AtomicU64::new(current_timestamp()),
            sequence: AtomicU64::new(0),
        }
    }

    pub fn update_timestamp(&self) {
        self.timestamp.store(current_timestamp(), Ordering::Release);
    }

    pub fn age_seconds(&self) -> u64 {
        let current = current_timestamp();
        let stored = self.timestamp.load(Ordering::Acquire);
        current.saturating_sub(stored)
    }

    pub fn is_stale(&self, timeout_seconds: u64) -> bool {
        let state = self.state.load();
        match state {
            SlotState::Writing | SlotState::Reading => {
                self.age_seconds() > timeout_seconds
            }
            _ => false,
        }
    }

    pub fn recover_stale(&self, timeout_seconds: u64) -> bool {
        if self.is_stale(timeout_seconds) {
            self.state.store(SlotState::Empty);
            self.update_timestamp();
            true
        } else {
            false
        }
    }
}

impl Default for SlotMetadata {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

pub struct Heartbeat {
    timestamp: AtomicU64,
    interval: u64,
    active: AtomicU8,
}

impl Heartbeat {
    pub fn new(interval_seconds: u64) -> Self {
        Self {
            timestamp: AtomicU64::new(current_timestamp()),
            interval: interval_seconds,
            active: AtomicU8::new(1),
        }
    }

    pub fn beat(&self) {
        self.timestamp.store(current_timestamp(), Ordering::Release);
    }

    pub fn is_alive(&self, timeout_seconds: u64) -> bool {
        if self.active.load(Ordering::Acquire) == 0 {
            return false;
        }
        let current = current_timestamp();
        let last = self.timestamp.load(Ordering::Acquire);
        current.saturating_sub(last) <= timeout_seconds
    }

    pub fn stop(&self) {
        self.active.store(0, Ordering::Release);
    }

    pub fn start_monitoring(
        self: &Arc<Self>,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> thread::JoinHandle<()> {
        let heartbeat = Arc::clone(self);
        thread::spawn(move || {
            while heartbeat.active.load(Ordering::Acquire) == 1 {
                thread::sleep(Duration::from_secs(heartbeat.interval));
                if !heartbeat.is_alive(heartbeat.interval * 3) {
                    callback();
                    break;
                }
            }
        })
    }
}

pub struct RecoveryManager {
    pid_path: String,
    heartbeat: Arc<Heartbeat>,
    active: AtomicU8,
}

impl RecoveryManager {
    pub fn new(_shm_path: &str) -> Self {
        let pid_path = format!("{}.pid", _shm_path);
        Self {
            pid_path,
            heartbeat: Arc::new(Heartbeat::new(1)),
            active: AtomicU8::new(0),
        }
    }

    pub fn is_already_running(&self) -> bool {
        if let Ok(pid_str) = fs::read_to_string(&self.pid_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if process_exists(pid) {
                    return true;
                }
            }
        }
        false
    }

    pub fn register_daemon(&self) -> Result<(), String> {
        if self.is_already_running() {
            return Err("Another daemon is already running".to_string());
        }

        let pid = std::process::id();
        fs::write(&self.pid_path, pid.to_string())
            .map_err(|e| format!("Failed to write PID file: {}", e))?;

        self.active.store(1, Ordering::Release);
        self.heartbeat.beat();
        Ok(())
    }

    pub fn unregister_daemon(&self) {
        self.active.store(0, Ordering::Release);
        self.heartbeat.stop();
        let _ = fs::remove_file(&self.pid_path);
    }

    pub fn heartbeat(&self) -> &Arc<Heartbeat> {
        &self.heartbeat
    }

    pub fn recover_stale_slots(
        &self,
        slots: &[SlotMetadata],
        timeout_seconds: u64,
    ) -> usize {
        let mut recovered = 0;
        for slot in slots {
            if slot.recover_stale(timeout_seconds) {
                recovered += 1;
            }
        }
        recovered
    }

    pub fn cleanup_orphaned_shm(path: &str) -> Result<bool, String> {
        let pid_path = format!("{}.pid", path);

        if let Ok(pid_str) = fs::read_to_string(&pid_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if !process_exists(pid) {
                    let _ = fs::remove_file(&pid_path);
                    let _ = fs::remove_file(path);
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

impl Drop for RecoveryManager {
    fn drop(&mut self) {
        self.unregister_daemon();
    }
}

fn process_exists(pid: u32) -> bool {
    #[cfg(unix)]
    {
        unsafe {
            libc::kill(pid as libc::pid_t, 0) == 0
        }
    }
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION,
        };

        match unsafe {
            OpenProcess(PROCESS_QUERY_INFORMATION, false, pid)
        } {
            Ok(handle) => {
                let _ = unsafe { CloseHandle(handle) };
                true
            }
            Err(_) => false,
        }
    }
}
