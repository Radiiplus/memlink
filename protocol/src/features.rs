//! Feature flags for protocol capabilities.
//!
//! Defines feature flags (STREAMING, BATCHING, PRIORITY_DEGRADATION)
//! and utility functions for feature bitmap operations.

pub const FEATURE_NONE: u32 = 0;

pub mod feature_flags {
    pub const STREAMING: u32 = 1 << 0;
    pub const BATCHING: u32 = 1 << 1;
    pub const PRIORITY_DEGRADATION: u32 = 1 << 2;
}

pub use feature_flags::{BATCHING, PRIORITY_DEGRADATION, STREAMING};

pub fn has_feature(features: u32, feature: u32) -> bool {
    features & feature != 0
}

pub fn enable_feature(features: u32, feature: u32) -> u32 {
    features | feature
}

pub fn disable_feature(features: u32, feature: u32) -> u32 {
    features & !feature
}

pub fn intersect_features(a: u32, b: u32) -> u32 {
    a & b
}

pub fn union_features(a: u32, b: u32) -> u32 {
    a | b
}

pub fn feature_names(features: u32) -> alloc::vec::Vec<&'static str> {
    let mut names = alloc::vec::Vec::new();

    if has_feature(features, STREAMING) {
        names.push("STREAMING");
    }
    if has_feature(features, BATCHING) {
        names.push("BATCHING");
    }
    if has_feature(features, PRIORITY_DEGRADATION) {
        names.push("PRIORITY_DEGRADATION");
    }

    if names.is_empty() {
        names.push("NONE");
    }

    names
}
