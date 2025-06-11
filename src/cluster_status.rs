#[derive(Debug, PartialEq)]
pub enum ClusterStatus {
    Downloaded(String),
    Built(String),
    Running(String),
    Ready,
}
