use std::process::Command;

use crate::{docker_client::Result, prelude::DockerError};

/// Attempts to start the Docker daemon process based on the operating system.
///
/// # Errors
/// Returns `DockerError::ConnectionError` if the start command fails.
pub async fn start_docker_daemon() -> Result<()> {
    let result = if cfg!(target_os = "macos") {
        // On macOS, try to start Docker Desktop
        start_docker_macos().await
    } else if cfg!(target_os = "windows") {
        // On Windows, try to start Docker Desktop
        start_docker_windows().await
    } else {
        // On Linux, try to start Docker service
        start_docker_linux().await
    };

    result
}

/// Starts Docker Desktop on macOS.
async fn start_docker_macos() -> Result<()> {
    // Try different possible locations for Docker Desktop
    let docker_paths = ["/Applications/Docker.app", "/System/Applications/Docker.app"];

    for path in &docker_paths {
        let output = Command::new("open").arg("-a").arg(path).output();

        match output {
            Ok(output) if output.status.success() => {
                return Ok(());
            }
            Ok(_) => continue,  // Try next path
            Err(_) => continue, // Try next path
        }
    }

    // If Docker Desktop paths don't work, try starting docker service directly
    let output = Command::new("sudo")
        .args(["launchctl", "start", "com.docker.docker"])
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(DockerError::ConnectionError(
            "Failed to start Docker on macOS. Please start Docker Desktop manually.".to_string(),
        )),
    }
}

/// Starts Docker Desktop on Windows.
async fn start_docker_windows() -> Result<()> {
    // Try to start Docker Desktop
    let docker_paths = [
        r"C:\Program Files\Docker\Docker\Docker Desktop.exe",
        r"C:\Program Files (x86)\Docker\Docker\Docker Desktop.exe",
    ];

    for path in &docker_paths {
        let output = Command::new("cmd").args(["/C", "start", "", path]).output();

        match output {
            Ok(output) if output.status.success() => {
                return Ok(());
            }
            Ok(_) => continue,  // Try next path
            Err(_) => continue, // Try next path
        }
    }

    // Try PowerShell approach
    let output = Command::new("powershell")
        .args(["-Command", "Start-Process 'Docker Desktop'"])
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(DockerError::ConnectionError(
            "Failed to start Docker on Windows. Please start Docker Desktop manually.".to_string(),
        )),
    }
}

/// Starts Docker service on Linux.
async fn start_docker_linux() -> Result<()> {
    // Try systemctl first (most common on modern Linux)
    let systemctl_output = Command::new("sudo").args(["systemctl", "start", "docker"]).output();

    if let Ok(output) = systemctl_output {
        if output.status.success() {
            return Ok(());
        }
    }

    // Try service command (older systems)
    let service_output = Command::new("sudo").args(["service", "docker", "start"]).output();

    if let Ok(output) = service_output {
        if output.status.success() {
            return Ok(());
        }
    }

    // Try direct dockerd command (last resort)
    let dockerd_output = Command::new("sudo").args(["dockerd", "--detach"]).output();

    match dockerd_output {
            Ok(output) if output.status.success() => Ok(()),
            _ => Err(DockerError::ConnectionError(
                "Failed to start Docker on Linux. Please start Docker service manually with 'sudo systemctl start docker' or 'sudo service docker start'.".to_string()
            )),
        }
}
