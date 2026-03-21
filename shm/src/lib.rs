//! Shared memory IPC system with multi-priority message queues, futex signaling,
//! and cross-platform support (Linux, macOS, Windows). Provides daemon-client
//! architecture with crash recovery, bounds checking, and backpressure control.

pub mod platform;
pub mod mmap;
pub mod control;
pub mod layout;
pub mod buffer;
pub mod futex;
pub mod priority;
pub mod pring;
pub mod transport;
pub mod recovery;
pub mod safety;

pub use platform::Platform;
pub use mmap::MmapSegment;
pub use control::ControlRegion;
pub use layout::{
    CONTROL_REGION_SIZE, RING_BUFFER_OFFSET, MIN_SEGMENT_SIZE,
    DEFAULT_SEGMENT_SIZE, MAX_SEGMENT_SIZE, PAGE_SIZE,
};
pub use buffer::{RingBuffer, RingBufferError, SlotId, Priority as BufferPriority, MAX_SLOT_SIZE};
pub use futex::{Futex, FutexError, FutexResult};
pub use priority::{Priority, calculate_slot_distribution};
pub use pring::PriorityRingBuffer;
pub use transport::{ShmTransport, NrelayShmTransport, ShmError, ShmResult};
pub use recovery::{RecoveryManager, Heartbeat, SlotMetadata, SlotState};
pub use safety::{BoundsChecker, PoisonGuard, PoisonState, SafeShmAccess, BoundsError, SafeAccessError};
