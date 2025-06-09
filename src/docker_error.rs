use std::fmt::Display;

#[derive(Debug)]
pub enum DockerError {
    ConnectionError(String),
    ECRCredentialsError(String),
    ImageError(String),
    ContainerError(String),
    NotInstalled,
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
            DockerError::ImageError(msg) => write!(f, "Docker image error: {}", msg),
            DockerError::ContainerError(msg) => write!(f, "Docker container error: {}", msg),
            DockerError::NotInstalled => write!(f, "Docker is not installed"),
        }
    }
}

impl std::error::Error for DockerError {}
