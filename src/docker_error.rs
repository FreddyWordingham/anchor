use std::fmt::Display;

#[derive(Debug)]
pub enum DockerError {
    ConnectionError(String),
    ECRCredentialsError(String),
    ImageError { image: String, message: String },
    ContainerError { container: String, message: String },
    NotInstalled,
}

impl DockerError {
    /// Create an ImageError with context
    pub fn image_error<S: AsRef<str>, M: AsRef<str>>(image: S, message: M) -> Self {
        Self::ImageError {
            image: image.as_ref().to_string(),
            message: message.as_ref().to_string(),
        }
    }

    /// Create a ContainerError with context
    pub fn container_error<S: AsRef<str>, M: AsRef<str>>(container: S, message: M) -> Self {
        Self::ContainerError {
            container: container.as_ref().to_string(),
            message: message.as_ref().to_string(),
        }
    }
}

impl From<bollard::errors::Error> for DockerError {
    fn from(err: bollard::errors::Error) -> Self {
        match err {
            bollard::errors::Error::DockerResponseServerError { message, .. } => DockerError::ConnectionError(message),
            bollard::errors::Error::IOError { err: _ } => DockerError::ConnectionError(format!("IO Error: {}", err)),
            _ => DockerError::ConnectionError(err.to_string()),
        }
    }
}

impl Display for DockerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DockerError::ConnectionError(msg) => write!(f, "Docker connection error: {}", msg),
            DockerError::ECRCredentialsError(msg) => write!(f, "Docker ECR credentials error: {}", msg),
            DockerError::ImageError { image, message } => {
                write!(f, "Docker image error for '{}': {}", image, message)
            }
            DockerError::ContainerError { container, message } => {
                write!(f, "Docker container error for '{}': {}", container, message)
            }
            DockerError::NotInstalled => write!(f, "Docker is not installed"),
        }
    }
}

impl std::error::Error for DockerError {}
