//! MessagePack serializer implementation.
//!
//! Provides MessagePackSerializer struct implementing the Serializer trait
//! using rmp-serde for high-performance binary serialization.

use alloc::format;
use alloc::vec::Vec;

use crate::emsg::ErrorMessage;
use crate::error::{ProtocolError, Result};
use crate::request::Request;
use crate::response::Response;
use crate::serializer::Serializer;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MessagePackSerializer;

impl MessagePackSerializer {
    pub const fn new() -> Self {
        Self
    }
}

pub const MSGPACK: MessagePackSerializer = MessagePackSerializer;

impl Serializer for MessagePackSerializer {
    fn serialize<T: serde::Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        rmp_serde::to_vec(value).map_err(|e| {
            ProtocolError::SerializationFailed(format!("MessagePack serialization error: {}", e))
        })
    }

    fn deserialize<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        rmp_serde::from_slice(bytes).map_err(|e| {
            ProtocolError::SerializationFailed(format!(
                "MessagePack deserialization error: {}",
                e
            ))
        })
    }

    fn serialize_request(&self, req: &Request) -> Result<Vec<u8>> {
        self.serialize(req)
    }

    fn deserialize_request(&self, bytes: &[u8]) -> Result<Request> {
        self.deserialize(bytes)
    }

    fn serialize_response(&self, resp: &Response) -> Result<Vec<u8>> {
        self.serialize(resp)
    }

    fn deserialize_response(&self, bytes: &[u8]) -> Result<Response> {
        self.deserialize(bytes)
    }

    fn serialize_error(&self, err: &ErrorMessage) -> Result<Vec<u8>> {
        self.serialize(err)
    }

    fn deserialize_error(&self, bytes: &[u8]) -> Result<ErrorMessage> {
        self.deserialize(bytes)
    }
}

pub fn default_serializer() -> &'static MessagePackSerializer {
    &MSGPACK
}
