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
    /// Accepts full image reference (e.g., "my-registry.amazonaws.com/my-repo:latest" or "nginx:latest")
    pub async fn is_image_downloaded<S: AsRef<str>>(&self, image_reference: S) -> Result<bool> {
        let target_ref = image_reference.as_ref();

        // Extract short tag for comparison
        let short_tag = target_ref.split('/').last().unwrap_or(target_ref);

        for image in self.list_images().await? {
            for tag in &image.repo_tags {
                // Check both full URI and short tag
                if tag == target_ref || tag == short_tag {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Pull (download) a Docker image.
    /// Accepts full image reference (e.g., "my-registry.amazonaws.com/my-repo:latest" or "nginx:latest")
    pub async fn pull_image<S: AsRef<str>>(&self, image_reference: S) -> Result<()> {
        let options = CreateImageOptionsBuilder::default()
            .from_image(image_reference.as_ref())
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
                    return Err(DockerError::image_error(
                        image_reference,
                        format!("Failed to pull image: {}", err),
                    ));
                }
            }
        }

        // After the stream finishes, move to a new line so the prompt isn't stuck at the end of the last overwrite
        println!();

        Ok(())
    }

    /// Remove (delete) a Docker image.
    /// Accepts image reference or image ID
    pub async fn remove_image<S: AsRef<str>>(&self, image_reference: S) -> Result<()> {
        let options = RemoveImageOptionsBuilder::default().force(true).build();
        self.docker
            .remove_image(image_reference.as_ref(), Some(options), Some(self.credentials.clone()))
            .await
            .map_err(|err| DockerError::image_error(image_reference, format!("Failed to remove image: {}", err)))?;
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
    /// image_reference: Full image reference (e.g., "my-registry.amazonaws.com/my-repo:latest")
    /// container_name: Name to assign to the container
    pub async fn build_container<S: AsRef<str>, T: AsRef<str>>(
        &self,
        image_reference: S,
        container_name: T,
        port_mappings: &[(u16, u16)],
    ) -> Result<String> {
        // Check if image exists first
        if !self.is_image_downloaded(image_reference.as_ref()).await? {
            return Err(DockerError::container_error(
                container_name,
                format!("Cannot build container: image '{}' not found", image_reference.as_ref()),
            ));
        }

        // Configure port bindings
        let mut exposed_ports = HashMap::new();
        let mut port_bindings = HashMap::new();

        for (container_port, host_port) in port_mappings {
            // Add to exposed ports (Docker requires the "/tcp" suffix)
            exposed_ports.insert(format!("{}/tcp", container_port), HashMap::new());

            // Add to port bindings
            port_bindings.insert(
                format!("{}/tcp", container_port),
                Some(vec![PortBinding {
                    host_port: Some(host_port.to_string()),
                    ..Default::default()
                }]),
            );
        }

        let config = ContainerCreateBody {
            image: Some(image_reference.as_ref().to_string()),
            exposed_ports: Some(exposed_ports),
            host_config: Some(bollard::models::HostConfig {
                port_bindings: Some(port_bindings),
                ..Default::default()
            }),
            ..Default::default()
        };

        let options = CreateContainerOptionsBuilder::default().name(container_name.as_ref()).build();

        // Create the container
        let container_info = self.docker.create_container(Some(options), config).await.map_err(|err| {
            DockerError::container_error(
                container_name,
                format!(
                    "Failed to create container from image '{}': {}",
                    image_reference.as_ref(),
                    err
                ),
            )
        })?;

        Ok(container_info.id)
    }

    /// Check if a container with the given name exists.
    pub async fn is_container_built<S: AsRef<str>>(&self, container_name: S) -> Result<bool> {
        let mut built = false;
        for container in self.list_containers().await? {
            if let Some(names) = &container.names {
                for name in names {
                    if name == &format!("/{}", container_name.as_ref()) {
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
                    if name == &format!("/{}", container_name.as_ref()) {
                        running = true;
                        break;
                    }
                }
            }
        }
        Ok(running)
    }

    /// Start a Docker container by name or ID.
    pub async fn start_container<S: AsRef<str>>(&self, container_name_or_id: S) -> Result<()> {
        let options = StartContainerOptionsBuilder::default().build();
        self.docker
            .start_container(container_name_or_id.as_ref(), Some(options))
            .await
            .map_err(|err| {
                DockerError::container_error(container_name_or_id.as_ref(), format!("Failed to start container: {}", err))
            })?;

        Ok(())
    }

    /// Stop a Docker container by name or ID.
    pub async fn stop_container<S: AsRef<str>>(&self, container_name_or_id: S) -> Result<()> {
        let options = StopContainerOptionsBuilder::default()
            .t(10) // 10 seconds timeout
            .build();
        self.docker
            .stop_container(container_name_or_id.as_ref(), Some(options))
            .await
            .map_err(|err| {
                DockerError::container_error(container_name_or_id.as_ref(), format!("Failed to stop container: {}", err))
            })?;
        Ok(())
    }

    /// Remove (delete) a Docker container by name or ID.
    pub async fn remove_container<S: AsRef<str>>(&self, container_name_or_id: S) -> Result<()> {
        let options = RemoveContainerOptionsBuilder::default().force(true).build();
        self.docker
            .remove_container(container_name_or_id.as_ref(), Some(options))
            .await
            .map_err(|err| {
                DockerError::container_error(container_name_or_id.as_ref(), format!("Failed to remove container: {}", err))
            })?;
        Ok(())
    }
}
