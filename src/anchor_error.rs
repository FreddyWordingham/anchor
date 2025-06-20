use std::fmt::{Display, Formatter};

/// Result type for Anchor operations, encapsulating `AnchorError`.
pub type AnchorResult<T> = Result<T, AnchorError>;

/// Errors that can occur when interacting with the Docker daemon.
#[derive(Debug)]
pub enum AnchorError {
    /// Docker is not installed on the system.
    DockerNotInstalled,
    /// Error connecting to the Docker daemon.
    ConnectionError(String),
    /// Error retrieving ECR credentials.
    ECRCredentialsError(String),
    /// Error related to a specific Docker image.
    ImageError {
        /// The reference of the Docker image associated with the error.
        image: String,
        /// A message describing the error.
        message: String,
    },
    /// Error related to a specific Docker container.
    ContainerError {
        /// The name of the Docker container associated with the error.
        container: String,
        /// A message describing the error.
        message: String,
    },
    /// IO stream error.
    IoStreamError(String),
}

impl AnchorError {
    /// Create an `ImageError` with context
    pub fn image_error<S: AsRef<str>, M: AsRef<str>>(image: S, message: M) -> Self {
        Self::ImageError {
            image: image.as_ref().to_string(),
            message: message.as_ref().to_string(),
        }
    }

    /// Create a `ContainerError` with context
    pub fn container_error<S: AsRef<str>, M: AsRef<str>>(container: S, message: M) -> Self {
        Self::ContainerError {
            container: container.as_ref().to_string(),
            message: message.as_ref().to_string(),
        }
    }
}

impl From<std::io::Error> for AnchorError {
    fn from(err: std::io::Error) -> Self {
        Self::IoStreamError(err.to_string())
    }
}

impl From<bollard::errors::Error> for AnchorError {
    fn from(err: bollard::errors::Error) -> Self {
        match err {
            bollard::errors::Error::DockerResponseServerError { message, .. } => Self::ConnectionError(message),
            bollard::errors::Error::IOError { err: _ } => Self::ConnectionError(format!("IO Error: {err}")),
            _ => Self::ConnectionError(err.to_string()),
        }
    }
}

impl Display for AnchorError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DockerNotInstalled => write!(fmt, "Docker is not installed"),
            Self::ConnectionError(message) => write!(fmt, "Docker connection error: {message}"),
            Self::ECRCredentialsError(message) => write!(fmt, "Docker ECR credentials error: {message}"),
            Self::ImageError { image, message } => {
                write!(fmt, "Docker image error for '{image}': {message}")
            }
            Self::ContainerError { container, message } => {
                write!(fmt, "Docker container error for '{container}': {message}")
            }
            Self::IoStreamError(message) => write!(fmt, "Docker io stream error: {message}"),
        }
    }
}

impl std::error::Error for AnchorError {}
