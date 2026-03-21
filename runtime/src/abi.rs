//! Module ABI version checking and compatibility validation.

pub const MEMLINK_ABI_VERSION: u32 = 1;
pub const MIN_SUPPORTED_ABI_VERSION: u32 = 1;
pub const MAX_SUPPORTED_ABI_VERSION: u32 = 1;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct AbiInfo {
    pub version: u32,
    pub flags: u32,
}

impl AbiInfo {
    pub fn new() -> Self {
        AbiInfo {
            version: MEMLINK_ABI_VERSION,
            flags: 0,
        }
    }

    pub fn with_version_and_flags(version: u32, flags: u32) -> Self {
        AbiInfo { version, flags }
    }

    pub fn is_compatible(&self) -> bool {
        self.version >= MIN_SUPPORTED_ABI_VERSION && self.version <= MAX_SUPPORTED_ABI_VERSION
    }

    pub fn supports_state_serialization(&self) -> bool {
        self.flags & 0x01 != 0
    }

    pub fn supports_async(&self) -> bool {
        self.flags & 0x02 != 0
    }

    pub fn mismatch_severity(&self) -> AbiMismatchSeverity {
        if self.is_compatible() {
            AbiMismatchSeverity::Compatible
        } else if self.version < MIN_SUPPORTED_ABI_VERSION {
            AbiMismatchSeverity::TooOld
        } else {
            AbiMismatchSeverity::TooNew
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiMismatchSeverity {
    Compatible,
    TooOld,
    TooNew,
}

pub fn validate_abi_version(module_version: u32) -> Result<(), AbiVersionError> {
    if module_version >= MIN_SUPPORTED_ABI_VERSION && module_version <= MAX_SUPPORTED_ABI_VERSION {
        Ok(())
    } else if module_version < MIN_SUPPORTED_ABI_VERSION {
        Err(AbiVersionError::TooOld {
            module: module_version,
            min_supported: MIN_SUPPORTED_ABI_VERSION,
        })
    } else {
        Err(AbiVersionError::TooNew {
            module: module_version,
            max_supported: MAX_SUPPORTED_ABI_VERSION,
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum AbiVersionError {
    #[error("Module ABI version {module} is too old (minimum supported: {min_supported})")]
    TooOld {
        module: u32,
        min_supported: u32,
    },

    #[error("Module ABI version {module} is too new (maximum supported: {max_supported})")]
    TooNew {
        module: u32,
        max_supported: u32,
    },
}
