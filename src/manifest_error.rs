use std::{fmt::Display, io};

/// Errors that can occur during manifest operations.
///
/// Covers serialization, validation, and I/O errors when working with manifest files.
#[derive(Debug)]
pub enum ManifestError {
    /// JSON serialization or deserialization failed
    SerializationError(serde_json::Error),
    /// Manifest content validation failed (e.g., duplicate ports, invalid names)
    ValidationError(String),
    /// File I/O operation failed (reading or writing manifest files)
    IoError(io::Error),
}

impl From<serde_json::Error> for ManifestError {
    fn from(err: serde_json::Error) -> Self {
        ManifestError::SerializationError(err)
    }
}

impl From<io::Error> for ManifestError {
    fn from(err: io::Error) -> Self {
        ManifestError::IoError(err)
    }
}

impl Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::SerializationError(err) => {
                write!(f, "Manifest serialization error: {}", err)
            }
            ManifestError::ValidationError(msg) => write!(f, "Manifest validation error: {}", msg),
            ManifestError::IoError(err) => write!(f, "Manifest IO error: {}", err),
        }
    }
}

impl std::error::Error for ManifestError {}
