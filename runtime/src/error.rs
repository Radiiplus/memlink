//! Error types for the memlink runtime.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Module file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid module format: {0}")]
    InvalidModuleFormat(String),

    #[error("Unsupported module reference format")]
    UnsupportedReference,

    #[error("Registry-based module resolution is not implemented")]
    RegistryNotImplemented,

    #[error("No resolver found that can handle the module reference")]
    NoResolverFound,

    #[error("This resolver cannot handle the request; delegation intended")]
    NotOurs,

    #[error("Failed to load shared library: {0}")]
    LibraryLoadFailed(String),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Module call failed with error code: {0}")]
    ModuleCallFailed(i32),

    #[error("Module panicked: {0}")]
    ModulePanicked(String),

    #[error("Reload timed out waiting for {0} in-flight calls to complete")]
    ReloadTimeout(usize),

    #[error("Reload already in progress for module {0}")]
    ReloadInProgress(u64),
}

pub type Result<T> = std::result::Result<T, Error>;
