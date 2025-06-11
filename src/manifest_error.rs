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
        Self::SerializationError(err)
    }
}

impl From<io::Error> for ManifestError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl Display for ManifestError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializationError(err) => {
                write!(fmt, "Manifest serialization error: {err}")
            }
            Self::ValidationError(message) => write!(fmt, "Manifest validation error: {message}"),
            Self::IoError(err) => write!(fmt, "Manifest IO error: {err}"),
        }
    }
}

impl std::error::Error for ManifestError {}
