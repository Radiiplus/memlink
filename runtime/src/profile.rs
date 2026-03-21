//! Module profile - configuration and metadata for loaded modules.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleProfile {
    pub name: String,
    pub baseline_memory_mb: u64,
    pub max_memory_mb: u64,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl Default for ModuleProfile {
    fn default() -> Self {
        ModuleProfile {
            name: "unknown".to_string(),
            baseline_memory_mb: 0,
            max_memory_mb: 64,
            version: None,
            description: None,
        }
    }
}

impl ModuleProfile {
    pub fn new(name: impl Into<String>) -> Self {
        ModuleProfile {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn with_memory_limits(mut self, baseline_mb: u64, max_mb: u64) -> Self {
        self.baseline_memory_mb = baseline_mb;
        self.max_memory_mb = max_mb;
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn memory_usage_ratio(&self, current_mb: u64) -> f32 {
        if self.max_memory_mb == 0 {
            0.0
        } else {
            current_mb as f32 / self.max_memory_mb as f32
        }
    }

    pub fn is_over_memory_limit(&self, current_mb: u64) -> bool {
        current_mb > self.max_memory_mb
    }
}
