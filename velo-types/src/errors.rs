// VeloError type
// Requirement 1.6

use std::fmt;

#[derive(Debug, Clone)]
pub enum VeloError {
    InvalidRange,
    InvalidBufferId,
    InvalidPosition,
    NoFilePath,
    NoBuffersOpen,
    UndoNotSupported,
    IoError(String),
    ParseError(String),
    PluginError(String),
    ConfigError(String),
    Other(String),
}

impl fmt::Display for VeloError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VeloError::InvalidRange => write!(f, "Invalid range"),
            VeloError::InvalidBufferId => write!(f, "Invalid buffer ID"),
            VeloError::InvalidPosition => write!(f, "Invalid position"),
            VeloError::NoFilePath => write!(f, "No file path"),
            VeloError::NoBuffersOpen => write!(f, "No buffers open"),
            VeloError::UndoNotSupported => write!(f, "Undo not supported for this command"),
            VeloError::IoError(msg) => write!(f, "I/O error: {}", msg),
            VeloError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            VeloError::PluginError(msg) => write!(f, "Plugin error: {}", msg),
            VeloError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            VeloError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for VeloError {}
