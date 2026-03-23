//! Fixed 32-byte message header.
//!
//! Defines MessageHeader struct with methods for creation, validation,
//! serialization (as_bytes), and deserialization (from_bytes).

use core::mem::size_of;

use crate::error::{ProtocolError, Result};
use crate::magic::{HEADER_SIZE, MAX_PAYLOAD_SIZE, MEMLINK_MAGIC, PROTOCOL_VERSION};
use crate::types::{MethodHash, ModuleId, MessageType, RequestId};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MessageHeader {
    magic: u32,
    version: u8,
    msg_type: u8,
    features: u16,
    request_id: u64,
    module_id: u64,
    method_hash: u32,
    payload_len: u32,
}

const _: () = {
    assert!(
        size_of::<MessageHeader>() == HEADER_SIZE,
        "MessageHeader must be exactly 32 bytes"
    );
};

impl MessageHeader {
    pub fn new(
        msg_type: MessageType,
        request_id: RequestId,
        module_id: ModuleId,
        method_hash: MethodHash,
        payload_len: u32,
    ) -> Self {
        Self {
            magic: MEMLINK_MAGIC,
            version: PROTOCOL_VERSION,
            msg_type: msg_type.as_u8(),
            features: 0,
            request_id,
            module_id,
            method_hash,
            payload_len,
        }
    }

    pub fn with_features(
        msg_type: MessageType,
        features: u16,
        request_id: RequestId,
        module_id: ModuleId,
        method_hash: MethodHash,
        payload_len: u32,
    ) -> Self {
        Self {
            magic: MEMLINK_MAGIC,
            version: PROTOCOL_VERSION,
            msg_type: msg_type.as_u8(),
            features,
            request_id,
            module_id,
            method_hash,
            payload_len,
        }
    }

    pub fn magic(&self) -> u32 {
        self.magic
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn msg_type(&self) -> u8 {
        self.msg_type
    }

    pub fn message_type(&self) -> Option<MessageType> {
        MessageType::from_u8(self.msg_type)
    }

    pub fn features(&self) -> u16 {
        self.features
    }

    pub fn has_feature(&self, feature: u16) -> bool {
        self.features & feature != 0
    }

    pub fn request_id(&self) -> RequestId {
        self.request_id
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn method_hash(&self) -> MethodHash {
        self.method_hash
    }

    pub fn payload_len(&self) -> u32 {
        self.payload_len
    }

    pub fn validate(&self) -> Result<()> {
        if self.magic != MEMLINK_MAGIC {
            return Err(ProtocolError::InvalidMagic(self.magic));
        }

        if self.version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(self.version));
        }

        if MessageType::from_u8(self.msg_type).is_none() {
            return Err(ProtocolError::InvalidMessageType(self.msg_type));
        }

        if self.payload_len as usize > MAX_PAYLOAD_SIZE {
            return Err(ProtocolError::PayloadTooLarge(
                self.payload_len as usize,
                MAX_PAYLOAD_SIZE,
            ));
        }

        Ok(())
    }

    pub fn as_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];

        bytes[0..4].copy_from_slice(&self.magic.to_be_bytes());
        bytes[4] = self.version;
        bytes[5] = self.msg_type;
        bytes[6..8].copy_from_slice(&self.features.to_be_bytes());
        bytes[8..16].copy_from_slice(&self.request_id.to_be_bytes());
        bytes[16..24].copy_from_slice(&self.module_id.to_be_bytes());
        bytes[24..28].copy_from_slice(&self.method_hash.to_be_bytes());
        bytes[28..32].copy_from_slice(&self.payload_len.to_be_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &[u8; HEADER_SIZE]) -> Result<Self> {
        let magic = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let version = bytes[4];
        let msg_type = bytes[5];
        let features = u16::from_be_bytes([bytes[6], bytes[7]]);
        let request_id =
            u64::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]);
        let module_id =
            u64::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23]]);
        let method_hash = u32::from_be_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let payload_len = u32::from_be_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);

        let header = Self {
            magic,
            version,
            msg_type,
            features,
            request_id,
            module_id,
            method_hash,
            payload_len,
        };

        header.validate()?;

        Ok(header)
    }

    pub fn request(
        request_id: RequestId,
        module_id: ModuleId,
        method_hash: MethodHash,
        payload_len: u32,
    ) -> Self {
        Self::new(
            MessageType::Request,
            request_id,
            module_id,
            method_hash,
            payload_len,
        )
    }

    pub fn response(
        request_id: RequestId,
        module_id: ModuleId,
        method_hash: MethodHash,
        payload_len: u32,
    ) -> Self {
        Self::new(
            MessageType::Response,
            request_id,
            module_id,
            method_hash,
            payload_len,
        )
    }

    pub fn notify(
        module_id: ModuleId,
        method_hash: MethodHash,
        payload_len: u32,
        features: u16,
    ) -> Self {
        Self::with_features(
            MessageType::Event,
            features,
            0,
            module_id,
            method_hash,
            payload_len,
        )
    }

    pub fn heartbeat() -> Self {
        Self::new(
            MessageType::HealthCheck,
            0,
            0,
            0,
            0,
        )
    }
}
