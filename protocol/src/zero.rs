//! Zero-copy message parsing.
//!
//! Defines ZeroCopyRequest and ZeroCopyResponse for parsing messages
//! directly from shared memory without heap allocations.

use alloc::format;
use alloc::string::ToString;

use crate::error::{ProtocolError, Result};
use crate::header::MessageHeader;
use crate::magic::HEADER_SIZE;
use crate::request::Request;
use crate::shm::ShmView;
use crate::types::{Priority, RequestId, TraceId};

#[derive(Debug, Clone)]
pub struct ZeroCopyRequest<'a> {
    pub header: MessageHeader,
    pub module_name: &'a str,
    pub method_name: &'a str,
    pub args: &'a [u8],
    pub trace_id: TraceId,
    pub deadline_ns: Option<u64>,
}

impl<'a> ZeroCopyRequest<'a> {
    pub fn parse(shm: &'a ShmView<'a>) -> Result<Self> {
        let header = shm.read_header()?;

        let mut offset = HEADER_SIZE;
        let data = shm.as_slice();

        if offset + 4 > data.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for module_name length".to_string(),
            ));
        }
        let module_len =
            u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
                as usize;
        offset += 4;

        if offset + module_len > data.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for module_name".to_string(),
            ));
        }
        let module_bytes = &data[offset..offset + module_len];
        let module_name = core::str::from_utf8(module_bytes).map_err(|_| {
            ProtocolError::InvalidHeader("module_name is not valid UTF-8".to_string())
        })?;
        offset += module_len;

        if offset + 4 > data.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for method_name length".to_string(),
            ));
        }
        let method_len =
            u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
                as usize;
        offset += 4;

        if offset + method_len > data.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for method_name".to_string(),
            ));
        }
        let method_bytes = &data[offset..offset + method_len];
        let method_name = core::str::from_utf8(method_bytes).map_err(|_| {
            ProtocolError::InvalidHeader("method_name is not valid UTF-8".to_string())
        })?;
        offset += method_len;

        if offset + 4 > data.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for args length".to_string(),
            ));
        }
        let args_len =
            u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
                as usize;
        offset += 4;

        if offset + args_len > data.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for args".to_string(),
            ));
        }
        let args = &data[offset..offset + args_len];
        offset += args_len;

        if offset + 16 > data.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for trace_id".to_string(),
            ));
        }
        let trace_id = u128::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
            data[offset + 8],
            data[offset + 9],
            data[offset + 10],
            data[offset + 11],
            data[offset + 12],
            data[offset + 13],
            data[offset + 14],
            data[offset + 15],
        ]);
        offset += 16;

        if offset + 8 > data.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for deadline_ns".to_string(),
            ));
        }
        let deadline = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
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

    pub fn to_owned(&self) -> Request {
        use crate::request::Request;

        let mut req = Request::new(
            self.header.request_id(),
            Priority::Normal,
            self.module_name,
            self.method_name,
            self.args.to_vec(),
        )
        .with_trace_id(self.trace_id);

        if let Some(deadline) = self.deadline_ns {
            req = req.with_deadline_ns(Some(deadline));
        }

        req
    }

    pub fn request_id(&self) -> RequestId {
        self.header.request_id()
    }

    pub fn priority(&self) -> Priority {
        Priority::Normal
    }

    pub fn serialized_size(&self) -> usize {
        HEADER_SIZE +
        4 + self.module_name.len() +
        4 + self.method_name.len() +
        4 + self.args.len() +
        16 +
        8
    }
}

#[derive(Debug, Clone)]
pub struct ZeroCopyResponse<'a> {
    pub header: MessageHeader,
    pub data: &'a [u8],
    pub status: crate::types::StatusCode,
}

impl<'a> ZeroCopyResponse<'a> {
    pub fn parse(shm: &'a ShmView<'a>) -> Result<Self> {
        let header = shm.read_header()?;

        let data = shm.as_slice();
        let mut offset = HEADER_SIZE;

        if offset >= data.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for status code".to_string(),
            ));
        }
        let status_code = data[offset];
        let status = crate::types::StatusCode::from_u8(status_code).ok_or_else(|| {
            ProtocolError::InvalidHeader(format!("unknown status code: {}", status_code))
        })?;
        offset += 1;

        if offset + 4 > data.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for data length".to_string(),
            ));
        }
        let data_len =
            u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
                as usize;
        offset += 4;

        if offset + data_len > data.len() {
            return Err(ProtocolError::InvalidHeader(
                "insufficient data for data".to_string(),
            ));
        }
        let data = &data[offset..offset + data_len];

        Ok(Self {
            header,
            data,
            status,
        })
    }

    pub fn to_owned(&self) -> crate::response::Response {
        use crate::response::Response;

        Response::with_routing(
            self.header.request_id(),
            self.status,
            self.data.to_vec(),
            self.header.module_id(),
            self.header.method_hash(),
        )
    }
}
