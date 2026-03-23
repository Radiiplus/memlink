//! Error message structure.
//!
//! Defines ErrorMessage struct with error_code, error_message, and
//! optional retry_after_ms. Includes serialization methods.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::error::{ProtocolError, Result};
use crate::header::MessageHeader;
use crate::magic::MAX_PAYLOAD_SIZE;
use crate::types::{ModuleId, RequestId};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ErrorMessage {
    pub header: MessageHeader,
    pub error_code: u32,
    pub error_message: String,
    pub retry_after_ms: Option<u64>,
}

impl ErrorMessage {
    pub fn new(request_id: RequestId, error_code: u32, error_message: String) -> Self {
        let header = MessageHeader::new(
            crate::types::MessageType::Error,
            request_id,
            0,
            0,
            0,
        );

        Self {
            header,
            error_code,
            error_message,
            retry_after_ms: None,
        }
    }

    pub fn with_retry_after_ms(mut self, retry_after_ms: Option<u64>) -> Self {
        self.retry_after_ms = retry_after_ms;
        self
    }

    pub fn with_module_id(mut self, module_id: ModuleId) -> Self {
        self.header = MessageHeader::new(
            crate::types::MessageType::Error,
            self.header.request_id(),
            module_id,
            self.header.method_hash(),
            0,
        );
        self
    }

    pub fn request_id(&self) -> RequestId {
        self.header.request_id()
    }

    pub fn error_code(&self) -> u32 {
        self.error_code
    }

    pub fn error_message(&self) -> &str {
        &self.error_message
    }

    pub fn retry_after_ms(&self) -> Option<u64> {
        self.retry_after_ms
    }

    pub fn into_bytes(self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.error_code.to_le_bytes());

        let message_bytes = self.error_message.as_bytes();
        bytes.extend_from_slice(&(message_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(message_bytes);

        let retry = self.retry_after_ms.unwrap_or(0);
        bytes.extend_from_slice(&retry.to_le_bytes());

        if bytes.len() > MAX_PAYLOAD_SIZE {
            return Err(ProtocolError::PayloadTooLarge(bytes.len(), MAX_PAYLOAD_SIZE));
        }

        Ok(bytes)
    }

    pub fn from_bytes(payload: &[u8], header: MessageHeader) -> Result<Self> {
        let mut offset = 0;

        if offset + 4 > payload.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for error_code".to_string(),
            ));
        }
        let error_code =
            u32::from_le_bytes([payload[offset], payload[offset + 1], payload[offset + 2], payload[offset + 3]]);
        offset += 4;

        if offset + 4 > payload.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for error_message length".to_string(),
            ));
        }
        let message_len =
            u32::from_le_bytes([payload[offset], payload[offset + 1], payload[offset + 2], payload[offset + 3]])
                as usize;
        offset += 4;

        if offset + message_len > payload.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for error_message".to_string(),
            ));
        }
        let message_bytes = &payload[offset..offset + message_len];
        let error_message = String::from_utf8(message_bytes.to_vec()).map_err(|_| {
            ProtocolError::InvalidHeader("error_message is not valid UTF-8".to_string())
        })?;
        offset += message_len;

        if offset + 8 > payload.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for retry_after_ms".to_string(),
            ));
        }
        let retry = u64::from_le_bytes([
            payload[offset],
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
            payload[offset + 4],
            payload[offset + 5],
            payload[offset + 6],
            payload[offset + 7],
        ]);
        let retry_after_ms = if retry == 0 { None } else { Some(retry) };

        Ok(Self {
            header,
            error_code,
            error_message,
            retry_after_ms,
        })
    }
}
