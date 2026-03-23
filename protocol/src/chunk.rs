//! Chunked transfer for large streams (Phase 2).
//!
//! Defines Chunk and ChunkedStream structs for streaming large payloads.
//! Currently reserved/unimplemented - placeholders for future development.

use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub offset: u64,
    pub data: Vec<u8>,
}

impl Chunk {
    #[allow(dead_code)]
    pub fn new(offset: u64, data: Vec<u8>) -> Self {
        Self { offset, data }
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.data.len()
    }

    #[allow(dead_code)]
    pub fn is_last(&self, total_size: u64) -> bool {
        self.offset + self.data.len() as u64 >= total_size
    }

    #[allow(dead_code)]
    pub fn serialize(&self) -> Vec<u8> {
        unimplemented!("Chunk serialization will be implemented in Phase 2")
    }

    #[allow(dead_code)]
    pub fn deserialize(_bytes: &[u8]) -> Option<Self> {
        unimplemented!("Chunk deserialization will be implemented in Phase 2")
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ChunkFlags {
    None = 0,
    Last = 1,
    Compressed = 2,
    RequiresAck = 4,
    Retransmit = 8,
}

impl ChunkFlags {
    #[allow(dead_code)]
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => ChunkFlags::Last,
            2 => ChunkFlags::Compressed,
            4 => ChunkFlags::RequiresAck,
            8 => ChunkFlags::Retransmit,
            _ => ChunkFlags::None,
        }
    }

    #[allow(dead_code)]
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    #[allow(dead_code)]
    pub fn has_flag(flags: u8, flag: ChunkFlags) -> bool {
        flags & flag.as_u8() != 0
    }

    #[allow(dead_code)]
    pub fn set_flag(flags: &mut u8, flag: ChunkFlags) {
        *flags |= flag.as_u8();
    }

    #[allow(dead_code)]
    pub fn clear_flag(flags: &mut u8, flag: ChunkFlags) {
        *flags &= !flag.as_u8();
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChunkedStream {
    pub total_size: u64,
    pub transferred: u64,
    pub current_offset: u64,
    pub chunk_size: usize,
}

#[allow(dead_code)]
impl ChunkedStream {
    pub fn new(total_size: u64, chunk_size: usize) -> Self {
        Self {
            total_size,
            transferred: 0,
            current_offset: 0,
            chunk_size,
        }
    }

    pub fn progress(&self) -> f64 {
        if self.total_size == 0 {
            return 1.0;
        }
        self.transferred as f64 / self.total_size as f64
    }

    pub fn remaining(&self) -> u64 {
        self.total_size.saturating_sub(self.transferred)
    }

    pub fn is_complete(&self) -> bool {
        self.transferred >= self.total_size
    }
}
