//! Error types for protocol operations.
//!
//! Defines ProtocolError enum with variants for invalid magic, unsupported
//! versions, payload errors, serialization failures, and buffer overflows.

use alloc::string::String;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("invalid magic number: expected 0x4D4C4E4B, got 0x{0:08X}")]
    InvalidMagic(u32),

    #[error("unsupported protocol version: {0}")]
    UnsupportedVersion(u8),

    #[error("invalid message type: {0}")]
    InvalidMessageType(u8),

    #[error("payload too large: {0} bytes (max: {1} bytes)")]
    PayloadTooLarge(usize, usize),

    #[error("checksum mismatch: expected 0x{expected:08X}, got 0x{actual:08X}")]
    ChecksumMismatch {
        expected: u32,
        actual: u32,
    },

    #[error("serialization failed: {0}")]
    SerializationFailed(String),

    #[error("buffer overflow: required {required} bytes, available {available} bytes")]
    BufferOverflow {
        required: usize,
        available: usize,
    },

    #[error("invalid header: {0}")]
    InvalidHeader(String),

    #[error("invalid request ID: {0}")]
    InvalidRequestId(u64),

    #[error("unknown module ID: {0}")]
    UnknownModule(u64),

    #[error("unknown method hash: 0x{0:08X}")]
    UnknownMethod(u32),

    #[error("operation timed out after {0} ms")]
    Timeout(u64),

    #[error("connection closed: {0}")]
    ConnectionClosed(String),

    #[error("resource quota exceeded: {0}")]
    QuotaExceeded(String),
}

pub type Result<T, E = ProtocolError> = core::result::Result<T, E>;

impl ProtocolError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            ProtocolError::Timeout(_)
                | ProtocolError::BufferOverflow { .. }
                | ProtocolError::QuotaExceeded(_)
        )
    }

    pub fn is_corruption(&self) -> bool {
        matches!(
            self,
            ProtocolError::InvalidMagic(_)
                | ProtocolError::ChecksumMismatch { .. }
                | ProtocolError::InvalidHeader(_)
        )
    }

    pub fn is_protocol_mismatch(&self) -> bool {
        matches!(
            self,
            ProtocolError::InvalidMagic(_) | ProtocolError::UnsupportedVersion(_)
        )
    }
}
