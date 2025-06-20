use std::fmt::{Display, Formatter, Result};

/// Container health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            HealthStatus::Starting => write!(fmt, "starting"),
            HealthStatus::Healthy => write!(fmt, "healthy"),
            HealthStatus::Unhealthy => write!(fmt, "unhealthy"),
            HealthStatus::None => write!(fmt, "none"),
        }
    }
}
