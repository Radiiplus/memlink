//! Persistent state references for memlink modules, allowing data to persist across calls.
//!
//! Provides ArenaRef for storing offsets to values in an arena, enabling
//! data to survive arena resets across module invocations.

use std::marker::PhantomData;
use std::ptr;

use crate::arena::Arena;

pub struct ArenaRef<T> {
    offset: usize,
    _phantom: PhantomData<T>,
}

unsafe impl<T: Send> Send for ArenaRef<T> {}
unsafe impl<T: Sync> Sync for ArenaRef<T> {}

impl<T> ArenaRef<T> {
    pub fn new(arena: &Arena, value: T) -> Option<Self> {
        let slot = arena.alloc_with(value)?;
        let offset = unsafe {
            (slot as *const T as *const u8)
                .offset_from(arena.base_ptr() as *const u8) as usize
        };

        Some(ArenaRef {
            offset,
            _phantom: PhantomData,
        })
    }

    pub unsafe fn from_offset(offset: usize) -> Self {
        ArenaRef {
            offset,
            _phantom: PhantomData,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub unsafe fn get<'a>(&self, arena: &'a Arena) -> &'a T {
        &*(arena.base_ptr().add(self.offset) as *const T)
    }

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut<'a>(&self, arena: &'a Arena) -> &'a mut T {
        &mut *(arena.base_ptr().add(self.offset) as *mut T)
    }

    pub unsafe fn write(&self, arena: &Arena, value: T) {
        ptr::write(self.get_mut(arena), value);
    }

    pub unsafe fn read(&self, arena: &Arena) -> T
    where
        T: Copy,
    {
        ptr::read(self.get(arena))
    }
}

impl<T> Clone for ArenaRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ArenaRef<T> {}
