//! Shared memory view for zero-copy access.
//!
//! Defines ShmView struct for safe, bounds-checked access to shared
//! memory regions with methods for reading headers and payloads.

use alloc::string::ToString;
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::error::{ProtocolError, Result};
use crate::header::MessageHeader;
use crate::magic::HEADER_SIZE;

#[cfg(feature = "shm")]
pub use memlink_shm::{
    RingBuffer, Priority as ShmPriority, Platform, MmapSegment, ControlRegion,
    Futex, FutexError, FutexResult, PriorityRingBuffer,
    ShmTransport, ShmError, ShmResult,
    RecoveryManager, Heartbeat, SlotMetadata, SlotState,
    BoundsChecker, PoisonGuard, BoundsError,
};

pub const SHM_ALIGNMENT: usize = 64;

#[derive(Debug)]
pub struct ShmView<'a> {
    ptr: NonNull<u8>,
    len: usize,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> ShmView<'a> {
    pub unsafe fn new(ptr: *const u8, len: usize) -> Self {
        let non_null_ptr = NonNull::new(ptr as *mut u8).unwrap_or_else(|| {
            panic!("ShmView::new called with null pointer");
        });

        Self {
            ptr: non_null_ptr,
            len,
            _phantom: PhantomData,
        }
    }

    pub fn from_slice(slice: &'a [u8]) -> Self {
        Self {
            ptr: NonNull::from(slice).cast(),
            len: slice.len(),
            _phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &'a [u8] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn read_header(&self) -> Result<MessageHeader> {
        if self.len < HEADER_SIZE {
            return Err(ProtocolError::BufferOverflow {
                required: HEADER_SIZE,
                available: self.len,
            });
        }

        let header_bytes: &[u8; HEADER_SIZE] = self.as_slice()[..HEADER_SIZE]
            .try_into()
            .map_err(|_| ProtocolError::InvalidHeader("failed to convert to header array".to_string()))?;

        MessageHeader::from_bytes(header_bytes)
    }

    pub fn read_payload_at(&self, offset: usize, payload_len: usize) -> Result<&'a [u8]> {
        let end = offset
            .checked_add(payload_len)
            .ok_or(ProtocolError::BufferOverflow {
                required: offset + payload_len,
                available: self.len,
            })?;

        if end > self.len {
            return Err(ProtocolError::BufferOverflow {
                required: end,
                available: self.len,
            });
        }

        Ok(&self.as_slice()[offset..end])
    }

    pub fn read_payload(&self, payload_len: usize) -> Result<&'a [u8]> {
        self.read_payload_at(HEADER_SIZE, payload_len)
    }

    pub fn sub_view(&self, offset: usize, len: usize) -> Result<ShmView<'a>> {
        let end = offset
            .checked_add(len)
            .ok_or(ProtocolError::BufferOverflow {
                required: offset + len,
                available: self.len,
            })?;

        if end > self.len {
            return Err(ProtocolError::BufferOverflow {
                required: end,
                available: self.len,
            });
        }

        unsafe {
            Ok(ShmView::new(
                self.ptr.as_ptr().add(offset),
                len,
            ))
        }
    }

    pub fn has_minimum(&self, min_bytes: usize) -> bool {
        self.len >= min_bytes
    }

    pub unsafe fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }
}

pub fn is_aligned(ptr: *const u8) -> bool {
    (ptr as usize) % SHM_ALIGNMENT == 0
}
