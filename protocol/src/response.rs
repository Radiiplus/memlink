//! Response message structure.
//!
//! Defines Response struct with header, data payload, and StatusCode.
//! Includes serialization and deserialization methods.

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::error::{ProtocolError, Result};
use crate::header::MessageHeader;
use crate::magic::MAX_PAYLOAD_SIZE;
use crate::types::{MethodHash, ModuleId, RequestId, StatusCode};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Response {
    pub header: MessageHeader,
    pub data: Vec<u8>,
    pub status: StatusCode,
}

impl Response {
    pub fn success(request_id: RequestId, data: Vec<u8>) -> Self {
        let header = MessageHeader::new(
            crate::types::MessageType::Response,
            request_id,
            0,
            0,
            data.len() as u32,
        );

        Self {
            header,
            data,
            status: StatusCode::Success,
        }
    }

    pub fn error(request_id: RequestId, status: StatusCode, data: Vec<u8>) -> Self {
        let header = MessageHeader::new(
            crate::types::MessageType::Response,
            request_id,
            0,
            0,
            data.len() as u32,
        );

        Self {
            header,
            data,
            status,
        }
    }

    pub fn with_routing(
        request_id: RequestId,
        status: StatusCode,
        data: Vec<u8>,
        module_id: ModuleId,
        method_hash: MethodHash,
    ) -> Self {
        let header = MessageHeader::new(
            crate::types::MessageType::Response,
            request_id,
            module_id,
            method_hash,
            data.len() as u32,
        );

        Self {
            header,
            data,
            status,
        }
    }

    pub fn request_id(&self) -> RequestId {
        self.header.request_id()
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    pub fn is_error(&self) -> bool {
        self.status.is_error()
    }

    pub fn into_bytes(self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        bytes.push(self.status.as_u8());

        bytes.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.data);

        if bytes.len() > MAX_PAYLOAD_SIZE {
            return Err(ProtocolError::PayloadTooLarge(bytes.len(), MAX_PAYLOAD_SIZE));
        }

        Ok(bytes)
    }

    pub fn from_bytes(payload: &[u8], header: MessageHeader) -> Result<Self> {
        if payload.is_empty() {
            return Err(ProtocolError::InvalidHeader(
                "empty payload".to_string(),
            ));
        }

        let status_code = payload[0];
        let status = StatusCode::from_u8(status_code).ok_or_else(|| {
            ProtocolError::InvalidHeader(format!("unknown status code: {}", status_code))
        })?;

        if payload.len() < 5 {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for data length".to_string(),
            ));
        }

        let data_len =
            u32::from_le_bytes([payload[1], payload[2], payload[3], payload[4]]) as usize;

        if payload.len() < 5 + data_len {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for data".to_string(),
            ));
        }

        let data = payload[5..5 + data_len].to_vec();

        Ok(Self {
            header,
            data,
            status,
        })
    }
}
