use std::{
    fmt::{Display, Formatter, Result},
    time::Duration,
};

use crate::{
    format::{format_bytes, format_duration},
    health_status::HealthStatus,
};

/// Runtime metrics for a running container
#[derive(Debug, Clone, Copy)]
pub struct ContainerMetrics {
    /// Container uptime since it was started
    pub uptime: Duration,
    /// Current memory usage in bytes
    pub memory_usage: u64,
    /// Memory limit for the container in bytes (if set)
    pub memory_limit: Option<u64>,
    /// Memory usage as a percentage of the limit (if limit is set)
    pub memory_percentage: Option<f64>,
    /// Current CPU usage percentage (0.0 to 100.0+)
    pub cpu_percentage: f64,
    /// Number of processes running in the container
    pub process_count: u32,
    /// Network bytes received
    pub network_rx_bytes: u64,
    /// Network bytes transmitted
    pub network_tx_bytes: u64,
    /// Block I/O bytes read
    pub block_read_bytes: u64,
    /// Block I/O bytes written
    pub block_write_bytes: u64,
    /// Container restart count
    pub restart_count: u32,
    /// Container exit code (if it has exited and restarted)
    pub last_exit_code: Option<i64>,
    /// Health status if health check is configured
    pub health_status: Option<HealthStatus>,
}

impl ContainerMetrics {
    /// Create a new ContainerMetrics with default values
    pub fn new() -> Self {
        Self {
            uptime: Duration::from_secs(0),
            memory_usage: 0,
            memory_limit: None,
            memory_percentage: None,
            cpu_percentage: 0.0,
            process_count: 0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
            block_read_bytes: 0,
            block_write_bytes: 0,
            restart_count: 0,
            last_exit_code: None,
            health_status: Some(HealthStatus::None),
        }
    }

    /// Calculate memory percentage if limit is available
    pub fn calculate_memory_percentage(&mut self) {
        if let Some(limit) = self.memory_limit {
            if limit > 0 {
                self.memory_percentage = Some((self.memory_usage as f64 / limit as f64) * 100.0);
            }
        }
    }

    /// Get formatted memory usage string
    pub fn memory_usage_display(&self) -> String {
        match (self.memory_percentage, self.memory_limit) {
            (Some(pct), Some(limit)) => {
                format!("{} / {} ({:.1}%)", format_bytes(self.memory_usage), format_bytes(limit), pct)
            }
            _ => format_bytes(self.memory_usage),
        }
    }

    /// Get formatted network usage string
    pub fn network_usage_display(&self) -> String {
        format!(
            "↓{} ↑{}",
            format_bytes(self.network_rx_bytes),
            format_bytes(self.network_tx_bytes)
        )
    }

    /// Get formatted disk I/O string
    pub fn disk_io_display(&self) -> String {
        format!(
            "R:{} W:{}",
            format_bytes(self.block_read_bytes),
            format_bytes(self.block_write_bytes)
        )
    }
}

impl Default for ContainerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ContainerMetrics {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(
            fmt,
            "Uptime: {}\nMemory: {}\nCPU: {:.1}%\nProcesses: {}\nNetwork: {}\nDisk I/O: {}\nRestarts: {}\nLast Exit Code: {:?}\nHealth: {}",
            format_duration(self.uptime),
            self.memory_usage_display(),
            self.cpu_percentage,
            self.process_count,
            self.network_usage_display(),
            self.disk_io_display(),
            self.restart_count,
            self.last_exit_code,
            self.health_status.unwrap_or(HealthStatus::None)
        )
    }
}
