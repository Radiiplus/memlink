//! Serialization interface for memlink modules.
//!
//! Provides the Serializer trait and BincodeSerializer implementation for
//! efficient IPC communication using MessagePack serialization.

use serde::Serialize;

use crate::error::{ModuleError, Result};

pub trait Serializer: Send + Sync {
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>>;
    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T>;
}

pub use serde::de::DeserializeOwned;

#[derive(Debug, Clone, Copy)]
pub struct BincodeSerializer;

impl Serializer for BincodeSerializer {
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        rmp_serde::to_vec(value).map_err(ModuleError::from)
    }

    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        rmp_serde::from_slice(bytes).map_err(ModuleError::from)
    }
}

pub fn default_serializer() -> BincodeSerializer {
    BincodeSerializer
}
