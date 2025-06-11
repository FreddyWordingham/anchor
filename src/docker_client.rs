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

/// Type alias for Results that may return `DockerError`.
///
/// Provides a convenient shorthand for Docker operations that can fail,
/// eliminating the need to repeatedly specify the error type.
pub type Result<T> = std::result::Result<T, DockerError>;

/// Client for interacting with the Docker daemon and container registry.
///
/// Provides high-level operations for managing Docker images and containers,
/// including authenticated pulls from registries like AWS ECR. Maintains
/// connection state and platform information for the Docker environment.
#[derive(Debug)]
pub struct DockerClient {
    /// Handle to the Docker daemon connection
    docker: Docker,
    /// Registry credentials for authenticated image operations
    credentials: DockerCredentials,
    /// Platform string (e.g., "linux/amd64") of the Docker host
    platform: String,
}

impl DockerClient {
    /// Creates a new Docker client with the provided credentials.
    ///
    /// Establishes connection to the local Docker daemon and retrieves platform information.
    ///
    /// # Arguments
    /// * `credentials` - Docker registry credentials for authenticated pulls
    ///
    /// # Errors
    /// Returns `DockerError::ConnectionError` if Docker daemon is unreachable.
    pub async fn new(credentials: DockerCredentials) -> Result<Self> {
        // Try to connect to Docker daemon
        let docker = Docker::connect_with_local_defaults().map_err(|err| DockerError::ConnectionError(err.to_string()))?;

        // Get platform information
        let info = docker.info().await?;
        let os = info.os_type.as_deref().unwrap_or("?");
        let arch = info.architecture.as_deref().unwrap_or("?");
        let platform = format!("{os}/{arch}");

        Ok(Self {
            docker,
            credentials,
            platform,
        })
    }

    /// Returns the platform string (OS/architecture) of the Docker daemon.
    ///
    /// Format: "linux/amd64", "darwin/arm64", etc.
    #[must_use]
    pub fn platform(&self) -> &str {
        &self.platform
    }

    /// Checks if the Docker daemon is still responsive.
    ///
    /// Useful for health checks and connection validation.
    pub async fn is_docker_running(&self) -> bool {
        self.docker.version().await.is_ok()
    }

    /// Lists all Docker images on the system, including intermediate images.
    ///
    /// # Errors
    /// Returns `DockerError::ConnectionError` if the Docker API call fails.
    pub async fn list_images(&self) -> Result<Vec<ImageSummary>> {
        let options = ListImagesOptionsBuilder::default().all(true).build();
        self.docker
            .list_images(Some(options))
            .await
            .map_err(|err| DockerError::ConnectionError(err.to_string()))
    }

