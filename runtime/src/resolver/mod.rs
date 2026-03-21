//! Module resolution - locating and validating module artifacts.

pub mod chain;
pub mod local;

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleRef {
    LocalPath(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactHandle {
    Local(PathBuf),
}

impl ArtifactHandle {
    pub fn path(&self) -> &Path {
        match self {
            ArtifactHandle::Local(path) => path,
        }
    }
}

pub trait ModuleResolver: Send + Sync {
    fn resolve(&self, reference: ModuleRef) -> Result<ArtifactHandle>;
}

impl ModuleRef {
    pub fn parse(spec: &str) -> Result<Self> {
        if spec.contains('@') && !spec.contains('/') && !spec.contains('\\') && !spec.starts_with('.') {
            return Err(Error::RegistryNotImplemented);
        }

        let is_path = spec.contains('/')
            || spec.contains('\\')
            || spec.starts_with('.')
            || spec.starts_with("..");

        if !is_path {
            return Err(Error::UnsupportedReference);
        }

        Ok(ModuleRef::LocalPath(PathBuf::from(spec)))
    }
}
