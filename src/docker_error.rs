use std::fmt::{Display, Formatter};

/// Errors that can occur when interacting with the Docker daemon.
#[derive(Debug)]
pub enum DockerError {
    /// Docker is not installed or not found
    NotInstalled,
    /// Error connecting to the Docker daemon or server
    ConnectionError(String),
    /// Error retrieving ECR credentials
    ECRCredentialsError(String),
    /// Specific error related to a Docker image
    ImageError {
        /// The Docker image that caused the error
        image: String,
        /// A descriptive error message
        message: String,
    },
    /// Specific error related to a Docker container
    ContainerError {
        /// The Docker container that caused the error
        container: String,
        /// A descriptive error message
        message: String,
    },
    /// Error reading from or writing to a Docker stream
    IoStreamError(String),
}

impl DockerError {
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

impl From<std::io::Error> for DockerError {
    fn from(err: std::io::Error) -> Self {
        Self::IoStreamError(err.to_string())
    }
}

impl From<bollard::errors::Error> for DockerError {
    fn from(err: bollard::errors::Error) -> Self {
        match err {
            bollard::errors::Error::DockerResponseServerError { message, .. } => Self::ConnectionError(message),
            bollard::errors::Error::IOError { err: _ } => Self::ConnectionError(format!("IO Error: {err}")),
            _ => Self::ConnectionError(err.to_string()),
        }
    }
}

impl Display for DockerError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInstalled => write!(fmt, "Docker is not installed"),
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

impl std::error::Error for DockerError {}
