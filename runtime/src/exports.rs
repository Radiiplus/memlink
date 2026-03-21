//! Required and optional module exports definition.

pub const REQUIRED_EXPORTS: &[&str] = &[
    "memlink_init",
    "memlink_call",
    "memlink_shutdown",
];

pub const OPTIONAL_EXPORTS: &[&str] = &[
    "memlink_get_state_size",
    "memlink_serialize_state",
    "memlink_deserialize_state",
];

pub fn is_required_export(name: &str) -> bool {
    REQUIRED_EXPORTS.contains(&name)
}

pub fn is_optional_export(name: &str) -> bool {
    OPTIONAL_EXPORTS.contains(&name)
}

pub fn is_known_export(name: &str) -> bool {
    is_required_export(name) || is_optional_export(name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportCategory {
    Required,
    Optional,
    Unknown,
}

pub fn categorize_export(name: &str) -> ExportCategory {
    if is_required_export(name) {
        ExportCategory::Required
    } else if is_optional_export(name) {
        ExportCategory::Optional
    } else {
        ExportCategory::Unknown
    }
}