    /// Checks if a specific Docker image is available locally.
    ///
    /// Supports both full registry URIs and short tags for matching.
    ///
    /// # Arguments
    /// * `image_reference` - Full image URI or short name (e.g., "nginx:latest")
    ///
    /// # Errors
    /// Returns `DockerError` if the image list cannot be retrieved.
    pub async fn is_image_downloaded<S: AsRef<str>>(&self, image_reference: S) -> Result<bool> {
        let target_ref = image_reference.as_ref();

        // Extract short tag for comparison
        let short_tag = target_ref.split('/').next_back().unwrap_or(target_ref);

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

    /// Downloads a Docker image from a registry.
    ///
    /// Shows real-time progress during the download process with overwriting output.
    /// Automatically uses the configured credentials for authenticated registries.
    ///
    /// # Arguments
    /// * `image_reference` - Full image URI to download
    ///
    /// # Errors
    /// Returns `DockerError::ImageError` if the download fails.
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
                    let line = format!("[{layer}] {status_text} {progress_text}");

                    // "\r" moves cursor back to start; "\x1B[K" clears from cursor to end of line
                    print!("\r\x1B[K{line}");
                    stdout().flush()?;
                }
                Err(err) => {
                    println!(); // ensure we drop to a new line if an error occurs
                    return Err(DockerError::image_error(
                        image_reference,
                        format!("Failed to pull image: {err}"),
                    ));
                }
            }
        }

        // After the stream finishes, move to a new line so the prompt isn't stuck at the end of the last overwrite
        println!();

        Ok(())
    }

    /// Removes a Docker image from the local system.
    ///
    /// Forces removal even if the image is in use by stopped containers.
    ///
    /// # Arguments
    /// * `image_reference` - Image name, tag, or ID to remove
    ///
    /// # Errors
    /// Returns `DockerError::ImageError` if removal fails.
    pub async fn remove_image<S: AsRef<str>>(&self, image_reference: S) -> Result<()> {
        let options = RemoveImageOptionsBuilder::default().force(true).build();
        let _unused = self
            .docker
            .remove_image(image_reference.as_ref(), Some(options), Some(self.credentials.clone()))
            .await
            .map_err(|err| DockerError::image_error(image_reference, format!("Failed to remove image: {err}")))?;
        Ok(())
    }

    /// Lists all containers on the system (running and stopped).
    ///
    /// # Errors
    /// Returns `DockerError` if the container list cannot be retrieved.
    pub async fn list_containers(&self) -> Result<Vec<ContainerSummary>> {
        let options = ListContainersOptionsBuilder::default().all(true).build();
        Ok(self.docker.list_containers(Some(options)).await?)
    }

    /// Lists only currently running containers.
    ///
    /// # Errors
    /// Returns `DockerError` if the container list cannot be retrieved.
    pub async fn list_running_containers(&self) -> Result<Vec<ContainerSummary>> {
        let filters = HashMap::from([("status", vec!["running"])]);
        let options = ListContainersOptionsBuilder::default().all(false).filters(&filters).build();
        Ok(self.docker.list_containers(Some(options)).await?)
    }

    /// Creates a new Docker container from an image with port mappings.
    ///
    /// The container is created but not started. Configures port bindings
    /// to map container ports to host ports.
    ///
    /// # Arguments
    /// * `image_reference` - Docker image to create container from
    /// * `container_name` - Name to assign to the new container
    /// * `port_mappings` - Array of (`container_port`, `host_port`) tuples
    ///
    /// # Returns
    /// The container ID of the created container.
    ///
    /// # Errors
    /// Returns `DockerError::ContainerError` if creation fails or image doesn't exist.
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
            #[expect(
                clippy::zero_sized_map_values,
                reason = "The seemingly odd choice of a `HashMap::new` type for the map value is a upstream requirement for a `bollard::models::PortBinding`."
            )]
            let _unused = exposed_ports.insert(format!("{container_port}/tcp"), HashMap::new());

            // Add to port bindings
            let _unused = port_bindings.insert(
                format!("{container_port}/tcp"),
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

    /// Checks if a container with the given name exists (built but may not be running).
    ///
    /// # Arguments
    /// * `container_name` - Name of the container to check
    ///
    /// # Errors
    /// Returns `DockerError` if the container list cannot be retrieved.
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

    /// Checks if a container with the given name is currently running.
    ///
    /// # Arguments
    /// * `container_name` - Name of the container to check
    ///
    /// # Errors
    /// Returns `DockerError` if the container status cannot be determined.
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

    /// Starts an existing Docker container.
    ///
    /// The container must already be created (built) before it can be started.
    ///
    /// # Arguments
    /// * `container_name_or_id` - Container name or ID to start
    ///
    /// # Errors
    /// Returns `DockerError::ContainerError` if the container cannot be started.
    pub async fn start_container<S: AsRef<str>>(&self, container_name_or_id: S) -> Result<()> {
        let options = StartContainerOptionsBuilder::default().build();
        self.docker
            .start_container(container_name_or_id.as_ref(), Some(options))
            .await
            .map_err(|err| {
                DockerError::container_error(container_name_or_id.as_ref(), format!("Failed to start container: {err}"))
            })?;

        Ok(())
    }

    /// Stops a running Docker container gracefully.
    ///
    /// Sends SIGTERM and waits up to 10 seconds before forcing termination.
    ///
    /// # Arguments
    /// * `container_name_or_id` - Container name or ID to stop
    ///
    /// # Errors
    /// Returns `DockerError::ContainerError` if the container cannot be stopped.
    pub async fn stop_container<S: AsRef<str>>(&self, container_name_or_id: S) -> Result<()> {
        let options = StopContainerOptionsBuilder::default()
            .t(10) // 10 seconds timeout
            .build();
        self.docker
            .stop_container(container_name_or_id.as_ref(), Some(options))
            .await
            .map_err(|err| {
                DockerError::container_error(container_name_or_id.as_ref(), format!("Failed to stop container: {err}"))
            })?;
        Ok(())
    }

    /// Forcefully removes a Docker container.
    ///
    /// Removes the container even if it's currently running.
    ///
    /// # Arguments
    /// * `container_name_or_id` - Container name or ID to remove
    ///
    /// # Errors
    /// Returns `DockerError::ContainerError` if removal fails.
    pub async fn remove_container<S: AsRef<str>>(&self, container_name_or_id: S) -> Result<()> {
        let options = RemoveContainerOptionsBuilder::default().force(true).build();
        self.docker
            .remove_container(container_name_or_id.as_ref(), Some(options))
            .await
            .map_err(|err| {
                DockerError::container_error(container_name_or_id.as_ref(), format!("Failed to remove container: {err}"))
            })?;
        Ok(())
    }
}
