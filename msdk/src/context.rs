//! Call context for memlink module invocations, providing execution environment and utilities.

use std::time::{Duration, Instant};

use crate::arena::Arena;
use crate::caller::ModuleCaller;
use crate::error::{ModuleError, Result};

pub struct CallContext<'a> {
    arena: &'a Arena,
    backpressure: f32,
    trace_id: u128,
    span_id: u64,
    deadline: Option<Instant>,
    module_caller: Option<&'a ModuleCaller>,
}

impl<'a> CallContext<'a> {
    pub fn new(
        arena: &'a Arena,
        backpressure: f32,
        trace_id: u128,
        span_id: u64,
        deadline: Option<Instant>,
        module_caller: Option<&'a ModuleCaller>,
    ) -> Self {
        CallContext {
            arena,
            backpressure,
            trace_id,
            span_id,
            deadline,
            module_caller,
        }
    }

    pub fn backpressure(&self) -> f32 {
        self.backpressure
    }

    pub fn arena(&self) -> &'a Arena {
        self.arena
    }

    pub fn trace_id(&self) -> u128 {
        self.trace_id
    }

    pub fn span_id(&self) -> u64 {
        self.span_id
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub fn is_expired(&self) -> bool {
        self.deadline
            .map(|d| Instant::now() > d)
            .unwrap_or(false)
    }

    pub fn remaining_time(&self) -> Option<Duration> {
        self.deadline.map(|d| {
            d.checked_duration_since(Instant::now())
                .unwrap_or(Duration::ZERO)
        })
    }

    pub fn module_caller(&self) -> Option<&'a ModuleCaller> {
        self.module_caller
    }

    pub async fn call(&self, module: &str, method: &str, args: &[u8]) -> Result<Vec<u8>> {
        match &self.module_caller {
            Some(caller) => caller.call(module, method, args).await,
            None => Err(ModuleError::ServiceUnavailable),
        }
    }

    pub async fn call_with_timeout(
        &self,
        module: &str,
        method: &str,
        args: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>> {
        match &self.module_caller {
            Some(caller) => caller.call_with_timeout(module, method, args, timeout).await,
            None => Err(ModuleError::ServiceUnavailable),
        }
    }

    pub fn set_backpressure(&mut self, backpressure: f32) {
        self.backpressure = backpressure.clamp(0.0, 1.0);
    }
}
