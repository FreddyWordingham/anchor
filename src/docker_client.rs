use bollard::{
    Docker,
    auth::DockerCredentials,
    models::{ContainerCreateBody, ContainerSummary, CreateImageInfo, ImageSummary, PortBinding},
    query_parameters::{
        CreateContainerOptionsBuilder, CreateImageOptionsBuilder, ListContainersOptionsBuilder, ListImagesOptionsBuilder,
        RemoveContainerOptionsBuilder, RemoveImageOptionsBuilder, StartContainerOptionsBuilder, StopContainerOptionsBuilder,
    },
};
use futures_util::StreamExt;
use std::{
    collections::HashMap,
    io::{Write, stdout},
};

use crate::prelude::DockerError;

pub type Result<T> = std::result::Result<T, DockerError>;

pub struct DockerClient {
    docker: Docker,
    credentials: DockerCredentials,
    platform: String,
}

impl DockerClient {
    /// Creates a new `DockerClient` instance by connecting to the Docker socket and and fetching ECR credentials.
    pub async fn new(credentials: DockerCredentials) -> Result<Self> {
        // Try to connect to Docker daemon
        let docker = Docker::connect_with_local_defaults().map_err(|err| DockerError::ConnectionError(err.to_string()))?;

        // Get platform information
        let info = docker.info().await?;
        let os = info.os_type.as_deref().unwrap_or("?");
        let arch = info.architecture.as_deref().unwrap_or("?");
        let platform = format!("{}/{}", os, arch);

        Ok(DockerClient {
            docker,
            credentials,
            platform,
        })
    }

    /// Get the platform information.
    pub fn platform(&self) -> &str {
        &self.platform
    }

    /// Check if Docker daemon is still running.
    pub async fn is_docker_running(&self) -> bool {
        self.docker.version().await.is_ok()
    }

    /// List all downloaded images.
    pub async fn list_images(&self) -> Result<Vec<ImageSummary>> {
        let options = ListImagesOptionsBuilder::default().all(true).build();
        self.docker
            .list_images(Some(options))
            .await
            .map_err(|err| DockerError::ConnectionError(err.to_string()))
    }

    /// Check if a specific image is already pulled.
    pub async fn is_image_downloaded<S: AsRef<str>>(&self, image_name: S) -> Result<bool> {
        let filters = HashMap::from([("reference", vec![image_name.as_ref()])]);
        let options = ListImagesOptionsBuilder::default().all(true).filters(&filters).build();
        let images = self.docker.list_images(Some(options)).await?;
        Ok(!images.is_empty())
    }

    /// Pull (download) a Docker image.
    pub async fn download_image<S: AsRef<str>>(&self, image_name: S) -> Result<()> {
        let options = CreateImageOptionsBuilder::default()
            .from_image(image_name.as_ref())
            .platform(&self.platform)
            .build();

        let mut stream = self.docker.create_image(Some(options), None, Some(self.credentials.clone()));

        while let Some(result) = stream.next().await {
            match result {
                Ok(CreateImageInfo {
                    status, progress, id, ..
                }) => {
                    let layer = id.unwrap_or_default();
                    let status_text = status.unwrap_or_default();
                    let progress_text = progress.unwrap_or_default();
                    let line = format!("[{}] {} {}", layer, status_text, progress_text);

                    // "\r" moves cursor back to start; "\x1B[K" clears from cursor to end of line
                    print!("\r\x1B[K{}", line);
                    stdout().flush().unwrap();
                }
                Err(err) => {
                    println!(); // ensure we drop to a new line if an error occurs
                    return Err(DockerError::ImageError(err.to_string()));
                }
            }
        }

        // After the stream finishes, move to a new line so the prompt isn’t stuck at the end of the last overwrite
        println!();

        Ok(())
    }

    /// Remove (delete) a Docker image.
    pub async fn remove_image<S: AsRef<str>>(&self, image_name: S) -> Result<()> {
        let options = RemoveImageOptionsBuilder::default().force(true).build();
        self.docker
            .remove_image(image_name.as_ref(), Some(options), Some(self.credentials.clone()))
            .await
            .map_err(|err| DockerError::ImageError(err.to_string()))?;
        Ok(())
    }

