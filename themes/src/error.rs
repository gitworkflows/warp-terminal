//! Error types for theme operations

use std::fmt;

/// Result type alias for theme operations
pub type Result<T> = std::result::Result<T, ThemeError>;

/// Errors that can occur during theme operations
#[derive(Debug, Clone)]
pub enum ThemeError {
    /// Theme file not found
    ThemeNotFound(String),
    /// Invalid theme format
    InvalidFormat(String),
    /// Invalid color specification
    InvalidColor(String),
    /// IO error
    IoError(String),
    /// YAML parsing error
    ParseError(String),
    /// Theme validation error
    ValidationError(String),
    /// Missing required field
    MissingField(String),
    /// Unsupported theme version
    UnsupportedVersion(String),
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeError::ThemeNotFound(name) => {
                write!(f, "Theme not found: {}", name)
            }
            ThemeError::InvalidFormat(msg) => {
                write!(f, "Invalid theme format: {}", msg)
            }
            ThemeError::InvalidColor(msg) => {
                write!(f, "Invalid color: {}", msg)
            }
            ThemeError::IoError(msg) => {
                write!(f, "IO error: {}", msg)
            }
            ThemeError::ParseError(msg) => {
                write!(f, "Parse error: {}", msg)
            }
            ThemeError::ValidationError(msg) => {
                write!(f, "Validation error: {}", msg)
            }
            ThemeError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
            ThemeError::UnsupportedVersion(version) => {
                write!(f, "Unsupported theme version: {}", version)
            }
        }
    }
}

impl std::error::Error for ThemeError {}

impl From<std::io::Error> for ThemeError {
    fn from(error: std::io::Error) -> Self {
        ThemeError::IoError(error.to_string())
    }
}

impl From<serde_yaml::Error> for ThemeError {
    fn from(error: serde_yaml::Error) -> Self {
        ThemeError::ParseError(error.to_string())
    }
}
