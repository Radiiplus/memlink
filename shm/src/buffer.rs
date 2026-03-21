//! Lock-free single-producer single-consumer (SPSC) ring buffer.
//! Provides atomic slot state management with cache-line aligned slots.

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::{fmt, ptr};

pub const MAX_SLOT_SIZE: usize = 4096;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    Empty = 0,
    Ready = 1,
    Reading = 2,
}

#[repr(C, align(64))]
pub struct RingHeader {
    pub head: AtomicU64,
    pub tail: AtomicU64,
    pub capacity: u64,
    pub write_seq: AtomicU64,
    pub read_seq: AtomicU64,
    _padding: [u8; 64 - 8 * 6],
}

#[repr(C, align(64))]
pub struct Slot {
    pub state: AtomicU8,
    pub priority: u8,
    pub len: AtomicU32,
    pub data: [u8; MAX_SLOT_SIZE],
    _padding: [u8; 64 - ((1 + 1 + 4 + MAX_SLOT_SIZE) % 64)],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingBufferError {
    Full,
    Empty,
    InvalidState,
    DataTooLarge,
}

impl fmt::Display for RingBufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RingBufferError::Full => write!(f, "Ring buffer is full"),
            RingBufferError::Empty => write!(f, "Ring buffer is empty"),
            RingBufferError::InvalidState => write!(f, "Invalid slot state"),
            RingBufferError::DataTooLarge => write!(f, "Data exceeds maximum slot size"),
        }
    }
}

impl std::error::Error for RingBufferError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotId {
    pub index: u64,
    pub seq: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Priority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
}

pub struct RingBuffer {
    header: *mut RingHeader,
    slots: *mut Slot,
    capacity: usize,
}

unsafe impl Send for RingBuffer {}
unsafe impl Sync for RingBuffer {}

impl RingBuffer {
    unsafe fn with_capacity(capacity: usize) -> Result<Self, RingBufferError> {
        if capacity == 0 || !capacity.is_power_of_two() {
            return Err(RingBufferError::Full);
        }

        let header = Box::into_raw(Box::new(RingHeader {
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
            capacity: capacity as u64,
            write_seq: AtomicU64::new(0),
            read_seq: AtomicU64::new(0),
            _padding: [0u8; 64 - 48],
        }));

        let mut slots = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            slots.push(Slot {
                state: AtomicU8::new(SlotState::Empty as u8),
                priority: 0,
                len: AtomicU32::new(0),
                data: [0u8; MAX_SLOT_SIZE],
                _padding: [0u8; 64 - ((1 + 1 + 4 + MAX_SLOT_SIZE) % 64)],
            });
        }
        let slots = Box::into_raw(slots.into_boxed_slice()) as *mut Slot;

