//! Local file resolver for shared libraries.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::{Error, Result};
use crate::resolver::{ArtifactHandle, ModuleRef, ModuleResolver};

pub struct LocalResolver;

impl LocalResolver {
    pub fn new() -> Self {
        LocalResolver
    }
}

impl Default for LocalResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleResolver for LocalResolver {
    fn resolve(&self, reference: ModuleRef) -> Result<ArtifactHandle> {
        let ModuleRef::LocalPath(path) = reference;

        if !path.exists() {
            return Err(Error::FileNotFound(path));
        }

        validate_extension(&path)?;
        validate_shared_library(&path)?;

        Ok(ArtifactHandle::Local(path))
    }
}

fn validate_extension(path: &Path) -> Result<()> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        "so" | "dll" | "dylib" => Ok(()),
        _ => Err(Error::InvalidModuleFormat(format!(
            "Invalid extension '.{}': expected .so, .dll, or .dylib",
            extension
        ))),
    }
}

pub fn validate_shared_library(path: &Path) -> Result<()> {
    let mut file = File::open(path).map_err(|e| {
        Error::InvalidModuleFormat(format!("Failed to open file: {}", e))
    })?;

    let mut buffer = [0u8; 16];
    file.read_exact(&mut buffer).map_err(|e| {
        Error::InvalidModuleFormat(format!("Failed to read file header: {}", e))
    })?;

    if buffer[0..4] == [0x7f, 0x45, 0x4c, 0x46] {
        return Ok(());
    }

    if buffer[0..4] == [0xce, 0xfa, 0xed, 0xfe]
        || buffer[0..4] == [0xfe, 0xed, 0xfa, 0xce]
        || buffer[0..4] == [0xcf, 0xfa, 0xed, 0xfe]
        || buffer[0..4] == [0xfe, 0xed, 0xfa, 0xcf]
    {
        return Ok(());
    }

    if buffer[0..2] == [0x4d, 0x5a] {
        return Ok(());
    }

    Err(Error::InvalidModuleFormat(format!(
        "File '{}' does not have valid shared library magic bytes. Found: {:02x?}",
        path.display(),
        &buffer[0..4]
    )))
}
