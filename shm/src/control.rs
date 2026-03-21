//! Control region for shared memory coordination between daemon and clients.
//! Contains head/tail pointers, sequence numbers, futex words, and state flags.

use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU64, Ordering};

pub const CONTROL_REGION_SIZE: usize = 4096;

const DATA_SIZE: usize = std::mem::size_of::<ControlRegionData>();

#[repr(C)]
pub struct ControlRegionData {
    pub client_head: AtomicU64,
    pub daemon_tail: AtomicU64,
    pub client_seq: AtomicU64,
    pub daemon_seq: AtomicU64,
    pub client_futex: AtomicU32,
    pub daemon_futex: AtomicU32,
    pub daemon_alive: AtomicBool,
    pub client_count: AtomicU32,
    pub backpressure: AtomicU32,
    pub version: AtomicU16,
    pub flags: AtomicU16,
}

#[repr(C, align(4096))]
pub struct ControlRegion {
    pub data: ControlRegionData,
    padding: [u8; CONTROL_REGION_SIZE - DATA_SIZE],
}

const _: () = assert!(
    std::mem::size_of::<ControlRegion>() == CONTROL_REGION_SIZE,
    "ControlRegion must be exactly 4096 bytes"
);

const _: () = assert!(
    std::mem::align_of::<ControlRegion>() >= 4096,
    "ControlRegion must be page-aligned (4096 bytes)"
);

impl ControlRegion {
    /// Initialize a new control region with default values
    ///
    /// # Safety
    ///
    /// This function uses `std::ptr::write` to initialize the structure.
    /// The caller must ensure:
    /// - `this` points to properly allocated and aligned memory
    /// - The memory is writable and not accessed by other threads during initialization
    /// - This is called only once before any other access
    pub unsafe fn init(this: *mut Self) {
        std::ptr::write(
            this,
            Self {
                data: ControlRegionData {
                    client_head: AtomicU64::new(0),
                    daemon_tail: AtomicU64::new(0),
                    client_seq: AtomicU64::new(0),
                    daemon_seq: AtomicU64::new(0),
                    client_futex: AtomicU32::new(0),
                    daemon_futex: AtomicU32::new(0),
                    daemon_alive: AtomicBool::new(false),
                    client_count: AtomicU32::new(0),
                    backpressure: AtomicU32::new(0),
                    version: AtomicU16::new(1),
                    flags: AtomicU16::new(0),
                },
                padding: [0u8; CONTROL_REGION_SIZE - DATA_SIZE],
            },
        );
    }

    pub fn backpressure(&self) -> f32 {
        self.data.backpressure.load(Ordering::Acquire) as f32 / 1000.0
    }

    pub fn set_backpressure(&self, value: f32) {
        let clamped = value.clamp(0.0, 1.0);
        let scaled = (clamped * 1000.0) as u32;
        self.data.backpressure.store(scaled, Ordering::Release);
    }

    pub fn increment_client_count(&self) -> u32 {
        self.data.client_count.fetch_add(1, Ordering::AcqRel) + 1
    }

    pub fn decrement_client_count(&self) -> u32 {
        self.data.client_count
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_sub(1)
            })
            .unwrap_or(0)
    }

    pub fn set_daemon_alive(&self, alive: bool) {
        self.data.daemon_alive.store(alive, Ordering::Release);
    }

    pub fn is_daemon_alive(&self) -> bool {
        self.data.daemon_alive.load(Ordering::Acquire)
    }

    pub fn client_count(&self) -> u32 {
        self.data.client_count.load(Ordering::Acquire)
    }

    pub fn version(&self) -> u16 {
        self.data.version.load(Ordering::Acquire)
    }

    pub fn client_head(&self) -> u64 {
        self.data.client_head.load(Ordering::Acquire)
    }

    pub fn set_client_head(&self, pos: u64) {
        self.data.client_head.store(pos, Ordering::Release);
    }

    pub fn daemon_tail(&self) -> u64 {
        self.data.daemon_tail.load(Ordering::Acquire)
    }

    pub fn set_daemon_tail(&self, pos: u64) {
        self.data.daemon_tail.store(pos, Ordering::Release);
    }

    pub fn client_seq(&self) -> u64 {
        self.data.client_seq.load(Ordering::Acquire)
    }

    pub fn increment_client_seq(&self) -> u64 {
        self.data.client_seq.fetch_add(1, Ordering::AcqRel) + 1
    }

    pub fn daemon_seq(&self) -> u64 {
        self.data.daemon_seq.load(Ordering::Acquire)
    }

    pub fn increment_daemon_seq(&self) -> u64 {
        self.data.daemon_seq.fetch_add(1, Ordering::AcqRel) + 1
    }

    pub fn client_futex(&self) -> u32 {
        self.data.client_futex.load(Ordering::Acquire)
    }

    pub fn set_client_futex(&self, val: u32) {
        self.data.client_futex.store(val, Ordering::Release);
    }

    pub fn daemon_futex(&self) -> u32 {
        self.data.daemon_futex.load(Ordering::Acquire)
    }

    pub fn set_daemon_futex(&self, val: u32) {
        self.data.daemon_futex.store(val, Ordering::Release);
    }
}
