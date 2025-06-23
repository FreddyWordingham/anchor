use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

/// Container health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Health check is starting
    Starting,
    /// Container is healthy
    Healthy,
    /// Container is unhealthy
    Unhealthy,
    /// No health check configured
    None,
}

impl Display for HealthStatus {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            Self::Starting => write!(fmt, "Starting"),
            Self::Healthy => write!(fmt, "Healthy"),
            Self::Unhealthy => write!(fmt, "Unhealthy"),
            Self::None => write!(fmt, "None"),
        }
    }
}
