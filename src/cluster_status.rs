/// Represents the status of cluster operations, indicating which step was just completed.
///
/// Used as feedback during cluster startup to track progress across all containers.
#[derive(Debug, PartialEq)]
pub enum ClusterStatus {
    /// Image download completed for the specified container
    Downloaded(String),
    /// Container build completed for the specified container
    Built(String),
    /// Container startup completed for the specified container
    Running(String),
    /// All containers in the cluster are in their target state
    Ready,
}
