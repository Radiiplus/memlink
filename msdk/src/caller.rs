//! Module caller for nested invocations via internal channel system.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use crate::error::{ModuleError, Result};

pub const MAX_CALL_DEPTH: u32 = 5;

#[derive(Debug, Clone)]
pub struct InternalRequest {
    pub target_module: String,
    pub target_method: String,
    pub args: Vec<u8>,
    pub caller_trace_id: u128,
    pub deadline: Option<std::time::Instant>,
    pub depth: u32,
}

#[derive(Debug, Clone)]
pub struct InternalResponse {
    pub data: Option<Vec<u8>>,
    pub error: Option<(i32, String)>,
}

impl InternalResponse {
    pub fn success(data: Vec<u8>) -> Self {
        InternalResponse {
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: i32, message: String) -> Self {
        InternalResponse {
            data: None,
            error: Some((code, message)),
        }
    }
}

pub struct ModuleCaller {
    tx: Arc<tokio::sync::mpsc::Sender<InternalRequest>>,
    depth: u32,
    trace_id: u128,
    span_id: u64,
}

impl Clone for ModuleCaller {
    fn clone(&self) -> Self {
        ModuleCaller {
            tx: Arc::clone(&self.tx),
            depth: self.depth,
            trace_id: self.trace_id,
            span_id: self.span_id,
        }
    }
}

impl ModuleCaller {
    pub fn new(
        tx: tokio::sync::mpsc::Sender<InternalRequest>,
        depth: u32,
        trace_id: u128,
        span_id: u64,
    ) -> Self {
        ModuleCaller {
            tx: Arc::new(tx),
            depth,
            trace_id,
            span_id,
        }
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub fn trace_id(&self) -> u128 {
        self.trace_id
    }

    pub fn span_id(&self) -> u64 {
        self.span_id
    }

    pub async fn call(&self, module: &str, method: &str, args: &[u8]) -> Result<Vec<u8>> {
        self.call_with_timeout(module, method, args, Duration::from_secs(30)).await
    }

    pub async fn call_with_timeout(
        &self,
        module: &str,
        method: &str,
        args: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>> {
        if self.depth >= MAX_CALL_DEPTH {
            return Err(ModuleError::MaxCallDepthExceeded);
        }

        let deadline = Some(std::time::Instant::now() + timeout);

        let request = InternalRequest {
            target_module: module.to_string(),
            target_method: method.to_string(),
            args: args.to_vec(),
            caller_trace_id: self.trace_id,
            deadline,
            depth: self.depth + 1,
        };

        let response = self.send_request(request, timeout).await?;

        match response {
            InternalResponse {
                data: Some(data),
                error: None,
            } => Ok(data),
            InternalResponse {
                data: _,
                error: Some((code, msg)),
            } => {
                if code == -404 {
                    Err(ModuleError::ModuleNotFound(msg))
                } else if code == -408 {
                    Err(ModuleError::Timeout(timeout))
                } else {
                    Err(ModuleError::CallFailed(msg))
                }
            }
            _ => Err(ModuleError::CallFailed("unexpected response".to_string())),
        }
    }

    async fn send_request(
        &self,
        request: InternalRequest,
        timeout: Duration,
    ) -> Result<InternalResponse> {
        let tx = self.tx.as_ref();

        let send_future = tx.send(request);

        match tokio::time::timeout(timeout, send_future).await {
            Ok(Ok(())) => Ok(InternalResponse::success(vec![])),
            Ok(Err(_)) => Err(ModuleError::ServiceUnavailable),
            Err(_) => Err(ModuleError::Timeout(timeout)),
        }
    }

    pub fn for_nested_call(&self, span_id: u64) -> ModuleCaller {
        ModuleCaller {
            tx: Arc::clone(&self.tx),
            depth: self.depth + 1,
            trace_id: self.trace_id,
            span_id,
        }
    }
}

pub fn call_future(
    caller: &ModuleCaller,
    module: String,
    method: String,
    args: Vec<u8>,
) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>> {
    Box::pin(async move { caller.call(&module, &method, &args).await })
}
