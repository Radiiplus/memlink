//! Memlink runtime - dynamic module loading and execution.

pub mod abi;
pub mod arena;
pub mod error;
pub mod exports;
pub mod ffi;
pub mod instance;
pub mod mhash;
pub mod metrics;
pub mod panic;
pub mod profile;
pub mod reload;
pub mod resolver;
pub mod runtime;
pub mod safety;
pub mod validation;

pub use abi::{AbiInfo, AbiVersionError, MEMLINK_ABI_VERSION};
pub use arena::Arena;
pub use error::{Error, Result};
pub use exports::{ExportCategory, OPTIONAL_EXPORTS, REQUIRED_EXPORTS};
pub use ffi::{ModuleLoader, ModuleSymbols};
pub use instance::{ModuleInstance, ModuleProfile as InstanceProfile};
pub use mhash::fnv1a_hash;
pub use metrics::{Counter, Histogram, RuntimeMetrics};
pub use panic::{safe_call, safe_call_unchecked, PanicError};
pub use profile::ModuleProfile;
pub use reload::{ReloadConfig, ReloadState};
pub use resolver::{ArtifactHandle, ModuleRef, ModuleResolver};
pub use runtime::{ModuleHandle, ModuleRuntime, ModuleUsage, Runtime};
pub use safety::{MemoryTracker, SafetyConfig, StackDepth};
pub use validation::{validate_module, CachedValidation, ValidationCache, ValidationResult};
