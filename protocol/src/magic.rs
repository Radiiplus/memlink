//! Magic numbers and protocol constants.
//!
//! Defines fundamental constants including magic number, protocol version,
//! header size, and payload limits.

pub const MEMLINK_MAGIC: u32 = 0x4D4C4E4B;

pub const PROTOCOL_VERSION: u8 = 1;

pub const HEADER_SIZE: usize = 32;

pub const MAX_PAYLOAD_SIZE: usize = 64 * 1024 * 1024;

pub const CONTROL_REGION_SIZE: usize = 4096;

pub const MIN_PROTOCOL_VERSION: u8 = 1;
