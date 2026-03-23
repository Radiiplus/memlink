//! Serialization interface for memlink modules.
//!
//! Re-exports MessagePack serializer from memlink-protocol for
//! efficient IPC communication.

pub use memlink_protocol::msgpack::{default_serializer, MessagePackSerializer, MSGPACK};
pub use memlink_protocol::serializer::Serializer;
