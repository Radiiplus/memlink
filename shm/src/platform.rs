//! Platform detection for selecting OS-specific implementations at compile time.

use cfg_if::cfg_if;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
}

impl Platform {
    pub fn current() -> Self {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                Platform::Linux
            } else if #[cfg(target_os = "macos")] {
                Platform::MacOS
            } else if #[cfg(target_os = "windows")] {
                Platform::Windows
            } else {
                compile_error!("Unsupported platform. Only Linux, macOS, and Windows are supported.");
            }
        }
    }

    pub fn is_unix(&self) -> bool {
        matches!(self, Platform::Linux | Platform::MacOS)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Linux => "linux",
            Platform::MacOS => "macos",
            Platform::Windows => "windows",
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
