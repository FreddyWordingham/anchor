use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

/// Represents the status a container can be in during its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStatus {
    /// Image not available, needs to be downloaded
    Missing,
    /// Image locally available
    Downloaded,
    /// Container build completed for the specified container
    Built,
    /// Container startup completed for the specified container
    Running,
}

impl ResourceStatus {
    /// Returns true if the resource is in Missing state
    #[must_use]
    pub const fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    /// Returns true if the resource is at least available (Available, Built, or Running)
    #[must_use]
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Downloaded | Self::Built | Self::Running)
    }

    /// Returns true if the resource is at least built (Built or Running)
    #[must_use]
    pub const fn is_built(&self) -> bool {
        matches!(self, Self::Built | Self::Running)
    }

    /// Returns true if the resource is in Running state
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl Display for ResourceStatus {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            Self::Missing => write!(fmt, "Missing"),
            Self::Downloaded => write!(fmt, "Downloaded"),
            Self::Built => write!(fmt, "Built"),
            Self::Running => write!(fmt, "Running"),
        }
    }
}
