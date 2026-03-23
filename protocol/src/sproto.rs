//! Stream protocol constants.
//!
//! Defines constants for stream handles including sizes, offsets,
//! timeouts, and chunk size limits for streaming operations.

pub const STREAM_ID_SIZE: usize = 32;

pub const STREAM_CHECKSUM_SIZE: usize = 16;

pub const STREAM_TOTAL_SIZE_OFFSET: usize = 32;

pub const STREAM_CHECKSUM_OFFSET: usize = 40;

pub const STREAM_EXPIRES_OFFSET: usize = 56;

pub const STREAM_HANDLE_SIZE: usize = 80;

pub const STREAM_HEADER_SIZE: usize = STREAM_HANDLE_SIZE;

pub const DEFAULT_STREAM_TIMEOUT_NS: u64 = 30_000_000_000;

pub const MAX_STREAM_TIMEOUT_NS: u64 = 3_600_000_000_000;

pub const MIN_CHUNK_SIZE: usize = 4096;

pub const DEFAULT_CHUNK_SIZE: usize = 65536;

pub const MAX_CHUNK_SIZE: usize = 1048576;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    Initializing = 0,
    Ready = 1,
    Transferring = 2,
    Complete = 3,
    Error = 4,
}

impl StreamState {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(StreamState::Initializing),
            1 => Some(StreamState::Ready),
            2 => Some(StreamState::Transferring),
            3 => Some(StreamState::Complete),
            4 => Some(StreamState::Error),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamMode {
    Push = 0,
    Pull = 1,
    Bidirectional = 2,
}

impl StreamMode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(StreamMode::Push),
            1 => Some(StreamMode::Pull),
            2 => Some(StreamMode::Bidirectional),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}
