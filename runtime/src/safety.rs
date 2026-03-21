//! Safety hardening features for the runtime.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::error::{Error, Result};

pub const DEFAULT_CALL_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_MAX_STACK_DEPTH: usize = 100;
pub const DEFAULT_MEMORY_LIMIT: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct SafetyConfig {
    pub call_timeout: Duration,
    pub max_stack_depth: usize,
    pub memory_limit: usize,
    pub enforce_timeout: bool,
    pub check_stack_depth: bool,
    pub enforce_memory_limit: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        SafetyConfig {
            call_timeout: DEFAULT_CALL_TIMEOUT,
            max_stack_depth: DEFAULT_MAX_STACK_DEPTH,
            memory_limit: DEFAULT_MEMORY_LIMIT,
            enforce_timeout: true,
            check_stack_depth: false,
            enforce_memory_limit: true,
        }
    }
}

impl SafetyConfig {
    pub fn with_call_timeout(mut self, timeout: Duration) -> Self {
        self.call_timeout = timeout;
        self
    }

    pub fn with_max_stack_depth(mut self, depth: usize) -> Self {
        self.max_stack_depth = depth;
        self
    }

    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = limit;
        self
    }

    pub fn with_timeout_enforcement(mut self, enabled: bool) -> Self {
        self.enforce_timeout = enabled;
        self
    }

    pub fn with_stack_depth_check(mut self, enabled: bool) -> Self {
        self.check_stack_depth = enabled;
        self
    }

    pub fn with_memory_limit_enforcement(mut self, enabled: bool) -> Self {
        self.enforce_memory_limit = enabled;
        self
    }
}

#[derive(Debug, Default)]
pub struct StackDepth {
    depth: AtomicUsize,
}

impl StackDepth {
    pub fn new() -> Self {
        StackDepth {
            depth: AtomicUsize::new(0),
        }
    }

    pub fn enter(&self, max_depth: usize) -> Result<()> {
        let new_depth = self.depth.fetch_add(1, Ordering::Relaxed) + 1;
        if new_depth > max_depth {
            self.depth.fetch_sub(1, Ordering::Relaxed);
            Err(Error::ModulePanicked(format!(
                "Stack depth {} exceeds maximum {}",
                new_depth, max_depth
            )))
        } else {
            Ok(())
        }
    }

    pub fn exit(&self) {
        self.depth.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn current(&self) -> usize {
        self.depth.load(Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.depth.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct MemoryTracker {
    used: AtomicUsize,
    limit: usize,
}

impl MemoryTracker {
    pub fn new(limit: usize) -> Self {
        MemoryTracker {
            used: AtomicUsize::new(0),
            limit,
        }
    }

    pub fn allocate(&self, size: usize) -> Result<()> {
        let mut current = self.used.load(Ordering::Relaxed);

        loop {
            let new_used = current + size;
            if new_used > self.limit {
                return Err(Error::InvalidModuleFormat(format!(
                    "Memory allocation of {} bytes would exceed limit of {} bytes",
                    size, self.limit
                )));
            }

            match self.used.compare_exchange(
                current,
                new_used,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),
                Err(x) => current = x,
            }
        }
    }

    pub fn free(&self, size: usize) {
        self.used.fetch_sub(size, Ordering::Relaxed);
    }

    pub fn used(&self) -> usize {
        self.used.load(Ordering::Relaxed)
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn remaining(&self) -> usize {
        self.limit - self.used.load(Ordering::Relaxed)
    }

    pub fn usage_ratio(&self) -> f32 {
        self.used.load(Ordering::Relaxed) as f32 / self.limit as f32
    }
}

pub fn with_timeout<F, R>(timeout: Duration, f: F) -> Result<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let timeout_flag = Arc::new(AtomicBool::new(false));
    let timeout_flag_clone = Arc::clone(&timeout_flag);

    let handle = thread::spawn(move || {
        let result = f();
        timeout_flag_clone.store(true, Ordering::Relaxed);
        result
    });

    match handle.join_timeout(timeout) {
        Ok(result) => Ok(result),
        Err(_) => Err(Error::ModuleCallFailed(-3)),
    }
}

trait JoinHandleTimeout<T> {
    fn join_timeout(self, timeout: Duration) -> thread::Result<T>;
}

impl<T> JoinHandleTimeout<T> for thread::JoinHandle<T> {
    fn join_timeout(self, timeout: Duration) -> thread::Result<T> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if self.is_finished() {
                return self.join();
            }
            thread::sleep(Duration::from_millis(1));
        }
        Err(Box::new("Timeout expired"))
    }
}

pub fn validate_call_safety(
    config: &SafetyConfig,
    stack_depth: &StackDepth,
    memory_tracker: Option<&MemoryTracker>,
) -> Result<()> {
    if config.check_stack_depth {
        stack_depth.enter(config.max_stack_depth)?;
    }

    if let Some(tracker) = memory_tracker {
        if config.enforce_memory_limit && tracker.used() > tracker.limit() {
            return Err(Error::InvalidModuleFormat(
                "Memory limit exceeded".to_string()
            ));
        }
    }

    Ok(())
}
