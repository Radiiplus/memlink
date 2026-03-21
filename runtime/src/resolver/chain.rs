//! Chain resolver for composite module resolution.

use crate::error::{Error, Result};
use crate::resolver::{ArtifactHandle, ModuleRef, ModuleResolver};

pub struct ChainResolver {
    resolvers: Vec<Box<dyn ModuleResolver>>,
}

impl ChainResolver {
    pub fn new() -> Self {
        ChainResolver {
            resolvers: Vec::new(),
        }
    }

    pub fn add<R: ModuleResolver + 'static>(&mut self, resolver: R) {
        self.resolvers.push(Box::new(resolver));
    }

    pub fn len(&self) -> usize {
        self.resolvers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.resolvers.is_empty()
    }
}

impl Default for ChainResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleResolver for ChainResolver {
    fn resolve(&self, reference: ModuleRef) -> Result<ArtifactHandle> {
        if self.resolvers.is_empty() {
            return Err(Error::NoResolverFound);
        }

        let mut last_error: Option<Error> = None;

        for resolver in &self.resolvers {
            match resolver.resolve(reference.clone()) {
                Ok(handle) => return Ok(handle),
                Err(Error::NotOurs) => {
                    continue;
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or(Error::NoResolverFound))
    }
}
