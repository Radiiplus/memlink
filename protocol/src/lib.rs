//! MemLink Protocol - Binary protocol definitions for high-performance IPC.

#![no_std]
#![allow(missing_docs)]
#![allow(clippy::missing_safety_doc)]
#![warn(missing_debug_implementations)]
#![deny(unused_qualifications)]

extern crate alloc;

pub mod error;
pub mod header;
pub mod magic;
pub mod types;
pub mod request;
pub mod response;
pub mod emsg;
pub mod serializer;
pub mod msgpack;
pub mod config;
pub mod shm;
pub mod zero;
pub mod arena;
pub mod shandle;
pub mod sproto;
pub mod chunk;
pub mod version;
pub mod features;
pub mod negotiation;

pub use error::{ProtocolError, Result};
pub use header::MessageHeader;
pub use magic::*;
pub use types::*;
pub use emsg::ErrorMessage;
pub use request::Request;
pub use response::Response;
pub use msgpack::{default_serializer, MessagePackSerializer, MSGPACK};
pub use serializer::Serializer;
pub use config::SerializerConfig;
pub use shm::ShmView;
pub use zero::{ZeroCopyRequest, ZeroCopyResponse};
pub use arena::ArenaSlice;
pub use shandle::{StreamHandle, StreamError};
pub use sproto::*;
pub use version::{ProtocolVersion, CURRENT_VERSION, MAX_VERSION, MIN_VERSION, SUPPORTED_VERSIONS, V1_0, V1_1, V1_2};
pub use features::{has_feature, FEATURE_NONE};
pub use negotiation::{negotiate_version, validate_version};

pub mod prelude {
    pub use crate::arena::{ArenaConfig, ArenaRef, ArenaSlice};
    pub use crate::chunk::{Chunk, ChunkFlags, ChunkedStream};
    pub use crate::config::SerializerConfig;
    pub use crate::error::{ProtocolError, Result};
    pub use crate::features::{has_feature, feature_flags, FEATURE_NONE, BATCHING, STREAMING};
    pub use crate::header::MessageHeader;
    pub use crate::magic::{
        CONTROL_REGION_SIZE, HEADER_SIZE, MAX_PAYLOAD_SIZE, MEMLINK_MAGIC, MIN_PROTOCOL_VERSION,
        PROTOCOL_VERSION,
    };
    pub use crate::msgpack::{default_serializer, MessagePackSerializer, MSGPACK};
    pub use crate::negotiation::{negotiate_version, validate_version};
    pub use crate::serializer::Serializer;
    #[cfg(feature = "shm")]
    pub use crate::shm::{
        RingBuffer, ShmPriority, Platform, MmapSegment, ControlRegion,
        Futex, PriorityRingBuffer, ShmTransport,
    };
    pub use crate::shm::{is_aligned, ShmView, SHM_ALIGNMENT};
    pub use crate::shandle::{StreamHandle, StreamError};
    pub use crate::sproto::{
        StreamState, StreamMode, STREAM_HANDLE_SIZE, STREAM_HEADER_SIZE,
        DEFAULT_STREAM_TIMEOUT_NS, MAX_STREAM_TIMEOUT_NS,
    };
    pub use crate::types::{
        MethodHash, MessageType, ModuleId, Priority, RequestId, SpanId, StatusCode, TraceId,
    };
    pub use crate::version::{
        ProtocolVersion, CURRENT_VERSION, MAX_VERSION, MIN_VERSION, SUPPORTED_VERSIONS, V1_0, V1_1,
        V1_2,
    };
    pub use crate::zero::{ZeroCopyRequest, ZeroCopyResponse};
    pub use crate::{ErrorMessage, Request, Response};
}
