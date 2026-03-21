//! Module instance - a loaded and callable module.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::arena::Arena;
use crate::error::{Error, Result};
use crate::ffi::symbols::{LibHandle, ModuleSymbols};
use crate::mhash::fnv1a_hash;
use crate::panic::{safe_call_unchecked, setup_panic_hook};

#[derive(Debug, Clone, Default)]
pub struct ModuleProfile {
    pub name: Option<String>,
}

struct ModuleInstanceInner {
    #[allow(dead_code)]
    handle: LibHandle,
    _symbols: ModuleSymbols<'static>,
}

pub struct ModuleInstance {
    inner: Arc<ModuleInstanceInner>,
    path: PathBuf,
    arena: Arena,
    profile: ModuleProfile,
}

unsafe impl Send for ModuleInstance {}
unsafe impl Sync for ModuleInstance {}

impl ModuleInstance {
    pub(crate) fn new(handle: LibHandle, symbols: ModuleSymbols, path: PathBuf) -> Self {
        setup_panic_hook();

        let symbols = unsafe {
            std::mem::transmute::<ModuleSymbols<'_>, ModuleSymbols<'static>>(symbols)
        };

        ModuleInstance {
            inner: Arc::new(ModuleInstanceInner {
                handle,
                _symbols: symbols,
            }),
            path,
            arena: Arena::with_default_capacity(),
            profile: ModuleProfile::default(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn profile(&self) -> &ModuleProfile {
        &self.profile
    }

    pub fn arena(&self) -> &Arena {
        &self.arena
    }

    fn symbols(&self) -> &ModuleSymbols<'static> {
        &self.inner._symbols
    }

    pub fn init(&self, config: &[u8]) -> Result<()> {
        safe_call_unchecked(|| {
            let result = unsafe { (self.symbols().memlink_init)(config.as_ptr(), config.len()) };

            if result == 0 {
                Ok(())
            } else {
                Err(Error::ModuleCallFailed(result))
            }
        })?
    }

    pub fn call(&self, method: &str, args: &[u8]) -> Result<Vec<u8>> {
        let method_id = fnv1a_hash(method);
        let mut output = vec![0u8; 4096];

        safe_call_unchecked(|| {
            let result = unsafe {
                (self.symbols().memlink_call)(method_id, args.as_ptr(), args.len(), output.as_mut_ptr())
            };

            if result == 0 {
                Ok(output)
            } else {
                Err(Error::ModuleCallFailed(result))
            }
        })?
    }

    pub fn shutdown(&self) -> Result<()> {
        safe_call_unchecked(|| {
            let result = unsafe { (self.symbols().memlink_shutdown)() };

            if result == 0 {
                Ok(())
            } else {
                Err(Error::ModuleCallFailed(result))
            }
        })?
    }
}

impl Clone for ModuleInstance {
    fn clone(&self) -> Self {
        ModuleInstance {
            inner: Arc::clone(&self.inner),
            path: self.path.clone(),
            arena: self.arena.clone(),
            profile: self.profile.clone(),
        }
    }
}

impl std::fmt::Debug for ModuleInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleInstance")
            .field("path", &self.path)
            .field("profile", &self.profile)
            .finish()
    }
}

impl Drop for ModuleInstance {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
