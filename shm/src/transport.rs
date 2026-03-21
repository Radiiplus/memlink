//! High-level shared memory transport trait with daemon-client architecture.
//! Provides priority-based messaging, futex signaling, and connection management.

use crate::buffer::RingBufferError;
use crate::control::ControlRegion;
use crate::futex::{Futex, FutexError};
use crate::mmap::MmapSegment;
use crate::priority::Priority;
use crate::pring::PriorityRingBuffer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShmError {
    Disconnected,
    Timeout,
    BufferFull,
    ProtocolMismatch,
    InvalidState,
    MessageTooLarge,
    Other(&'static str),
}

impl std::fmt::Display for ShmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShmError::Disconnected => write!(f, "Shared memory disconnected"),
            ShmError::Timeout => write!(f, "Operation timed out"),
            ShmError::BufferFull => write!(f, "Buffer is full"),
            ShmError::ProtocolMismatch => write!(f, "Protocol version mismatch"),
            ShmError::InvalidState => write!(f, "Invalid state for operation"),
            ShmError::MessageTooLarge => write!(f, "Message too large"),
            ShmError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for ShmError {}

impl From<RingBufferError> for ShmError {
    fn from(err: RingBufferError) -> Self {
        match err {
            RingBufferError::Full => ShmError::BufferFull,
            RingBufferError::Empty => ShmError::InvalidState,
            RingBufferError::InvalidState => ShmError::InvalidState,
            RingBufferError::DataTooLarge => ShmError::MessageTooLarge,
        }
    }
}

impl From<FutexError> for ShmError {
    fn from(err: FutexError) -> Self {
        match err {
            FutexError::Timeout => ShmError::Timeout,
            FutexError::Interrupted => ShmError::InvalidState,
            FutexError::InvalidArgument => ShmError::InvalidState,
            FutexError::Unsupported => ShmError::Other("Futex unsupported"),
            FutexError::Other(_) => ShmError::Other("Futex error"),
        }
    }
}

pub type ShmResult<T> = Result<T, ShmError>;

pub trait ShmTransport {
    fn write(&self, priority: Priority, data: &[u8]) -> ShmResult<()>;
    fn read(&self) -> ShmResult<(Priority, Vec<u8>)>;
    fn signal(&self);
    fn wait(&self, timeout: Option<Duration>) -> ShmResult<()>;
    fn is_connected(&self) -> bool;
    fn protocol_version(&self) -> u16;
}

pub struct NrelayShmTransport {
    #[allow(dead_code)]
    mmap: MmapSegment,
    control: *const ControlRegion,
    ring: PriorityRingBuffer,
    futex: Futex,
    is_daemon: bool,
    connected: AtomicBool,
    expected_version: u16,
}

unsafe impl Send for NrelayShmTransport {}
unsafe impl Sync for NrelayShmTransport {}

impl NrelayShmTransport {
    pub fn create(path: &str, size: usize, version: u16) -> ShmResult<Self> {
        if size < 8192 {
            return Err(ShmError::InvalidState);
        }

        let mut mmap = MmapSegment::create(path, size)
            .map_err(|_| ShmError::Other("Failed to create mmap"))?;

        let control_ptr = mmap.as_slice().as_ptr() as *const ControlRegion;
        let control_mut = mmap.as_mut_slice().as_mut_ptr() as *mut ControlRegion;

        unsafe {
            ControlRegion::init(control_mut);
            (*control_mut).data.version.store(version, Ordering::Release);
            (*control_mut).data.daemon_alive.store(true, Ordering::Release);
            let _ = mmap.flush();
        }

        let ring = PriorityRingBuffer::new(256)
            .map_err(|_| ShmError::Other("Failed to create ring buffer"))?;

        let futex = Futex::new(0);

        Ok(Self {
            mmap,
            control: control_ptr,
            ring,
            futex,
            is_daemon: true,
            connected: AtomicBool::new(true),
            expected_version: version,
        })
    }

    pub fn connect(path: &str, version: u16) -> ShmResult<Self> {
        let mmap = MmapSegment::open(path, 8192)
            .map_err(|_| ShmError::Disconnected)?;

        let control_ptr = mmap.as_slice().as_ptr() as *const ControlRegion;

        unsafe {
            let actual_version = (*control_ptr).data.version.load(Ordering::Acquire);
            if actual_version != version {
                return Err(ShmError::ProtocolMismatch);
            }
        }

        let ring = PriorityRingBuffer::new(256)
            .map_err(|_| ShmError::Other("Failed to create ring buffer"))?;

        let futex = Futex::new(0);

        unsafe {
            (*control_ptr).data.client_count.fetch_add(1, Ordering::AcqRel);
        }

        Ok(Self {
            mmap,
            control: control_ptr,
            ring,
            futex,
            is_daemon: false,
            connected: AtomicBool::new(true),
            expected_version: version,
        })
    }

    pub fn backpressure(&self) -> f32 {
        unsafe {
            (*self.control).backpressure()
        }
    }

    pub fn set_backpressure(&self, value: f32) {
        unsafe {
            (*self.control).set_backpressure(value);
        }
    }

    pub fn is_daemon_alive(&self) -> bool {
        unsafe {
            (*self.control).data.daemon_alive.load(Ordering::Acquire)
        }
    }

    pub fn client_count(&self) -> u32 {
        unsafe {
            (*self.control).data.client_count.load(Ordering::Acquire)
        }
    }

    pub fn shutdown(&self) {
        if self.is_daemon {
            unsafe {
                (*self.control).data.daemon_alive.store(false, Ordering::Release);
            }
        }
        self.connected.store(false, Ordering::Release);
    }
}

impl ShmTransport for NrelayShmTransport {
    fn write(&self, priority: Priority, data: &[u8]) -> ShmResult<()> {
        if !self.connected.load(Ordering::Acquire) {
            return Err(ShmError::Disconnected);
        }

        if data.is_empty() {
            return Err(ShmError::InvalidState);
        }

        self.ring.write(priority, data)?;
        Ok(())
    }

    fn read(&self) -> ShmResult<(Priority, Vec<u8>)> {
        if !self.connected.load(Ordering::Acquire) {
            return Err(ShmError::Disconnected);
        }

        self.ring.read()
            .ok_or(ShmError::InvalidState)
    }

    fn signal(&self) {
        self.futex.wake_one();
    }

    fn wait(&self, timeout: Option<Duration>) -> ShmResult<()> {
        if !self.connected.load(Ordering::Acquire) {
            return Err(ShmError::Disconnected);
        }

        self.futex.wait(0, timeout)?;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire) && self.is_daemon_alive()
    }

    fn protocol_version(&self) -> u16 {
        self.expected_version
    }
}

impl Drop for NrelayShmTransport {
    fn drop(&mut self) {
        if !self.is_daemon {
            unsafe {
                (*self.control).data.client_count.fetch_sub(1, Ordering::AcqRel);
            }
        }
        self.connected.store(false, Ordering::Release);
    }
}
