//! Error types for memlink modules, including ModuleError enum and Result type alias.

use std::fmt;
use std::time::Duration;

pub type Result<T> = std::result::Result<T, ModuleError>;

#[derive(Debug, PartialEq)]
pub enum ModuleError {
    QuotaExceeded,
    CallFailed(String),
    Timeout(Duration),
    Panic(String),
    Serialize(String),
    ServiceUnavailable,
    InvalidMethod,
    ModuleNotFound(String),
    MaxCallDepthExceeded,
}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleError::QuotaExceeded => write!(f, "quota exceeded"),
            ModuleError::CallFailed(msg) => write!(f, "call failed: {}", msg),
            ModuleError::Timeout(duration) => write!(f, "operation timed out after {:?}", duration),
            ModuleError::Panic(payload) => write!(f, "panic: {}", payload),
            ModuleError::Serialize(msg) => write!(f, "serialization error: {}", msg),
            ModuleError::ServiceUnavailable => write!(f, "service unavailable"),
            ModuleError::InvalidMethod => write!(f, "invalid method"),
            ModuleError::ModuleNotFound(name) => write!(f, "module not found: {}", name),
            ModuleError::MaxCallDepthExceeded => write!(f, "maximum call depth exceeded"),
        }
    }
}

impl std::error::Error for ModuleError {}

impl From<rmp_serde::encode::Error> for ModuleError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        ModuleError::Serialize(err.to_string())
    }
}

impl From<rmp_serde::decode::Error> for ModuleError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        ModuleError::Serialize(err.to_string())
    }
}

unsafe impl Send for ModuleError {}
unsafe impl Sync for ModuleError {}
