//! Request/Response types for memlink module calls.
//!
//! Re-exports types from memlink-protocol for module method
//! invocations with support for tracing, deadlines, and serialization.

pub use memlink_protocol::{Request, Response};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CallArgs {
    pub bytes: Vec<u8>,
}

impl CallArgs {
    pub fn new(bytes: Vec<u8>) -> Self {
        CallArgs { bytes }
    }
}
