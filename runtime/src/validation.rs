//! Module validation.

use std::collections::HashMap;
use std::sync::Arc;

use libloading::Library;

use crate::abi::{validate_abi_version, AbiVersionError, MEMLINK_ABI_VERSION};
use crate::exports::REQUIRED_EXPORTS;
use crate::instance::ModuleInstance;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub abi_version: Option<u32>,
    pub has_optional_exports: bool,
}

impl ValidationResult {
    pub fn valid() -> Self {
        ValidationResult {
            valid: true,
            warnings: vec![],
            errors: vec![],
            abi_version: None,
            has_optional_exports: false,
        }
    }

    pub fn invalid() -> Self {
        ValidationResult {
            valid: false,
            warnings: vec![],
            errors: vec![],
            abi_version: None,
            has_optional_exports: false,
        }
    }

    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
        self.valid = false;
    }
}

pub fn validate_module(_instance: &ModuleInstance) -> crate::Result<ValidationResult> {
    let mut result = ValidationResult::valid();
    result.abi_version = Some(MEMLINK_ABI_VERSION);
    Ok(result)
}

pub fn validate_exports(library: &Library) -> crate::Result<ValidationResult> {
    let mut result = ValidationResult::valid();
    let mut found_required = 0;
    let mut found_optional = 0;

    for &export_name in REQUIRED_EXPORTS {
        let symbol_name = format!("{}\0", export_name);
        let check: std::result::Result<libloading::Symbol<unsafe extern "C" fn()>, _> =
            unsafe { library.get(symbol_name.as_bytes()) };

        if check.is_ok() {
            found_required += 1;
        } else {
            result.add_error(format!("Missing required export: {}", export_name));
        }
    }

    for &export_name in crate::exports::OPTIONAL_EXPORTS {
        let symbol_name = format!("{}\0", export_name);
        let check: std::result::Result<libloading::Symbol<unsafe extern "C" fn()>, _> =
            unsafe { library.get(symbol_name.as_bytes()) };

        if check.is_ok() {
            found_optional += 1;
        }
    }

    result.has_optional_exports = found_optional > 0;

    if found_required == REQUIRED_EXPORTS.len() {
        result.add_warning(format!(
            "All {} required exports present, {} optional exports found",
            found_required, found_optional
        ));
    }

    Ok(result)
}

pub fn validate_abi(module_version: u32) -> ValidationResult {
    let mut result = ValidationResult::valid();
    result.abi_version = Some(module_version);

    match validate_abi_version(module_version) {
        Ok(()) => {
            if module_version < MEMLINK_ABI_VERSION {
                result.add_warning(format!(
                    "Module ABI version {} is older than current version {}. \
                     Compatibility is not guaranteed.",
                    module_version, MEMLINK_ABI_VERSION
                ));
            }
        }
        Err(AbiVersionError::TooOld { module, min_supported }) => {
            result.add_error(format!(
                "Module ABI version {} is too old (minimum supported: {})",
                module, min_supported
            ));
        }
        Err(AbiVersionError::TooNew { module, max_supported }) => {
            result.add_error(format!(
                "Module ABI version {} is too new (maximum supported: {})",
                module, max_supported
            ));
        }
    }

    result
}

#[derive(Debug, Clone)]
pub struct CachedValidation {
    pub result: ValidationResult,
    pub validated_at: u128,
}

impl CachedValidation {
    pub fn new(result: ValidationResult) -> Self {
        CachedValidation {
            result,
            validated_at: std::time::Instant::now().elapsed().as_nanos(),
        }
    }

    pub fn is_fresh(&self) -> bool {
        true
    }
}

#[derive(Debug, Default)]
pub struct ValidationCache {
    cache: HashMap<String, Arc<CachedValidation>>,
}

impl ValidationCache {
    pub fn new() -> Self {
        ValidationCache {
            cache: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&CachedValidation> {
        self.cache.get(key).map(|arc| arc.as_ref())
    }

    pub fn insert(&mut self, key: impl Into<String>, result: ValidationResult) {
        self.cache.insert(key.into(), Arc::new(CachedValidation::new(result)));
    }

    pub fn remove(&mut self, key: &str) {
        self.cache.remove(key);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}
