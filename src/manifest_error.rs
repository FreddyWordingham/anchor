use std::{fmt::Display, io};

#[derive(Debug)]
pub enum ManifestError {
    SerializationError(serde_json::Error),
    ValidationError(String),
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
