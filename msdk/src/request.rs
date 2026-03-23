//! Request/Response types for memlink module calls.
//!
//! Defines Request, Response, and CallArgs structures for module method
//! invocations with support for tracing, deadlines, and serialization.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::serialize::{default_serializer, Serializer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub method_hash: u32,
    pub args: Vec<u8>,
    pub trace_id: u128,
    pub deadline_ns: Option<u64>,
}

impl Request {
    pub fn new(method_hash: u32, args: Vec<u8>) -> Self {
        Request {
            method_hash,
            args,
            trace_id: 0,
            deadline_ns: None,
        }
    }

    pub fn with_trace_id(mut self, trace_id: u128) -> Self {
        self.trace_id = trace_id;
        self
    }

    pub fn with_deadline(mut self, deadline_ns: u64) -> Self {
        self.deadline_ns = Some(deadline_ns);
        self
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        default_serializer().serialize(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        default_serializer().deserialize(bytes)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub data: Vec<u8>,
    pub error_code: Option<i32>,
}

impl Response {
    pub fn success(data: Vec<u8>) -> Self {
        Response {
            data,
            error_code: None,
        }
    }

    pub fn error(error_code: i32) -> Self {
        Response {
            data: vec![],
            error_code: Some(error_code),
        }
    }

    pub fn is_success(&self) -> bool {
        self.error_code.is_none()
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        default_serializer().serialize(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        default_serializer().deserialize(bytes)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallArgs {
    pub bytes: Vec<u8>,
}

impl CallArgs {
    pub fn new(bytes: Vec<u8>) -> Self {
        CallArgs { bytes }
    }
}
