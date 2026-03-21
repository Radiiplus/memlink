//! Memory layout constants for shared memory segment structure.
//! Control region at offset 0 (4KB), ring buffer follows.

pub const CONTROL_REGION_SIZE: usize = 4096;
pub const RING_BUFFER_OFFSET: usize = CONTROL_REGION_SIZE;
pub const MIN_SEGMENT_SIZE: usize = CONTROL_REGION_SIZE + 1;
pub const DEFAULT_SEGMENT_SIZE: usize = CONTROL_REGION_SIZE + 1024 * 1024;
pub const MAX_SEGMENT_SIZE: usize = CONTROL_REGION_SIZE + 256 * 1024 * 1024;
pub const PAGE_SIZE: usize = 4096;

pub const fn ring_buffer_size(segment_size: usize) -> Option<usize> {
    if segment_size < MIN_SEGMENT_SIZE {
        None
    } else {
        Some(segment_size - RING_BUFFER_OFFSET)
    }
}

pub const fn segment_size_for_ring_buffer(ring_size: usize) -> usize {
    let total = RING_BUFFER_OFFSET + ring_size;
    (total + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}
