use std::process::Command;

use crate::anchor_error::{AnchorError, AnchorResult};

/// Attempts to start the Docker daemon process based on the operating system.
///
/// # Errors
/// Returns `AnchorError::ConnectionError` if the start command fails.
pub fn start_docker_daemon() -> AnchorResult<()> {
    if cfg!(target_os = "macos") {
        // On macOS, try to start Docker Desktop
        start_docker_macos()
    } else if cfg!(target_os = "windows") {
        // On Windows, try to start Docker Desktop
        start_docker_windows()
    } else {
        // On Linux, try to start Docker service
        start_docker_linux()
    }
}

/// Starts Docker Desktop on macOS.
fn start_docker_macos() -> AnchorResult<()> {
    // Try different possible locations for Docker Desktop
    let docker_paths = ["/Applications/Docker.app", "/System/Applications/Docker.app"];

    for path in &docker_paths {
        let output = Command::new("open").arg("-a").arg(path).output();

        match output {
            Ok(output) if output.status.success() => {
                return Ok(());
            }
            Ok(_) | Err(_) => {} // Try next path
        }
    }

    // If Docker Desktop paths don't work, try starting docker service directly
    let output = Command::new("sudo")
        .args(["launchctl", "start", "com.docker.docker"])
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(AnchorError::ConnectionError(
            "Failed to start Docker on macOS. Please start Docker Desktop manually.".to_string(),
        )),
    }
}

/// Starts Docker Desktop on Windows.
fn start_docker_windows() -> AnchorResult<()> {
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
            Ok(_) | Err(_) => {} // Try next path
        }
    }

    // Try PowerShell approach
    let output = Command::new("powershell")
        .args(["-Command", "Start-Process 'Docker Desktop'"])
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(AnchorError::ConnectionError(
            "Failed to start Docker on Windows. Please start Docker Desktop manually.".to_string(),
        )),
    }
}

/// Starts Docker service on Linux.
fn start_docker_linux() -> AnchorResult<()> {
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
            _ => Err(AnchorError::ConnectionError(
                "Failed to start Docker on Linux. Please start Docker service manually with 'sudo systemctl start docker' or 'sudo service docker start'.".to_string()
            )),
        }
}
