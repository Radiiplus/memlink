//! Memory safety utilities: bounds checking, panic guards, and safe access wrappers.

use std::cell::Cell;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct BoundsChecker {
    size: usize,
    base: usize,
}

impl BoundsChecker {
    pub fn new(base: *const u8, size: usize) -> Self {
        Self {
            size,
            base: base as usize,
        }
    }

    pub fn is_valid_offset(&self, offset: usize) -> bool {
        offset < self.size
    }

    pub fn is_valid_pointer(&self, ptr: *const u8) -> bool {
        let addr = ptr as usize;
        addr >= self.base && addr < self.base + self.size
    }

    pub fn validate_offset(&self, offset: usize) -> Result<usize, BoundsError> {
        if !self.is_valid_offset(offset) {
            return Err(BoundsError::OutOfBounds {
                offset,
                size: self.size,
            });
        }
        Ok(offset)
    }

    pub fn validate_range(
        &self,
        offset: usize,
        len: usize,
    ) -> Result<(), BoundsError> {
        if offset >= self.size {
            return Err(BoundsError::OutOfBounds {
                offset,
                size: self.size,
            });
        }

        let end = offset.checked_add(len).ok_or(BoundsError::Overflow)?;
        if end > self.size {
            return Err(BoundsError::OutOfBounds {
                offset: end,
                size: self.size,
            });
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundsError {
    OutOfBounds { offset: usize, size: usize },
    Overflow,
}

impl std::fmt::Display for BoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundsError::OutOfBounds { offset, size } => {
                write!(f, "Offset {} is out of bounds (size {})", offset, size)
            }
            BoundsError::Overflow => write!(f, "Arithmetic overflow in offset calculation"),
        }
    }
}

impl std::error::Error for BoundsError {}

pub struct PoisonGuard {
    poisoned: Arc<AtomicBool>,
    disarmed: Cell<bool>,
}

impl PoisonGuard {
    pub fn new(poisoned: Arc<AtomicBool>) -> Self {
        poisoned.store(true, Ordering::Release);
        Self {
            poisoned,
            disarmed: Cell::new(false),
        }
    }

    pub fn disarm(&self) {
        self.disarmed.set(true);
        self.poisoned.store(false, Ordering::Release);
    }

    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::Acquire)
    }
}

impl Drop for PoisonGuard {
    fn drop(&mut self) {
        if !self.disarmed.get() {
        }
    }
}

pub struct PoisonState {
    poisoned: Arc<AtomicBool>,
}

impl PoisonState {
    pub fn new() -> Self {
        Self {
            poisoned: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::Acquire)
    }

    pub fn set_poisoned(&self) {
        self.poisoned.store(true, Ordering::Release);
    }

    pub fn clear_poisoned(&self) {
        self.poisoned.store(false, Ordering::Release);
    }

    pub fn guard(&self) -> PoisonGuard {
        PoisonGuard::new(Arc::clone(&self.poisoned))
    }
}

impl Default for PoisonState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn with_panic_protection<T, F>(
    poison: &PoisonState,
    f: F,
) -> Result<T, Box<dyn std::any::Any + Send>>
where
    F: FnOnce() -> T,
{
    let guard = poison.guard();
    let result = panic::catch_unwind(AssertUnwindSafe(f));

    match result {
        Ok(val) => {
            guard.disarm();
            Ok(val)
        }
        Err(e) => {
            poison.set_poisoned();
            Err(e)
        }
    }
}

pub struct RobustFutex {
    value: AtomicUsize,
    poisoned: AtomicBool,
}

impl RobustFutex {
    pub fn new(initial: usize) -> Self {
        Self {
            value: AtomicUsize::new(initial),
            poisoned: AtomicBool::new(false),
        }
    }

    pub fn try_lock(&self) -> Result<bool, FutexError> {
        if self.poisoned.load(Ordering::Acquire) {
            return Err(FutexError::Poisoned);
        }

        let current = self.value.load(Ordering::Acquire);
        if current == 0 {
            match self.value.compare_exchange(
                0,
                1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    pub fn unlock(&self) {
        self.value.store(0, Ordering::Release);
    }

    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::Acquire)
    }

    pub fn mark_poisoned(&self) {
        self.poisoned.store(true, Ordering::Release);
    }

    pub fn clear_poisoned(&self) {
        self.poisoned.store(false, Ordering::Release);
        self.value.store(0, Ordering::Release);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FutexError {
    Poisoned,
    WouldBlock,
    Other,
}

impl std::fmt::Display for FutexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FutexError::Poisoned => write!(f, "Futex is poisoned (owner died)"),
            FutexError::WouldBlock => write!(f, "Operation would block"),
            FutexError::Other => write!(f, "Futex error"),
        }
    }
}

impl std::error::Error for FutexError {}

pub struct SafeShmAccess {
    bounds: BoundsChecker,
    poison: PoisonState,
}

impl SafeShmAccess {
    pub fn new(base: *const u8, size: usize) -> Self {
        Self {
            bounds: BoundsChecker::new(base, size),
            poison: PoisonState::new(),
        }
    }

    pub fn is_poisoned(&self) -> bool {
        self.poison.is_poisoned()
    }

    pub fn validate_offset(&self, offset: usize) -> Result<(), BoundsError> {
        self.bounds.validate_offset(offset).map(|_| ())
    }

    pub fn validate_range(&self, offset: usize, len: usize) -> Result<(), BoundsError> {
        self.bounds.validate_range(offset, len)
    }

    pub fn with_safe_access<T, F>(&self, offset: usize, len: usize, f: F) -> Result<T, SafeAccessError>
    where
        F: FnOnce() -> T,
    {
        self.bounds.validate_range(offset, len)?;

        if self.poison.is_poisoned() {
            return Err(SafeAccessError::Poisoned);
        }

        match with_panic_protection(&self.poison, f) {
            Ok(val) => Ok(val),
            Err(_) => Err(SafeAccessError::Panicked),
        }
    }

    pub fn bounds(&self) -> &BoundsChecker {
        &self.bounds
    }

    pub fn poison(&self) -> &PoisonState {
        &self.poison
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeAccessError {
    OutOfBounds,
    Poisoned,
    Panicked,
}

impl std::fmt::Display for SafeAccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafeAccessError::OutOfBounds => write!(f, "Access out of bounds"),
            SafeAccessError::Poisoned => write!(f, "Access is poisoned"),
            SafeAccessError::Panicked => write!(f, "Operation panicked"),
        }
    }
}

impl std::error::Error for SafeAccessError {}

impl From<BoundsError> for SafeAccessError {
    fn from(_: BoundsError) -> Self {
        SafeAccessError::OutOfBounds
    }
}
