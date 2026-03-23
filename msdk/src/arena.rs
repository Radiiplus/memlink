//! Bump-pointer arena allocator for temporary module allocations.
//!
//! Provides fast allocation for temporary data that lives only during
//! module call execution. All memory is reclaimed at once via reset().

use std::marker::PhantomData;
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Arena {
    base: NonNull<u8>,
    capacity: usize,
    used: AtomicUsize,
    _marker: PhantomData<*mut u8>,
}

unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

impl Arena {
    pub unsafe fn new(base: *mut u8, capacity: usize) -> Self {
        debug_assert!(!base.is_null(), "Arena base pointer must not be null");
        debug_assert!(capacity > 0, "Arena capacity must be > 0");

        Arena {
            base: NonNull::new_unchecked(base),
            capacity,
            used: AtomicUsize::new(0),
            _marker: PhantomData,
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn alloc<T>(&self) -> Option<&mut T> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        let offset = self.reserve_with_alignment(size, align)?;

        Some(unsafe { &mut *(self.base.as_ptr().add(offset) as *mut T) })
    }

    #[allow(clippy::mut_from_ref)]
    pub fn alloc_with<T>(&self, value: T) -> Option<&mut T> {
        let slot = self.alloc::<T>()?;
        unsafe { ptr::write(slot, value) };
        Some(slot)
    }

    #[allow(clippy::mut_from_ref)]
    pub fn alloc_bytes(&self, len: usize) -> Option<&mut [u8]> {
        if len == 0 {
            return Some(&mut []);
        }

        let offset = self.reserve(len)?;

        Some(unsafe {
            std::slice::from_raw_parts_mut(self.base.as_ptr().add(offset), len)
        })
    }

    pub fn base_ptr(&self) -> *mut u8 {
        self.base.as_ptr()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn used(&self) -> usize {
        self.used.load(Ordering::Relaxed)
    }

    pub fn remaining(&self) -> usize {
        self.capacity.saturating_sub(self.used())
    }

    pub fn usage(&self) -> f32 {
        self.used() as f32 / self.capacity as f32
    }

    pub fn reset(&self) {
        self.used.store(0, Ordering::Relaxed);
    }

    fn reserve(&self, size: usize) -> Option<usize> {
        if size == 0 {
            return Some(self.used.load(Ordering::Relaxed));
        }

        loop {
            let current = self.used.load(Ordering::Relaxed);
            let new_used = current.checked_add(size)?;

            if new_used > self.capacity {
                return None;
            }

            match self.used.compare_exchange(
                current,
                new_used,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(current),
                Err(_) => continue,
            }
        }
    }

    fn reserve_with_alignment(&self, size: usize, align: usize) -> Option<usize> {
        if size == 0 {
            return Some(self.used.load(Ordering::Relaxed));
        }

        loop {
            let current = self.used.load(Ordering::Relaxed);

            let aligned = (current + align - 1) & !(align - 1);
            let new_used = aligned.checked_add(size)?;

            if new_used > self.capacity {
                return None;
            }

            match self.used.compare_exchange(
                current,
                new_used,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(aligned),
                Err(_) => continue,
            }
        }
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        self.reset();
    }
}
