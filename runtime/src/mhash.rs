//! Method hashing for FFI-efficient dispatch.

const FNV_PRIME: u32 = 16777619;
const FNV_OFFSET_BASIS: u32 = 2166136261;

pub fn fnv1a_hash(method_name: &str) -> u32 {
    let mut hash = FNV_OFFSET_BASIS;

    for byte in method_name.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    hash
}

pub fn fnv1a_hash_bytes(data: &[u8]) -> u32 {
    let mut hash = FNV_OFFSET_BASIS;

    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    hash
}

pub fn verify_method_id(method_id: u32, method_name: &str) -> bool {
    fnv1a_hash(method_name) == method_id
}
