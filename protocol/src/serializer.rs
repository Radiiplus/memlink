//! Serializer trait for pluggable encoding.
//!
//! Defines Serializer trait with methods for serializing/deserializing
//! Request, Response, and ErrorMessage types.

use alloc::vec::Vec;

use crate::emsg::ErrorMessage;
use crate::error::Result;
use crate::request::Request;
use crate::response::Response;

pub trait Serializer: Send + Sync {
    fn serialize<T: serde::Serialize>(&self, value: &T) -> Result<Vec<u8>>;

    fn deserialize<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T>;

    fn serialize_request(&self, req: &Request) -> Result<Vec<u8>>;

    fn deserialize_request(&self, bytes: &[u8]) -> Result<Request>;

    fn serialize_response(&self, resp: &Response) -> Result<Vec<u8>>;

    fn deserialize_response(&self, bytes: &[u8]) -> Result<Response>;

    fn serialize_error(&self, err: &ErrorMessage) -> Result<Vec<u8>>;

    fn deserialize_error(&self, bytes: &[u8]) -> Result<ErrorMessage>;
}

pub trait MemLinkSerialize: serde::Serialize + serde::de::DeserializeOwned {}

impl<T> MemLinkSerialize for T where T: serde::Serialize + serde::de::DeserializeOwned {}