    /// List all containers (both running and stopped).
    pub async fn list_containers(&self) -> Result<Vec<ContainerSummary>> {
        let options = ListContainersOptionsBuilder::default().all(true).build();
        Ok(self.docker.list_containers(Some(options)).await?)
    }

    pub async fn list_running_containers(&self) -> Result<Vec<ContainerSummary>> {
        let filters = HashMap::from([("status", vec!["running"])]);
        let options = ListContainersOptionsBuilder::default().all(false).filters(&filters).build();
        Ok(self.docker.list_containers(Some(options)).await?)
    }

    /// Create a Docker container from an image.
    pub async fn create_container<S: AsRef<str>>(&self, image_name: S, container_port: u16, host_port: u16) -> Result<String> {
        // Check if image exists first
        if !self.is_image_downloaded(image_name.as_ref()).await? {
            return Err(DockerError::ImageError(format!("Image {} not found", image_name.as_ref())));
        }

        // Configure port bindings
        let mut exposed_ports = HashMap::new();
        exposed_ports.insert(container_port.to_string(), HashMap::new());

        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            container_port.to_string(),
            Some(vec![PortBinding {
                host_port: Some(host_port.to_string()),
                ..Default::default()
            }]),
        );

        let config = ContainerCreateBody {
            image: Some(image_name.as_ref().to_string()),
            exposed_ports: Some(exposed_ports),
            host_config: Some(bollard::models::HostConfig {
                port_bindings: Some(port_bindings),
                ..Default::default()
            }),
            ..Default::default()
        };

        fn extract_name(s: &str) -> Option<&str> {
            // look for the prefix
            let prefix = "uncertainty-engine-";
            let start = s.find(prefix)?;
            // take the substring from the prefix onward
            let rest = &s[start..];
            // stop at the colon
            let end = rest.find(':')?;
            Some(&rest[..end])
        }

        let name = extract_name(image_name.as_ref()).unwrap();
        let options = CreateContainerOptionsBuilder::default().name(name).build();

        // Create the container
        let container_info = self
            .docker
            .create_container(Some(options), config)
            .await
            .map_err(|err| DockerError::ContainerError(format!("Failed to create container: {}", err)))?;

        Ok(container_info.id)
    }

    /// Check if a container with the given name exists.
    pub async fn is_container_built<S: AsRef<str>>(&self, container_name: S) -> Result<bool> {
        let mut built = false;
        for container in self.list_containers().await? {
            if let Some(names) = &container.names {
                for name in names {
                    if name.contains(container_name.as_ref()) {
                        built = true;
                        break;
                    }
                }
            }
        }
        Ok(built)
    }

    /// Check if a container with the given name is currently running.
    pub async fn is_container_running<S: AsRef<str>>(&self, container_name: S) -> Result<bool> {
        let mut running = false;
        for container in self.list_running_containers().await? {
            if let Some(names) = &container.names {
                for name in names {
                    if name.contains(container_name.as_ref()) {
                        running = true;
                        break;
                    }
                }
            }
        }
        Ok(running)
    }

    /// Start a Docker container.
    pub async fn start_container<S: AsRef<str>>(&self, container_name: S) -> Result<()> {
        // Start the container
        let options = StartContainerOptionsBuilder::default().build();
        self.docker
            .start_container(container_name.as_ref(), Some(options))
            .await
            .map_err(|err| DockerError::ContainerError(format!("Failed to start container: {}", err)))?;

        Ok(())
    }

    /// Stop a Docker container.
    pub async fn stop_container<S: AsRef<str>>(&self, container_name: S) -> Result<()> {
        let options = StopContainerOptionsBuilder::default()
            .t(10) // 10 seconds timeout
            .build();
        self.docker
            .stop_container(container_name.as_ref(), Some(options))
            .await
            .map_err(|err| DockerError::ContainerError(format!("Failed to stop container: {}", err)))?;
        Ok(())
    }

    /// Remove (delete) a Docker container.
    pub async fn remove_container<S: AsRef<str>>(&self, container_name: S) -> Result<()> {
        let options = RemoveContainerOptionsBuilder::default().force(true).build();
        self.docker
            .remove_container(container_name.as_ref(), Some(options))
            .await
            .map_err(|err| DockerError::ContainerError(format!("Failed to remove container: {}", err)))?;
        Ok(())
    }
}