        Ok(Self {
            header,
            slots,
            capacity,
        })
    }

    pub fn new(capacity: usize) -> Result<Self, RingBufferError> {
        unsafe { Self::with_capacity(capacity) }
    }

    /// Create a ring buffer from raw pointers (for shared memory)
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `header` points to a valid RingHeader
    /// - `slots` points to an array of `capacity` Slot structures
    /// - Memory is properly aligned
    pub unsafe fn from_ptr(header: *mut RingHeader, slots: *mut Slot, capacity: usize) -> Self {
        Self {
            header,
            slots,
            capacity,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        let head = unsafe { (*self.header).head.load(Ordering::Acquire) };
        let tail = unsafe { (*self.header).tail.load(Ordering::Acquire) };
        head == tail
    }

    pub fn is_full(&self) -> bool {
        let head = unsafe { (*self.header).head.load(Ordering::Acquire) };
        let tail = unsafe { (*self.header).tail.load(Ordering::Acquire) };
        let capacity = self.capacity as u64;
        head.wrapping_sub(tail) >= capacity
    }

    pub fn len(&self) -> u64 {
        let head = unsafe { (*self.header).head.load(Ordering::Acquire) };
        let tail = unsafe { (*self.header).tail.load(Ordering::Acquire) };
        head.wrapping_sub(tail)
    }

    #[inline]
    fn next_slot_index(index: u64, capacity: u64) -> u64 {
        index & (capacity - 1)
    }

    #[inline]
    #[cfg(test)]
    #[allow(dead_code)]
    unsafe fn get_slot(&self, index: u64) -> &Slot {
        let slot_idx = Self::next_slot_index(index, self.capacity as u64);
        &*self.slots.add(slot_idx as usize)
    }

    pub fn write_slot(&self, priority: Priority, data: &[u8]) -> Result<SlotId, RingBufferError> {
        if data.len() > MAX_SLOT_SIZE {
            return Err(RingBufferError::DataTooLarge);
        }

        let head = unsafe { (*self.header).head.load(Ordering::Acquire) };
        let tail = unsafe { (*self.header).tail.load(Ordering::Acquire) };

        if head.wrapping_sub(tail) >= self.capacity as u64 {
            return Err(RingBufferError::Full);
        }

        let slot_idx = Self::next_slot_index(head, self.capacity as u64);

        unsafe {
            let slot = &mut *self.slots.add(slot_idx as usize);

            let current_state = slot.state.load(Ordering::Acquire);
            if current_state != SlotState::Empty as u8 {
                return Err(RingBufferError::InvalidState);
            }

            ptr::copy_nonoverlapping(data.as_ptr(), slot.data.as_mut_ptr(), data.len());
            slot.len.store(data.len() as u32, Ordering::Relaxed);
            slot.priority = priority as u8;

            let write_seq = (*self.header).write_seq.fetch_add(1, Ordering::Relaxed);

            std::sync::atomic::fence(Ordering::Release);

            slot.state.store(SlotState::Ready as u8, Ordering::Release);

            (*self.header).head.store(head.wrapping_add(1), Ordering::Release);

            Ok(SlotId {
                index: head,
                seq: write_seq,
            })
        }
    }

    pub fn read_slot(&self) -> Option<(Priority, Vec<u8>)> {
        let tail = unsafe { (*self.header).tail.load(Ordering::Acquire) };
        let head = unsafe { (*self.header).head.load(Ordering::Acquire) };

        if tail >= head {
            return None;
        }

        let slot_idx = Self::next_slot_index(tail, self.capacity as u64);

        unsafe {
            let slot = &*self.slots.add(slot_idx as usize);

            let current_state = slot.state.load(Ordering::Acquire);
            if current_state != SlotState::Ready as u8 {
                return None;
            }

            let len = slot.len.load(Ordering::Acquire) as usize;

            let mut data = Vec::with_capacity(len);
            ptr::copy_nonoverlapping(slot.data.as_ptr(), data.as_mut_ptr(), len);
            data.set_len(len);

            let priority = Priority::from_u8(slot.priority).unwrap_or(Priority::Normal);

            slot.state.store(SlotState::Reading as u8, Ordering::Relaxed);

            std::sync::atomic::fence(Ordering::Acquire);

            (*self.header).tail.store(tail.wrapping_add(1), Ordering::Release);

            slot.state.store(SlotState::Empty as u8, Ordering::Release);

            (*self.header).read_seq.fetch_add(1, Ordering::Relaxed);

            Some((priority, data))
        }
    }

    pub fn peek_slot(&self) -> Option<(Priority, Vec<u8>)> {
        let tail = unsafe { (*self.header).tail.load(Ordering::Acquire) };
        let head = unsafe { (*self.header).head.load(Ordering::Acquire) };

        if tail >= head {
            return None;
        }

        let slot_idx = Self::next_slot_index(tail, self.capacity as u64);

        unsafe {
            let slot = &*self.slots.add(slot_idx as usize);

            let current_state = slot.state.load(Ordering::Acquire);
            if current_state != SlotState::Ready as u8 {
                return None;
            }

            let len = slot.len.load(Ordering::Acquire) as usize;
            let mut data = Vec::with_capacity(len);
            ptr::copy_nonoverlapping(slot.data.as_ptr(), data.as_mut_ptr(), len);
            data.set_len(len);

            let priority = Priority::from_u8(slot.priority).unwrap_or(Priority::Normal);

            Some((priority, data))
        }
    }
}

impl Priority {
    fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Priority::Low),
            1 => Some(Priority::Normal),
            2 => Some(Priority::High),
            _ => None,
        }
    }
}

impl Drop for RingBuffer {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.header);
            let _ = Vec::from_raw_parts(self.slots, self.capacity, self.capacity);
        }
    }
}
