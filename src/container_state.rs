#[derive(Debug, PartialEq)]
pub enum ContainerState {
    Waiting,
    Downloaded,
    Built,
    Running,
}
