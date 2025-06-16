use serde::{Deserialize, Serialize};

/// Configuration for a cluster of containers.
#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    /// Names of containers in the cluster
    pub containers: Vec<String>,
}
