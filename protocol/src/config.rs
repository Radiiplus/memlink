//! Serialization configuration.
//!
//! Defines SerializerConfig with max_size, reject_unknown_fields,
//! and encoding options for controlling serialization behavior.

use crate::magic::MAX_PAYLOAD_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SerializerConfig {
    pub max_size: usize,
    pub reject_unknown_fields: bool,
    pub use_variable_length_encoding: bool,
}

impl Default for SerializerConfig {
    fn default() -> Self {
        Self::msgpack_default()
    }
}

impl SerializerConfig {
    pub const fn new(
        max_size: usize,
        reject_unknown_fields: bool,
        use_variable_length_encoding: bool,
    ) -> Self {
        Self {
            max_size,
            reject_unknown_fields,
            use_variable_length_encoding,
        }
    }

    pub const fn msgpack_default() -> Self {
        Self {
            max_size: MAX_PAYLOAD_SIZE,
            reject_unknown_fields: false,
            use_variable_length_encoding: true,
        }
    }

    pub const fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    pub const fn with_reject_unknown_fields(mut self, reject: bool) -> Self {
        self.reject_unknown_fields = reject;
        self
    }

    pub const fn with_variable_length_encoding(mut self, use_varint: bool) -> Self {
        self.use_variable_length_encoding = use_varint;
        self
    }

    pub fn validate_size(&self, size: usize) -> crate::error::Result<()> {
        if size > self.max_size {
            Err(crate::error::ProtocolError::PayloadTooLarge(size, self.max_size))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct FlatBufferSerializer {
    _private: (),
}
