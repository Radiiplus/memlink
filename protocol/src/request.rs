//! Request message structure.
//!
//! Defines Request struct with module/method names, args, trace_id,
//! and deadline. Includes serialization and deserialization methods.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::error::{ProtocolError, Result};
use crate::header::MessageHeader;
use crate::magic::MAX_PAYLOAD_SIZE;
use crate::types::{Priority, RequestId, TraceId};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Request {
    pub header: MessageHeader,
    pub module_name: String,
    pub method_name: String,
    pub args: Vec<u8>,
    pub trace_id: TraceId,
    pub deadline_ns: Option<u64>,
}

impl Request {
    pub fn new(
        request_id: RequestId,
        _priority: Priority,
        module: &str,
        method: &str,
        args: Vec<u8>,
    ) -> Self {
        let module_name = module.to_string();
        let method_name = method.to_string();

        let method_hash = compute_fnv1a_hash(method.as_bytes());
        let module_id = compute_fnv1a_hash(module.as_bytes()) as u64;

        let header = MessageHeader::new(
            crate::types::MessageType::Request,
            request_id,
            module_id,
            method_hash,
            args.len() as u32,
        );

        Self {
            header,
            module_name,
            method_name,
            args,
            trace_id: 0,
            deadline_ns: None,
        }
    }

    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self {
        self.trace_id = trace_id;
        self
    }

    pub fn with_deadline_ns(mut self, deadline_ns: Option<u64>) -> Self {
        self.deadline_ns = deadline_ns;
        self
    }

    pub fn request_id(&self) -> RequestId {
        self.header.request_id()
    }

    pub fn priority(&self) -> Priority {
        Priority::Normal
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    pub fn args(&self) -> &[u8] {
        &self.args
    }

    pub fn trace_id(&self) -> TraceId {
        self.trace_id
    }

    pub fn deadline_ns(&self) -> Option<u64> {
        self.deadline_ns
    }

    pub fn into_bytes(self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        let module_bytes = self.module_name.as_bytes();
        bytes.extend_from_slice(&(module_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(module_bytes);

        let method_bytes = self.method_name.as_bytes();
        bytes.extend_from_slice(&(method_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(method_bytes);

        bytes.extend_from_slice(&(self.args.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.args);

        bytes.extend_from_slice(&self.trace_id.to_le_bytes());

        let deadline = self.deadline_ns.unwrap_or(0);
        bytes.extend_from_slice(&deadline.to_le_bytes());

        if bytes.len() > MAX_PAYLOAD_SIZE {
            return Err(ProtocolError::PayloadTooLarge(bytes.len(), MAX_PAYLOAD_SIZE));
        }

        Ok(bytes)
    }

    pub fn from_bytes(payload: &[u8], header: MessageHeader) -> Result<Self> {
        let mut offset = 0;

        if offset + 4 > payload.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for module_name length".to_string(),
            ));
        }
        let module_len =
            u32::from_le_bytes([payload[offset], payload[offset + 1], payload[offset + 2], payload[offset + 3]])
                as usize;
        offset += 4;

        if offset + module_len > payload.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for module_name".to_string(),
            ));
        }
        let module_bytes = &payload[offset..offset + module_len];
        let module_name = String::from_utf8(module_bytes.to_vec()).map_err(|_| {
            ProtocolError::InvalidHeader("module_name is not valid UTF-8".to_string())
        })?;
        offset += module_len;

        if offset + 4 > payload.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for method_name length".to_string(),
            ));
        }
        let method_len =
            u32::from_le_bytes([payload[offset], payload[offset + 1], payload[offset + 2], payload[offset + 3]])
                as usize;
        offset += 4;

        if offset + method_len > payload.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for method_name".to_string(),
            ));
        }
        let method_bytes = &payload[offset..offset + method_len];
        let method_name = String::from_utf8(method_bytes.to_vec()).map_err(|_| {
            ProtocolError::InvalidHeader("method_name is not valid UTF-8".to_string())
        })?;
        offset += method_len;

        if offset + 4 > payload.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for args length".to_string(),
            ));
        }
        let args_len =
            u32::from_le_bytes([payload[offset], payload[offset + 1], payload[offset + 2], payload[offset + 3]])
                as usize;
        offset += 4;

        if offset + args_len > payload.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for args".to_string(),
            ));
        }
        let args = payload[offset..offset + args_len].to_vec();
        offset += args_len;

        if offset + 16 > payload.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for trace_id".to_string(),
            ));
        }
        let trace_id = u128::from_le_bytes([
            payload[offset],
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
            payload[offset + 4],
            payload[offset + 5],
            payload[offset + 6],
            payload[offset + 7],
            payload[offset + 8],
            payload[offset + 9],
            payload[offset + 10],
            payload[offset + 11],
            payload[offset + 12],
            payload[offset + 13],
            payload[offset + 14],
            payload[offset + 15],
        ]);
        offset += 16;

        if offset + 8 > payload.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for deadline_ns".to_string(),
            ));
        }
        let deadline = u64::from_le_bytes([
            payload[offset],
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
            payload[offset + 4],
            payload[offset + 5],
            payload[offset + 6],
            payload[offset + 7],
        ]);
        let deadline_ns = if deadline == 0 { None } else { Some(deadline) };

        Ok(Self {
            header,
            module_name,
            method_name,
            args,
            trace_id,
            deadline_ns,
        })
    }
}

fn compute_fnv1a_hash(data: &[u8]) -> u32 {
    const FNV_OFFSET_BASIS: u32 = 2166136261;
    const FNV_PRIME: u32 = 16777619;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in data {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
