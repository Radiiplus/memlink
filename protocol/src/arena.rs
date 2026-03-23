//! Arena-backed slice for zero-copy memory access.
//!
//! Defines ArenaSlice for offset-based references to arena memory,
//! ArenaConfig for configuration, and ArenaRef for arena management.

use alloc::string::ToString;

use core::ptr::NonNull;

use crate::error::{ProtocolError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaSlice {
    offset: usize,
    len: usize,
}

impl ArenaSlice {
    pub const fn new(offset: usize, len: usize) -> Self {
        Self { offset, len }
    }

    pub const fn empty() -> Self {
        Self { offset: 0, len: 0 }
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub unsafe fn resolve<'a>(&self, arena_base: *const u8) -> &'a [u8] {
        if self.len == 0 {
            return &[];
        }

        let ptr = arena_base.add(self.offset);
        core::slice::from_raw_parts(ptr, self.len)
    }

    pub unsafe fn resolve_mut<'a>(&self, arena_base: *mut u8) -> &'a mut [u8] {
        if self.len == 0 {
            return &mut [];
        }

        let ptr = arena_base.add(self.offset);
        core::slice::from_raw_parts_mut(ptr, self.len)
    }

    pub fn validate(&self, arena_size: usize) -> Result<()> {
        let end = self
            .offset
            .checked_add(self.len)
            .ok_or(ProtocolError::BufferOverflow {
                required: usize::MAX,
                available: arena_size,
            })?;

        if end > arena_size {
            return Err(ProtocolError::BufferOverflow {
                required: end,
                available: arena_size,
            });
        }

        Ok(())
    }

    pub unsafe fn from_ptr(arena_base: *const u8, ptr: *const u8, len: usize) -> Result<Self> {
        if ptr < arena_base {
            return Err(ProtocolError::InvalidHeader(
                "pointer is before arena base".to_string(),
            ));
        }

        let offset = (ptr as usize) - (arena_base as usize);
        Ok(Self::new(offset, len))
    }

    pub const fn end(&self) -> usize {
        self.offset + self.len
    }

    pub fn sub_slice(&self, start: usize, len: usize) -> Result<Self> {
        if start > self.len {
            return Err(ProtocolError::BufferOverflow {
                required: self.offset + start,
                available: self.offset + self.len,
            });
        }

        let end = start
            .checked_add(len)
            .ok_or(ProtocolError::BufferOverflow {
                required: usize::MAX,
                available: self.len,
            })?;

        if end > self.len {
            return Err(ProtocolError::BufferOverflow {
                required: self.offset + end,
                available: self.offset + self.len,
            });
        }

        Ok(Self::new(self.offset + start, len))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ArenaConfig {
    pub size: usize,
    pub alignment: usize,
    pub max_allocation: usize,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            size: 64 * 1024 * 1024,
            alignment: 8,
            max_allocation: 16 * 1024 * 1024,
        }
    }
}

impl ArenaConfig {
    pub const fn new(size: usize, alignment: usize, max_allocation: usize) -> Self {
        Self {
            size,
            alignment,
            max_allocation,
        }
    }

    pub fn validate_allocation(&self, size: usize) -> Result<()> {
        if size > self.max_allocation {
            return Err(ProtocolError::PayloadTooLarge(size, self.max_allocation));
        }

        if size > self.size {
            return Err(ProtocolError::PayloadTooLarge(size, self.size));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ArenaRef<'a> {
    base: NonNull<u8>,
    size: usize,
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a> ArenaRef<'a> {
    pub unsafe fn new(base: *mut u8, size: usize) -> Self {
        Self {
            base: NonNull::new(base).expect("ArenaRef base pointer is null"),
            size,
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn base(&self) -> *const u8 {
        self.base.as_ptr()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub unsafe fn resolve_slice(&self, slice: &ArenaSlice) -> Result<&'a [u8]> {
        slice.validate(self.size)?;
        Ok(slice.resolve(self.base.as_ptr()))
    }

    pub fn remaining(&self, offset: usize) -> usize {
        self.size.saturating_sub(offset)
    }
}
