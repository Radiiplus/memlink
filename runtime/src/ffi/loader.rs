//! Module loader - dynamic library loading via libloading.

use libloading::Library;
use std::path::Path;

use crate::error::{Error, Result};
use crate::ffi::symbols::{LibHandle, ModuleSymbols, ModuleCallFn, ModuleInitFn, ModuleShutdownFn};
use crate::resolver::ArtifactHandle;

pub struct ModuleLoader {
    _private: (),
}

impl ModuleLoader {
    pub fn new() -> Self {
        ModuleLoader { _private: () }
    }

    pub fn load(&self, artifact: ArtifactHandle) -> Result<crate::instance::ModuleInstance> {
        let path = artifact.path().to_path_buf();

        let library = unsafe { Self::dlopen(&path)? };
        let symbols = unsafe { Self::load_symbols(library.inner())? };

        Ok(crate::instance::ModuleInstance::new(
            library.clone(),
            symbols,
            path,
        ))
    }

    unsafe fn dlopen(path: &Path) -> Result<LibHandle> {
        Library::new(path)
            .map(LibHandle::new)
            .map_err(|e| {
                Error::LibraryLoadFailed(format!(
                    "Failed to load library at '{}': {}",
                    path.display(),
                    e
                ))
            })
    }

    unsafe fn load_symbols(library: &Library) -> Result<ModuleSymbols<'_>> {
        let memlink_init: libloading::Symbol<ModuleInitFn> = library
            .get(b"memlink_init\0")
            .map_err(|e| {
                Error::SymbolNotFound(format!("memlink_init: {}", e))
            })?;

        let memlink_call: libloading::Symbol<ModuleCallFn> = library
            .get(b"memlink_call\0")
            .map_err(|e| {
                Error::SymbolNotFound(format!("memlink_call: {}", e))
            })?;

        let memlink_shutdown: libloading::Symbol<ModuleShutdownFn> = library
            .get(b"memlink_shutdown\0")
            .map_err(|e| {
                Error::SymbolNotFound(format!("memlink_shutdown: {}", e))
            })?;

        Ok(ModuleSymbols::new(
            memlink_init,
            memlink_call,
            memlink_shutdown,
        ))
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}
