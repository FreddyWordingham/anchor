use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use crate::{
    cluster_status::ClusterStatus,
    command::Command,
    container_state::ContainerState,
    docker_client::{DockerClient, Result},
    docker_error::DockerError,
    manifest::Manifest,
};

/// Manages a collection of Docker containers as a cohesive cluster.
///
/// Handles the complete lifecycle of downloading images, building containers,
/// and coordinating startup/shutdown across multiple containers defined in a manifest.
#[derive(Debug)]
pub struct Cluster<'a> {
    /// The Docker client used to interact with the Docker daemon.
    client: &'a DockerClient,
    /// The manifest defining the containers in this cluster.
    manifest: Manifest,
    /// The current state of each container in the cluster.
    containers: HashMap<String, ContainerState>,
}

impl<'a> Cluster<'a> {
    /// Creates a new cluster from a manifest and synchronizes with the current Docker state.
    ///
    /// # Arguments
    /// * `client` - Docker client for container operations
    /// * `manifest` - Container definitions and configuration
    ///
    /// # Errors
    /// Returns `DockerError` if Docker daemon is unreachable or container state cannot be determined.
    pub async fn new(client: &'a DockerClient, manifest: Manifest) -> Result<Self> {
        let mut containers = HashMap::new();

        for (name, container) in &manifest.containers {
            match container.command {
                Command::Ignore => {}
                _ => {
                    _ = containers.insert(name.clone(), ContainerState::Waiting);
                }
            }
        }

        let mut cluster = Self {
            client,
            manifest,
            containers,
        };
        cluster.sync().await?;
        Ok(cluster)
    }

    /// Synchronizes cluster state with the actual Docker daemon state.
    ///
    /// Updates internal container states to match what's currently running in Docker.
    /// Should be called periodically or after external Docker operations.
    ///
    /// # Errors
    /// Returns `DockerError` if Docker daemon is unreachable or state cannot be determined.
    pub async fn sync(&mut self) -> Result<()> {
        // Check if Docker is running
        if !self.client.is_docker_running().await {
            return Err(DockerError::ConnectionError("Docker daemon is not running".to_string()));
        }

        // Check if all containers are in the correct state
        for (name, state) in &mut self.containers {
            if self
                .client
                .is_container_running(name)
                .await
                .map_err(|err| DockerError::container_error(name, format!("Failed to sync container state: {err}")))?
            {
                *state = ContainerState::Running;
            } else if self
                .client
                .is_container_built(name)
                .await
                .map_err(|err| DockerError::container_error(name, format!("Failed to sync container state: {err}")))?
            {
                *state = ContainerState::Built;
            } else if self
                .client
                .is_image_downloaded(name)
                .await
                .map_err(|err| DockerError::image_error(name, format!("Failed to sync image state: {err}")))?
            {
                *state = ContainerState::Downloaded;
            } else {
                *state = ContainerState::Waiting;
            }
        }

        Ok(())
    }

    /// Starts all containers in the cluster, progressing each to its target command state.
    ///
    /// Executes containers in dependency order (images → containers → running).
    /// Calls the provided callback for each state transition to provide progress feedback.
    ///
    /// # Arguments
    /// * `callback` - Function called for each state transition with the new status
    ///
    /// # Errors
    /// Returns `DockerError` if any container operation fails.
    pub async fn start<F>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(&ClusterStatus),
    {
        loop {
            match self.next().await? {
                ClusterStatus::Ready => break,
                status => {
                    callback(&status);
                }
            }
        }

        Ok(())
    }

    /// Executes the next step in the cluster startup process.
    ///
    /// Finds the first container needing progression and advances it one state.
    /// Returns the status of the operation that was just completed.
    ///
    /// # Returns
    /// * `ClusterStatus::Downloaded/Built/Running(name)` - Next step completed for named container
    /// * `ClusterStatus::Ready` - All containers have reached their target states
    ///
    /// # Errors
    /// Returns `DockerError` if the Docker operation fails.
    async fn next(&mut self) -> Result<ClusterStatus> {
        // Check if any image needs to be downloaded
        for (name, state) in &mut self.containers {
            if *state == ContainerState::Waiting {
                if !self.client.is_image_downloaded(name).await.map_err(|err| {
                    DockerError::image_error(name, format!("Failed to check image status during next(): {err}"))
                })? {
                    let uri = &self.manifest.containers[name].uri;
                    self.client
                        .pull_image(uri)
                        .await
                        .map_err(|err| DockerError::image_error(name, format!("Failed to pull image '{uri}': {err}")))?;
                }
                *state = ContainerState::Downloaded;
                return Ok(ClusterStatus::Downloaded(name.clone()));
            }
        }

        // Check if any container needs to be built
        for (name, state) in &mut self.containers {
            if *state == ContainerState::Downloaded {
                match self.manifest.containers[name].command {
                    Command::Build | Command::Run => {
                        if !self.client.is_container_built(name).await.map_err(|err| {
                            DockerError::container_error(
                                name,
                                format!("Failed to check container build status during next(): {err}"),
                            )
                        })? {
                            let uri = &self.manifest.containers[name].uri;
                            let port_mappings = &self.manifest.containers[name].port_mappings;
                            let _id = self.client.build_container(uri, name, port_mappings).await.map_err(|err| {
                                DockerError::container_error(
                                    name,
                                    format!("Failed to build container from image '{uri}': {err}"),
                                )
                            })?;
                        }
                        *state = ContainerState::Built;
                        return Ok(ClusterStatus::Built(name.clone()));
                    }
                    _ => {}
                }
            }
        }

        // Check if any container needs to be run
        for (name, state) in &mut self.containers {
            if *state == ContainerState::Built && matches!(self.manifest.containers[name].command, Command::Run) {
                if !self.client.is_container_running(name).await.map_err(|err| {
                    DockerError::container_error(name, format!("Failed to check container running status during next(): {err}"))
                })? {
                    self.client
                        .start_container(name)
                        .await
                        .map_err(|err| DockerError::container_error(name, format!("Failed to start container: {err}")))?;
                }
                *state = ContainerState::Running;
                return Ok(ClusterStatus::Running(name.clone()));
            }
        }

        Ok(ClusterStatus::Ready)
    }

    /// Stops all running containers in the cluster.
    ///
    /// Reduces container states from `Running` to `Built`, leaving containers
    /// available for restart without rebuilding.
    ///
    /// # Errors
    /// Returns `DockerError` if any container cannot be stopped.
    pub async fn stop(&mut self) -> Result<()> {
        // Ensure the cluster containers are in sync before stopping containers
        self.sync().await?;

        for (name, state) in &mut self.containers {
            if *state == ContainerState::Running {
                self.client.stop_container(name).await.map_err(|err| {
                    DockerError::container_error(name, format!("Failed to stop container during server shutdown: {err}"))
                })?;
                *state = ContainerState::Built;
            }
        }

        Ok(())
    }
}

impl Display for Cluster<'_> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "Cluster State:")?;
        for (name, state) in &self.containers {
            writeln!(fmt, "{name}: {state:?}")?;
        }
        Ok(())
    }
}
