/// Represents the current state of a container in the cluster lifecycle.
///
/// Containers progress through these states sequentially:
/// - `Waiting` → `Downloaded` → `Built` → `Running`
#[derive(Debug, PartialEq, Eq)]
pub enum ContainerState {
    /// Container is waiting to be processed (initial state)
    Waiting,
    /// Docker image has been downloaded but container not yet created
    Downloaded,
    /// Container has been created from the image but is not running
    Built,
    /// Container is actively running
    Running,
}
