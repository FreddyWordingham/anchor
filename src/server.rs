use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use crate::{
    command::Command, docker_client::DockerClient, docker_error::DockerError, manifest::Manifest, server_status::ServerStatus,
};

type Result<T> = std::result::Result<T, DockerError>;

#[derive(Debug, PartialEq)]
enum ContainerState {
    Waiting,
    Downloaded,
    Built,
    Running,
}

pub struct Server<'a> {
    client: &'a DockerClient,
    manifest: Manifest,
    containers: HashMap<String, ContainerState>,
}

impl<'a> Server<'a> {
    pub async fn new(client: &'a DockerClient, manifest: Manifest) -> Result<Self> {
        let mut containers = HashMap::new();

        for (name, container) in &manifest.containers {
            match container.command {
                Command::Ignore => continue,
                _ => {
                    containers.insert(name.clone(), ContainerState::Waiting);
                }
            }
        }

        let mut server = Self {
            client,
            manifest,
            containers,
        };
        server.sync().await?;
        Ok(server)
    }

    /// Syncronize the server state with the Docker daemon.
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
                .map_err(|err| DockerError::container_error(name, format!("Failed to sync container state: {}", err)))?
            {
                *state = ContainerState::Running;
            } else if self
                .client
                .is_container_built(name)
                .await
                .map_err(|err| DockerError::container_error(name, format!("Failed to sync container state: {}", err)))?
            {
                *state = ContainerState::Built;
            } else if self
                .client
                .is_image_downloaded(name)
                .await
                .map_err(|err| DockerError::image_error(name, format!("Failed to sync image state: {}", err)))?
            {
                *state = ContainerState::Downloaded;
            } else {
                *state = ContainerState::Waiting;
            }
        }

        Ok(())
    }

    pub async fn next(&mut self) -> Result<ServerStatus> {
        // Check if any image needs to be downloaded
        for (name, state) in &mut self.containers {
            if *state == ContainerState::Waiting {
                if !self.client.is_image_downloaded(name).await.map_err(|err| {
                    DockerError::image_error(name, format!("Failed to check image status during next(): {}", err))
                })? {
                    let uri = &self.manifest.containers[name].uri;
                    self.client
                        .pull_image(uri)
                        .await
                        .map_err(|err| DockerError::image_error(name, format!("Failed to pull image '{}': {}", uri, err)))?;
                }
                *state = ContainerState::Downloaded;
                return Ok(ServerStatus::Downloaded(name.clone()));
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
                                format!("Failed to check container build status during next(): {}", err),
                            )
                        })? {
                            let uri = &self.manifest.containers[name].uri;
                            let port_mappings = &self.manifest.containers[name].port_mappings;
                            self.client.build_container(uri, name, port_mappings).await.map_err(|err| {
                                DockerError::container_error(
                                    name,
                                    format!("Failed to build container from image '{}': {}", uri, err),
                                )
                            })?;
                        }
                        *state = ContainerState::Built;
                        return Ok(ServerStatus::Built(name.clone()));
                    }
                    _ => continue,
                }
            }
        }

        // Check if any container needs to be run
        for (name, state) in &mut self.containers {
            if *state == ContainerState::Built {
                match self.manifest.containers[name].command {
                    Command::Run => {
                        if !self.client.is_container_running(name).await.map_err(|err| {
                            DockerError::container_error(
                                name,
                                format!("Failed to check container running status during next(): {}", err),
                            )
                        })? {
                            self.client.start_container(name).await.map_err(|err| {
                                DockerError::container_error(name, format!("Failed to start container: {}", err))
                            })?;
                        }
                        *state = ContainerState::Running;
                        return Ok(ServerStatus::Running(name.clone()));
                    }
                    _ => continue,
                }
            }
        }

        Ok(ServerStatus::Ready)
    }

    /// Stop all running containers and reduce their state to `ContainerState::Built`.
    pub async fn stop(&mut self) -> Result<()> {
        // Ensure the server is in sync before stopping containers
        self.sync().await?;

        for (name, state) in &mut self.containers {
            if *state == ContainerState::Running {
                self.client.stop_container(name).await.map_err(|err| {
                    DockerError::container_error(name, format!("Failed to stop container during server shutdown: {}", err))
                })?;
                *state = ContainerState::Built;
            }
        }

        Ok(())
    }
}

impl Display for Server<'_> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "Server State:")?;
        for (name, state) in &self.containers {
            writeln!(fmt, "{}: {:?}", name, state)?;
        }
        Ok(())
    }
}
