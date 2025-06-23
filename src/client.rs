use bollard::{
    Docker,
    auth::DockerCredentials,
    models::{
        ContainerCreateBody, ContainerSummary, HostConfig, ImageSummary, Mount, MountBindOptions, MountTypeEnum,
        MountVolumeOptions, PortBinding,
    },
    query_parameters::{
        CreateContainerOptionsBuilder, CreateImageOptionsBuilder, InspectContainerOptions, ListContainersOptionsBuilder,
        ListImagesOptionsBuilder, RemoveContainerOptionsBuilder, RemoveImageOptionsBuilder, StartContainerOptionsBuilder,
        StopContainerOptionsBuilder,
    },
};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    anchor_error::{AnchorError, AnchorResult},
    container_metrics::ContainerMetrics,
    health_status::HealthStatus,
    mount_type::MountType,
    resource_status::ResourceStatus,
};

/// Client for interacting with the Docker daemon.
#[derive(Debug)]
pub struct Client {
    /// Handle to the Docker daemon connection
    docker: Docker,
    /// Registry credentials for authenticated image operations
    credentials: DockerCredentials,
    /// Platform string (e.g., "linux/amd64") of the Docker host
    platform: String,
}

impl Client {
    /// Creates a new Docker client with the provided credentials.
    ///
    /// Establishes connection to the local Docker daemon and retrieves platform information.
    ///
    /// # Arguments
    /// * `credentials` - Docker registry credentials for authenticated pulls
    ///
    /// # Errors
    /// Returns `AnchorError::ConnectionError` if Docker daemon is unreachable.
    pub async fn new(credentials: DockerCredentials) -> AnchorResult<Self> {
        // Try to connect to Docker daemon
        let docker = Docker::connect_with_local_defaults().map_err(|err| AnchorError::ConnectionError(err.to_string()))?;

        // Get platform information
        let info = docker.info().await?;
        let os = info.os_type.as_deref().unwrap_or("unknown");
        let arch = info.architecture.as_deref().unwrap_or("unknown");
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

    /// Gets the status of a Docker resource, which can be either an image or a container.
    ///
    /// Returns `ResourceStatus::Missing` if the image is missing,
    /// `ResourceStatus::Available` if the image is available but the container is not running,
    /// `ResourceStatus::Built` if the container exists but is not running,
    /// and `ResourceStatus::Running` if the container is currently running.
    ///
    /// # Arguments
    /// * `image_reference` - Full image URI or short name (e.g., "nginx:latest")
    /// * `container_name_or_id` - Container name or ID to check
    ///
    /// # Errors
    /// Returns `AnchorError` if the image or container list cannot be retrieved.
    pub async fn get_resource_status<S: AsRef<str>, T: AsRef<str>>(
        &self,
        image_reference: S,
        container_name_or_id: T,
    ) -> AnchorResult<ResourceStatus> {
        // Check image status first
        let image_status = self.get_image_status(image_reference).await?;
        if image_status.is_missing() {
            return Ok(image_status);
        }

        // If the image is available, check the container status
        let container_status = self.get_container_status(container_name_or_id).await?;
        if container_status.is_missing() {
            return Ok(image_status);
        }
        Ok(container_status)
    }

    /// Gets the status of a Docker image.
    ///
    /// Returns `ResourceStatus::Available` if the image is present locally,
    /// or `ResourceStatus::Missing` if it needs to be downloaded.
    ///
    /// # Arguments
    /// * `image_reference` - Full image URI or short name (e.g., "nginx:latest")
    ///
    /// # Errors
    /// Returns `AnchorError` if the image list cannot be retrieved.
    async fn get_image_status<S: AsRef<str>>(&self, image_reference: S) -> AnchorResult<ResourceStatus> {
        let is_available = self.is_image_downloaded(image_reference).await?;

        if is_available {
            Ok(ResourceStatus::Downloaded)
        } else {
            Ok(ResourceStatus::Missing)
        }
    }

    /// Gets the status of a Docker container.
    ///
    /// Returns the appropriate `ResourceStatus` based on the container's current state:
    /// - `ResourceStatus::Missing` if the container doesn't exist
    /// - `ResourceStatus::Built` if the container exists but is not running
    /// - `ResourceStatus::Running` if the container is running
    ///
    /// This is a lightweight check that doesn't collect detailed metrics.
    /// Use `get_container_metrics()` separately if you need detailed runtime information.
    ///
    /// # Arguments
    /// * `container_name_or_id` - Container name or ID to check
    ///
    /// # Errors
    /// Returns `AnchorError` if the container list cannot be retrieved.
    async fn get_container_status<S: AsRef<str>>(&self, container_name_or_id: S) -> AnchorResult<ResourceStatus> {
        let container_ref = container_name_or_id.as_ref();
        let containers = self.list_containers().await?;

        // Find the container by name or ID
        let container = containers.iter().find(|c| {
            // Check by ID (full or short)
            if let Some(id) = &c.id {
                if id == container_ref || id.starts_with(container_ref) {
                    return true;
                }
            }

            // Check by name
            if let Some(names) = &c.names {
                for name in names {
                    // Docker names start with '/', so we need to handle both formats
                    let clean_name = name.strip_prefix('/').unwrap_or(name);
                    if clean_name == container_ref || name == container_ref {
                        return true;
                    }
                }
            }

            false
        });

        container.map_or(Ok(ResourceStatus::Missing), |container| {
            let state = container
                .state
                .as_ref()
                .map_or_else(|| "unknown".to_string(), ToString::to_string);

            if state == "running" {
                // Container is running
                Ok(ResourceStatus::Running)
            } else {
                // Container exists but is not running
                Ok(ResourceStatus::Built)
            }
        })
    }

    /// Gets detailed runtime metrics for a container.
    ///
    /// This method performs heavier operations including Docker API calls for inspection
    /// and stats collection. Use sparingly for performance-sensitive applications.
    ///
    /// # Arguments
    /// * `container_name_or_id` - Container name or ID to get metrics for
    ///
    /// # Errors
    /// Returns `AnchorError::ContainerError` if the container doesn't exist, isn't running,
    /// or if metrics cannot be retrieved.
    pub async fn get_container_metrics<S: AsRef<str>>(&self, container_name_or_id: S) -> AnchorResult<ContainerMetrics> {
        let container_ref = container_name_or_id.as_ref();

        // Get container inspection details
        let inspect = self
            .docker
            .inspect_container(container_ref, None::<InspectContainerOptions>)
            .await
            .map_err(|err| AnchorError::container_error(container_ref, format!("Failed to inspect container: {err}")))?;

        // Get container stats (single shot, not streaming)
        let stats = self
            .docker
            .stats(
                container_ref,
                Some(
                    bollard::query_parameters::StatsOptionsBuilder::default()
                        .stream(false)
                        .build(),
                ),
            )
            .collect::<Vec<_>>()
            .await;

        let mut metrics = ContainerMetrics::new();

        // Calculate uptime from container start time
        if let Some(state) = inspect.state {
            if let Some(started_at) = state.started_at {
                // Parse the ISO 8601 timestamp from Docker
                match DateTime::parse_from_rfc3339(&started_at) {
                    Ok(start_time) => {
                        let start_timestamp = start_time.timestamp() as u64;

                        // Get current time
                        if let Ok(current_time) = SystemTime::now().duration_since(UNIX_EPOCH) {
                            let current_timestamp = current_time.as_secs();

                            // Calculate uptime
                            if current_timestamp >= start_timestamp {
                                metrics.uptime = Duration::from_secs(current_timestamp - start_timestamp);
                            } else {
                                // Handle edge case where start time is in the future (clock skew)
                                metrics.uptime = Duration::from_secs(0);
                            }
                        } else {
                            // Fallback if system time is unavailable
                            metrics.uptime = Duration::from_secs(0);
                        }
                    }
                    Err(_) => {
                        // If we can't parse the timestamp, try alternative parsing methods
                        // Docker sometimes uses slightly different formats
                        match started_at.parse::<DateTime<Utc>>() {
                            Ok(start_time) => {
                                let start_timestamp = start_time.timestamp() as u64;

                                if let Ok(current_time) = SystemTime::now().duration_since(UNIX_EPOCH) {
                                    let current_timestamp = current_time.as_secs();

                                    if current_timestamp >= start_timestamp {
                                        metrics.uptime = Duration::from_secs(current_timestamp - start_timestamp);
                                    } else {
                                        metrics.uptime = Duration::from_secs(0);
                                    }
                                } else {
                                    metrics.uptime = Duration::from_secs(0);
                                }
                            }
                            Err(err) => {
                                // Log the parsing error for debugging
                                eprintln!("Failed to parse container start time '{started_at}': {err}");
                                metrics.uptime = Duration::from_secs(0);
                            }
                        }
                    }
                }
            }

            // Get exit code
            metrics.last_exit_code = state.exit_code;

            // Get health status
            if let Some(health) = state.health {
                metrics.health_status =
                    Some(
                        health
                            .status
                            .as_ref()
                            .map_or(HealthStatus::None, |status| match status.to_string().as_str() {
                                "starting" => HealthStatus::Starting,
                                "healthy" => HealthStatus::Healthy,
                                "unhealthy" => HealthStatus::Unhealthy,
                                _ => HealthStatus::None,
                            }),
                    );
            }
        }

        // Extract metrics from stats if available
        if let Some(Ok(stat)) = stats.first() {
            // Memory metrics
            if let Some(memory) = &stat.memory_stats {
                metrics.memory_usage = memory.usage.unwrap_or(0);
                metrics.memory_limit = memory.limit;
                metrics.calculate_memory_percentage();
            }

            // CPU metrics
            if let Some(cpu) = &stat.cpu_stats {
                if let Some(precpu) = &stat.precpu_stats {
                    if let (Some(cpu_usage), Some(precpu_usage)) = (&cpu.cpu_usage, &precpu.cpu_usage) {
                        if let (Some(total_usage), Some(prev_total_usage)) = (cpu_usage.total_usage, precpu_usage.total_usage) {
                            let cpu_delta = total_usage.saturating_sub(prev_total_usage);
                            let system_delta = cpu
                                .system_cpu_usage
                                .unwrap_or(0)
                                .saturating_sub(precpu.system_cpu_usage.unwrap_or(0));

                            if system_delta > 0 {
                                let cpu_count = f64::from(cpu.online_cpus.unwrap_or(1));
                                metrics.cpu_percentage = (cpu_delta as f64 / system_delta as f64) * cpu_count * 100.0;
                            }
                        }
                    }
                }
            }

            // Network metrics
            if let Some(networks) = &stat.networks {
                metrics.network_rx_bytes = networks.rx_bytes.unwrap_or(0);
                metrics.network_tx_bytes = networks.tx_bytes.unwrap_or(0);
            }

            // Block I/O metrics
            if let Some(blkio) = &stat.blkio_stats {
                if let Some(io_service_bytes) = &blkio.io_service_bytes_recursive {
                    for entry in io_service_bytes {
                        match entry.op.as_deref() {
                            Some("read" | "Read") => metrics.block_read_bytes += entry.value.unwrap_or(0),
                            Some("write" | "Write") => metrics.block_write_bytes += entry.value.unwrap_or(0),
                            _ => {}
                        }
                    }
                }
            }

            // Process count (PIDs)
            if let Some(pids) = &stat.pids_stats {
                metrics.process_count = pids.current.unwrap_or(0) as u32;
            }
        }

        Ok(metrics)
    }

    /// Lists all Docker images on the system, including intermediate images.
    ///
    /// # Errors
    /// Returns `AnchorError::ConnectionError` if the Docker API call fails.
    pub async fn list_images(&self) -> AnchorResult<Vec<ImageSummary>> {
        let options = ListImagesOptionsBuilder::default().all(true).build();
        self.docker
            .list_images(Some(options))
            .await
            .map_err(|err| AnchorError::ConnectionError(err.to_string()))
    }

    /// Checks if a specific Docker image is available locally.
    ///
    /// Supports both full registry URIs and short tags for matching.
    ///
    /// # Arguments
    /// * `image_reference` - Full image URI or short name (e.g., "nginx:latest")
    ///
    /// # Errors
    /// Returns `AnchorError` if the image list cannot be retrieved.
    async fn is_image_downloaded<S: AsRef<str>>(&self, image_reference: S) -> AnchorResult<bool> {
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
    /// Automatically uses the configured credentials for authenticated registries.
    ///
    /// # Arguments
    /// * `image_reference` - Full image URI to download
    ///
    /// # Errors
    /// Returns `AnchorError::ImageError` if the download fails.
    pub async fn pull_image<S: AsRef<str>>(&self, image_reference: S) -> AnchorResult<()> {
        let options = CreateImageOptionsBuilder::default()
            .from_image(image_reference.as_ref())
            .platform(&self.platform)
            .build();

        let mut stream = self.docker.create_image(Some(options), None, Some(self.credentials.clone()));
        while let Some(result) = stream.next().await {
            match result {
                Ok(_) => {
                    // Image pull step completed successfully, continue
                }
                Err(err) => {
                    return Err(AnchorError::image_error(
                        image_reference,
                        format!("Failed to pull image: {err}"),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Creates a new Docker container from an image with port mappings, environment variables, and mounts.
    ///
    /// The container is created but not started. Configures port bindings
    /// to map container ports to host ports, sets environment variables, and
    /// sets up volume and bind mounts.
    ///
    /// # Arguments
    /// * `image_reference` - Docker image to create container from
    /// * `container_name` - Name to assign to the new container
    /// * `port_mappings` - `HashMap` mapping container ports to host ports
    /// * `env_vars` - `HashMap` of environment variable key-value pairs
    /// * `mounts` - Array of mount configurations (volumes, bind mounts, etc.)
    ///
    /// # Returns
    /// The container ID of the created container.
    ///
    /// # Errors
    /// Returns `AnchorError::ContainerError` if creation fails or image doesn't exist.
    pub async fn build_container<S: AsRef<str>, T: AsRef<str>>(
        &self,
        image_reference: S,
        container_name: T,
        port_mappings: &HashMap<u16, u16>,
        env_vars: &HashMap<String, String>,
        mounts: &[MountType],
    ) -> AnchorResult<String> {
        // Check if image exists first
        if !self.is_image_downloaded(image_reference.as_ref()).await? {
            return Err(AnchorError::container_error(
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

        // Configure environment variables
        let environment: Vec<String> = env_vars.iter().map(|(key, value)| format!("{key}={value}")).collect();

        // Configure mounts
        let mount_configs: Vec<Mount> = mounts
            .iter()
            .map(|mount| Mount {
                target: Some(mount.target().to_string()),
                source: mount.source().map(String::from),
                typ: Some(match mount {
                    MountType::Bind { .. } => MountTypeEnum::BIND,
                    MountType::Volume { .. } | MountType::AnonymousVolume { .. } => MountTypeEnum::VOLUME,
                }),
                read_only: Some(mount.is_read_only()),
                consistency: None,
                bind_options: match mount {
                    MountType::Bind { .. } => Some(MountBindOptions {
                        propagation: None,
                        non_recursive: None,
                        create_mountpoint: Some(true), // Create the mount point if it doesn't exist
                        read_only_force_recursive: None,
                        read_only_non_recursive: None,
                    }),
                    _ => None,
                },
                volume_options: match mount {
                    MountType::Volume { .. } | MountType::AnonymousVolume { .. } => Some(MountVolumeOptions {
                        no_copy: None,
                        labels: None,
                        driver_config: None,
                        subpath: None,
                    }),
                    MountType::Bind { .. } => None,
                },
                tmpfs_options: None,
                image_options: None,
            })
            .collect();

        let config = ContainerCreateBody {
            image: Some(image_reference.as_ref().to_string()),
            exposed_ports: Some(exposed_ports),
            env: if environment.is_empty() { None } else { Some(environment) },
            host_config: Some(HostConfig {
                port_bindings: Some(port_bindings),
                mounts: if mount_configs.is_empty() { None } else { Some(mount_configs) },
                ..Default::default()
            }),
            ..Default::default()
        };

        let options = CreateContainerOptionsBuilder::default().name(container_name.as_ref()).build();

        // Create the container
        let container_info = self.docker.create_container(Some(options), config).await.map_err(|err| {
            AnchorError::container_error(
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

    /// Removes a Docker image from the local system.
    ///
    /// Forces removal even if the image is in use by stopped containers.
    ///
    /// # Arguments
    /// * `image_reference` - Image name, tag, or ID to remove
    ///
    /// # Errors
    /// Returns `AnchorError::ImageError` if removal fails.
    pub async fn remove_image<S: AsRef<str>>(&self, image_reference: S) -> AnchorResult<()> {
        let options = RemoveImageOptionsBuilder::default().force(true).build();
        let _unused = self
            .docker
            .remove_image(image_reference.as_ref(), Some(options), Some(self.credentials.clone()))
            .await
            .map_err(|err| AnchorError::image_error(image_reference, format!("Failed to remove image: {err}")))?;
        Ok(())
    }

    /// Lists all containers on the system (running and stopped).
    ///
    /// # Errors
    /// Returns `AnchorError` if the container list cannot be retrieved.
    pub async fn list_containers(&self) -> AnchorResult<Vec<ContainerSummary>> {
        let options = ListContainersOptionsBuilder::default().all(true).build();
        Ok(self.docker.list_containers(Some(options)).await?)
    }

    /// Starts an existing Docker container.
    ///
    /// The container must already be created (built) before it can be started.
    ///
    /// # Arguments
    /// * `container_name_or_id` - Container name or ID to start
    ///
    /// # Errors
    /// Returns `AnchorError::ContainerError` if the container cannot be started.
    pub async fn start_container<S: AsRef<str>>(&self, container_name_or_id: S) -> AnchorResult<()> {
        let options = StartContainerOptionsBuilder::default().build();
        self.docker
            .start_container(container_name_or_id.as_ref(), Some(options))
            .await
            .map_err(|err| {
                AnchorError::container_error(container_name_or_id.as_ref(), format!("Failed to start container: {err}"))
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
    /// Returns `AnchorError::ContainerError` if the container cannot be stopped.
    pub async fn stop_container<S: AsRef<str>>(&self, container_name_or_id: S) -> AnchorResult<()> {
        let options = StopContainerOptionsBuilder::default()
            .t(10) // 10 seconds timeout
            .build();
        self.docker
            .stop_container(container_name_or_id.as_ref(), Some(options))
            .await
            .map_err(|err| {
                AnchorError::container_error(container_name_or_id.as_ref(), format!("Failed to stop container: {err}"))
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
    /// Returns `AnchorError::ContainerError` if removal fails.
    pub async fn remove_container<S: AsRef<str>>(&self, container_name_or_id: S) -> AnchorResult<()> {
        let options = RemoveContainerOptionsBuilder::default().force(true).build();
        self.docker
            .remove_container(container_name_or_id.as_ref(), Some(options))
            .await
            .map_err(|err| {
                AnchorError::container_error(container_name_or_id.as_ref(), format!("Failed to remove container: {err}"))
            })?;
        Ok(())
    }
}
