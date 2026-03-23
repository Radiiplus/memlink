//! Type aliases and enumerations.
//!
//! Defines core types including RequestId, ModuleId, StatusCode, Priority,
//! and MessageType enums used throughout the protocol.

pub type RequestId = u64;

pub type ModuleId = u64;

pub type MethodHash = u32;

pub type TraceId = u128;

pub type SpanId = u64;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StatusCode {
    Success = 0,
    ModuleNotFound = 1,
    MethodNotFound = 2,
    ExecutionError = 3,
    Timeout = 4,
    QuotaExceeded = 5,
    BackpressureRejection = 6,
}

impl StatusCode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(StatusCode::Success),
            1 => Some(StatusCode::ModuleNotFound),
            2 => Some(StatusCode::MethodNotFound),
            3 => Some(StatusCode::ExecutionError),
            4 => Some(StatusCode::Timeout),
            5 => Some(StatusCode::QuotaExceeded),
            6 => Some(StatusCode::BackpressureRejection),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn is_success(self) -> bool {
        matches!(self, StatusCode::Success)
    }

    pub fn is_error(self) -> bool {
        !self.is_success()
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, serde::Serialize, serde::Deserialize)]
pub enum Priority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Priority {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Priority::Low),
            1 => Some(Priority::Normal),
            2 => Some(Priority::High),
            3 => Some(Priority::Critical),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MessageType {
    Request = 0,
    Response = 1,
    Error = 2,
    StreamHandle = 3,
    HealthCheck = 16,
    LoadModule = 32,
    UnloadModule = 33,
    Stats = 48,
    Event = 96,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(MessageType::Request),
            1 => Some(MessageType::Response),
            2 => Some(MessageType::Error),
            3 => Some(MessageType::StreamHandle),
            16 => Some(MessageType::HealthCheck),
            32 => Some(MessageType::LoadModule),
            33 => Some(MessageType::UnloadModule),
            48 => Some(MessageType::Stats),
            96 => Some(MessageType::Event),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn expects_response(self) -> bool {
        matches!(
            self,
            MessageType::Request
                | MessageType::HealthCheck
                | MessageType::LoadModule
                | MessageType::UnloadModule
                | MessageType::Stats
        )
    }
}
