use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

/// Defines the target action to perform on a container during cluster operations.
///
/// Commands determine how far through the container lifecycle each container should progress.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Command {
    /// Skip this container entirely during cluster operations
    Ignore,
    /// Download the container image only (stop at `Downloaded` state)
    Download,
    /// Download image and create container (stop at `Built` state)
    Build,
    /// Download, build, and start the container (reach `Running` state)
    Run,
}

impl Display for Command {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            Self::Ignore => write!(fmt, "Ignore"),
            Self::Download => write!(fmt, "Download"),
            Self::Build => write!(fmt, "Build"),
            Self::Run => write!(fmt, "Run"),
        }
    }
}
