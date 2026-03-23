//! Version negotiation logic.
//!
//! Provides functions for negotiating protocol versions between clients
//! and servers, including feature intersection and validation.

use crate::error::{ProtocolError, Result};
use crate::features::intersect_features;
use crate::version::{ProtocolVersion, SUPPORTED_VERSIONS};

pub fn negotiate_version(client: &ProtocolVersion, server: &ProtocolVersion) -> Result<ProtocolVersion> {
    if client.major() != server.major() {
        return Err(ProtocolError::UnsupportedVersion(
            client.major().max(server.major()) as u8,
        ));
    }

    let negotiated_minor = client.minor().min(server.minor());
    let common_features = intersect_features(client.features(), server.features());

    Ok(ProtocolVersion::new(client.major(), negotiated_minor, common_features))
}

pub fn negotiate_with_server_versions(
    client: &ProtocolVersion,
    server_versions: &[ProtocolVersion],
) -> Result<ProtocolVersion> {
    let mut best_version: Option<ProtocolVersion> = None;

    for server_version in server_versions {
        if let Ok(negotiated) = negotiate_version(client, server_version) {
            if best_version.is_none() || negotiated.compare(&best_version.unwrap()) > 0 {
                best_version = Some(negotiated);
            }
        }
    }

    best_version.ok_or_else(|| ProtocolError::UnsupportedVersion(client.major() as u8))
}

pub fn validate_version(version: &ProtocolVersion) -> Result<()> {
    for supported in SUPPORTED_VERSIONS {
        if version.major() == supported.major() && version.minor() <= supported.minor() {
            return Ok(());
        }
    }

    Err(ProtocolError::UnsupportedVersion(version.major() as u8))
}

pub fn is_feature_supported(feature: u32) -> bool {
    feature != 0 && feature <= 0x00000007
}

pub fn version_string(version: &ProtocolVersion) -> alloc::string::String {
    use alloc::format;
    format!("MemLink/{}.{}", version.major(), version.minor())
}
