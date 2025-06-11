use serde::{Deserialize, Serialize};

use crate::command::Command;

/// Configuration for a single container within a cluster.
///
/// Defines the Docker image, network configuration, and desired lifecycle command
/// for one container in the cluster.
#[derive(Debug, Serialize, Deserialize)]
pub struct Container {
    /// Docker image URI (e.g., "nginx:latest" or "my-registry.com/app:v1.0")
    pub uri: String,
    /// Port mappings as (container_port, host_port) pairs
    pub port_mappings: Vec<(u16, u16)>,
    /// Target command determining how far to progress this container
    pub command: Command,
}
