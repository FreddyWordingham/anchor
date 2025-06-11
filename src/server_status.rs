#[derive(Debug, PartialEq)]
pub enum ServerStatus {
    Downloaded(String),
    Built(String),
    Running(String),
    Ready,
}
