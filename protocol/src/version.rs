//! Protocol versioning and compatibility.
//!
//! Defines ProtocolVersion struct with major/minor/features fields
//! and constants for V1_0, V1_1, V1_2 supported versions.

use crate::features::{BATCHING, STREAMING, FEATURE_NONE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
    pub features: u32,
}

impl ProtocolVersion {
    pub const fn new(major: u16, minor: u16, features: u32) -> Self {
        Self {
            major,
            minor,
            features,
        }
    }

    pub const fn major(&self) -> u16 {
        self.major
    }

    pub const fn minor(&self) -> u16 {
        self.minor
    }

    pub const fn features(&self) -> u32 {
        self.features
    }

    pub fn has_feature(&self, feature: u32) -> bool {
        self.features & feature != 0
    }

    pub fn enable_feature(&mut self, feature: u32) {
        self.features |= feature;
    }

    pub fn disable_feature(&mut self, feature: u32) {
        self.features &= !feature;
    }

    pub fn is_v1_0(&self) -> bool {
        self.major == 1 && self.minor == 0
    }

    pub fn is_compatible_with(&self, other: &ProtocolVersion) -> bool {
        self.major == other.major
    }

    pub fn compare(&self, other: &ProtocolVersion) -> i32 {
        if self.major != other.major {
            return self.major as i32 - other.major as i32;
        }
        if self.minor != other.minor {
            return self.minor as i32 - other.minor as i32;
        }
        0
    }
}

impl core::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

pub const V1_0: ProtocolVersion = ProtocolVersion::new(1, 0, FEATURE_NONE);

pub const V1_1: ProtocolVersion = ProtocolVersion::new(1, 1, STREAMING);

pub const V1_2: ProtocolVersion = ProtocolVersion::new(1, 2, STREAMING | BATCHING);

pub const SUPPORTED_VERSIONS: &[ProtocolVersion] = &[V1_0, V1_1, V1_2];

pub const CURRENT_VERSION: ProtocolVersion = V1_2;

pub const MIN_VERSION: ProtocolVersion = V1_0;

pub const MAX_VERSION: ProtocolVersion = V1_2;
