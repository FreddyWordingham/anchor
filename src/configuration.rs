use serde::{Deserialize, Serialize};

/// Configuration for a cluster of containers.
#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    /// Description of the cluster
    pub description: String,
    /// Names of containers in the cluster
    pub containers: Vec<String>,
}
