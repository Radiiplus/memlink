//! Structured logging for memlink modules, exporting logs to the daemon for aggregation.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

pub fn log(_level: Level, _message: &str, _fields: &[(&str, &str)]) {}

pub fn debug(message: &str, fields: &[(&str, &str)]) {
    log(Level::Debug, message, fields);
}

pub fn info(message: &str, fields: &[(&str, &str)]) {
    log(Level::Info, message, fields);
}

pub fn warn(message: &str, fields: &[(&str, &str)]) {
    log(Level::Warn, message, fields);
}

pub fn error(message: &str, fields: &[(&str, &str)]) {
    log(Level::Error, message, fields);
}
