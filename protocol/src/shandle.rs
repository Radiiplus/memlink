//! Stream handle for large payload transfers.
//!
//! Defines StreamHandle struct (80 bytes) with stream_id, total_size,
//! expires_ns, and checksum for referencing out-of-band data streams.

use core::mem::size_of;

use crate::error::Result;
use crate::magic::MEMLINK_MAGIC;
use crate::sproto::{STREAM_ID_SIZE, STREAM_HANDLE_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct StreamHandle {
    pub stream_id: [u8; STREAM_ID_SIZE],
    pub total_size: u64,
    pub expires_ns: u64,
    magic: u32,
    _reserved1: [u8; 4],
    pub checksum: u128,
}

const _: () = {
    assert!(
        size_of::<StreamHandle>() == STREAM_HANDLE_SIZE,
        "StreamHandle must be exactly 80 bytes"
    );
};

impl StreamHandle {
    pub fn new(stream_id: [u8; STREAM_ID_SIZE], total_size: u64, expires_ns: u64) -> Self {
        Self {
            stream_id,
            total_size,
            expires_ns,
            magic: MEMLINK_MAGIC,
            _reserved1: [0; 4],
            checksum: 0,
        }
    }

    pub fn generate(total_size: u64) -> Self {
        let stream_id = generate_random_id();
        let expires_ns = 0;
        Self::new(stream_id, total_size, expires_ns)
    }

    pub fn with_timeout(total_size: u64, timeout_ns: u64) -> Self {
        let stream_id = generate_random_id();
        let expires_ns = get_current_time_ns().saturating_add(timeout_ns);
        Self::new(stream_id, total_size, expires_ns)
    }

    pub fn stream_id(&self) -> &[u8; STREAM_ID_SIZE] {
        &self.stream_id
    }

    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    pub fn checksum(&self) -> u128 {
        self.checksum
    }

    pub fn expires_ns(&self) -> u64 {
        self.expires_ns
    }

    pub fn is_expired(&self) -> bool {
        if self.expires_ns == 0 {
            return false;
        }

        let now = get_current_time_ns();
        now > self.expires_ns
    }

    pub fn validate(&self) -> Result<(), StreamError> {
        if self.magic != MEMLINK_MAGIC {
            return Err(StreamError::InvalidMagic);
        }

        if self.stream_id.iter().all(|&b| b == 0) {
            return Err(StreamError::InvalidStreamId);
        }

        if self.is_expired() {
            return Err(StreamError::StreamExpired);
        }

        Ok(())
    }

    pub fn as_bytes(&self) -> [u8; STREAM_HANDLE_SIZE] {
        let mut bytes = [0u8; STREAM_HANDLE_SIZE];

        bytes[0..32].copy_from_slice(&self.stream_id);
        bytes[32..40].copy_from_slice(&self.total_size.to_le_bytes());
        bytes[40..48].copy_from_slice(&self.expires_ns.to_le_bytes());
        bytes[48..52].copy_from_slice(&self.magic.to_le_bytes());
        bytes[52..56].copy_from_slice(&self._reserved1);
        bytes[56..64].copy_from_slice(&[0u8; 8]);
        bytes[64..80].copy_from_slice(&self.checksum.to_le_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &[u8; STREAM_HANDLE_SIZE]) -> Result<Self, StreamError> {
        let mut stream_id = [0u8; STREAM_ID_SIZE];
        stream_id.copy_from_slice(&bytes[0..32]);

        let total_size = u64::from_le_bytes([
            bytes[32], bytes[33], bytes[34], bytes[35],
            bytes[36], bytes[37], bytes[38], bytes[39],
        ]);

        let expires_ns = u64::from_le_bytes([
            bytes[40], bytes[41], bytes[42], bytes[43],
            bytes[44], bytes[45], bytes[46], bytes[47],
        ]);

        let magic = u32::from_le_bytes([bytes[48], bytes[49], bytes[50], bytes[51]]);
        let _reserved1 = [bytes[52], bytes[53], bytes[54], bytes[55]];

        let checksum = u128::from_le_bytes([
            bytes[64], bytes[65], bytes[66], bytes[67],
            bytes[68], bytes[69], bytes[70], bytes[71],
            bytes[72], bytes[73], bytes[74], bytes[75],
            bytes[76], bytes[77], bytes[78], bytes[79],
        ]);

        let handle = Self {
            stream_id,
            total_size,
            expires_ns,
            magic,
            _reserved1,
            checksum,
        };

        handle.validate()?;

        Ok(handle)
    }

    pub fn set_checksum(&mut self, checksum: u128) {
        self.checksum = checksum;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamError {
    InvalidMagic,
    InvalidStreamId,
    StreamExpired,
    InvalidLength,
}

impl core::fmt::Display for StreamError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StreamError::InvalidMagic => write!(f, "invalid magic number"),
            StreamError::InvalidStreamId => write!(f, "invalid stream ID"),
            StreamError::StreamExpired => write!(f, "stream handle has expired"),
            StreamError::InvalidLength => write!(f, "invalid byte array length"),
        }
    }
}

fn generate_random_id() -> [u8; STREAM_ID_SIZE] {
    let mut id = [0u8; STREAM_ID_SIZE];
    let seed = get_current_time_ns() ^ (MEMLINK_MAGIC as u64);

    let mut state = seed;
    for chunk in id.chunks_mut(8) {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        chunk.copy_from_slice(&state.to_le_bytes());
    }

    id
}

#[inline]
fn get_current_time_ns() -> u64 {
    #[cfg(feature = "std")]
    {
        extern crate std;
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
    #[cfg(not(feature = "std"))]
    {
        0
    }
}
