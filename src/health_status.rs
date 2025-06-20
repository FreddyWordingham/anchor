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
            Self::Starting => write!(fmt, "starting"),
            Self::Healthy => write!(fmt, "healthy"),
            Self::Unhealthy => write!(fmt, "unhealthy"),
            Self::None => write!(fmt, "none"),
        }
    }
}
