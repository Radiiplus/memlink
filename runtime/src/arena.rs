//! Arena allocator for module memory management.

use std::sync::atomic::{AtomicUsize, Ordering};

pub const DEFAULT_ARENA_CAPACITY: usize = 64 * 1024 * 1024;

pub struct Arena {
    base: Vec<u8>,
    capacity: usize,
    used: AtomicUsize,
}

unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

impl Arena {
    pub fn new(capacity: usize) -> Self {
        Arena {
            base: vec![0u8; capacity],
            capacity,
            used: AtomicUsize::new(0),
        }
    }

    pub fn with_default_capacity() -> Self {
        Self::new(DEFAULT_ARENA_CAPACITY)
    }

    pub fn alloc(&self, size: usize) -> Option<*mut u8> {
        let mut current = self.used.load(Ordering::Relaxed);

        loop {
            let new_used = current + size;

            if new_used > self.capacity {
                return None;
            }

            match self.used.compare_exchange(
                current,
                new_used,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    let ptr = self.base.as_ptr() as usize + current;
                    return Some(ptr as *mut u8);
                }
                Err(actual) => {
                    current = actual;
                }
            }
        }
    }

    pub fn alloc_aligned(&self, size: usize, align: usize) -> Option<*mut u8> {
        debug_assert!(align.is_power_of_two(), "Alignment must be a power of 2");

        let mut current = self.used.load(Ordering::Relaxed);

        loop {
            let aligned = (current + align - 1) & !(align - 1);
            let new_used = aligned + size;

            if new_used > self.capacity {
                return None;
            }

            match self.used.compare_exchange(
                current,
                new_used,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    let ptr = self.base.as_ptr() as usize + aligned;
                    return Some(ptr as *mut u8);
                }
                Err(actual) => {
                    current = actual;
                }
            }
        }
    }

    pub fn reset(&self) {
        self.used.store(0, Ordering::Release);
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn used(&self) -> usize {
        self.used.load(Ordering::Acquire)
    }

    pub fn remaining(&self) -> usize {
        self.capacity - self.used.load(Ordering::Relaxed)
    }

    pub fn usage(&self) -> f32 {
        self.used() as f32 / self.capacity as f32
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.base
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.base
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

impl Clone for Arena {
    fn clone(&self) -> Self {
        Arena {
            base: self.base.clone(),
            capacity: self.capacity,
            used: AtomicUsize::new(self.used.load(Ordering::Relaxed)),
        }
    }
}

impl std::fmt::Debug for Arena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arena")
            .field("capacity", &self.capacity)
            .field("used", &self.used())
            .field("remaining", &self.remaining())
            .field("usage", &self.usage())
            .finish()
    }
}
