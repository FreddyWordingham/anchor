<p align="center" style="font-size: 2.5em">
    Anchor
</p>
<p align="center">
    <img src="./assets/images/icon.png" alt="Anchor Icon" width="200" style="border-radius: 5%; border: 2px solid #000;">
</p>
<p align="center" style="font-size: 1.5em">
    Declarative Docker Cluster Management in Rust
</p>

[![crates.io](https://img.shields.io/crates/v/anchor.svg)](https://crates.io/crates/anchor)
[![Documentation](https://docs.rs/anchor/badge.svg)](https://docs.rs/anchor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust library for managing Docker container clusters from Rust.
Anchor simplifies the process of downloading images, building containers, and orchestrating multi-container applications.

## Features

- 🐳 **Docker Container Management** - Create, start, stop, and monitor Docker containers
- 📊 **Real-time Metrics** - Collect detailed runtime metrics including CPU, memory, and network usage
- 🔒 **AWS ECR Integration** - Seamless authentication with Amazon Elastic Container Registry
- 💾 **Flexible Mount Support** - Bind mounts, named volumes, and anonymous volumes
- 🔍 **Resource Status Tracking** - Track the lifecycle status of images and containers
- 🚀 **Cross-platform** - Works on Linux, macOS, and Windows
- ⚡ **Async/Await** - Built with modern async Rust for high performance

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
anchor = "0.1.0"

# Enable AWS ECR support (optional)
anchor = { version = "0.1.0", features = ["aws_ecr"] }
```

## Quick Start

```rust
use anchor::prelude::*;
use bollard::auth::DockerCredentials;

#[tokio::main]
async fn main() -> AnchorResult<()> {
    // Create a Docker client
    let client = Client::new(DockerCredentials::default()).await?;

    // Check if Docker is running
    if !client.is_docker_running().await {
        start_docker_daemon()?;
    }

    // Pull an image
    client.pull_image("nginx:latest").await?;

    // Create and start a container
    let container_id = client.build_container(
        "nginx:latest",
        "my-nginx",
        &[(80, 8080)], // Port mapping: container:host
        &[("ENV_VAR", "value")], // Environment variables
        &[MountType::bind("/host/path", "/container/path")], // Mounts
    ).await?;

    client.start_container("my-nginx").await?;

    // Get container metrics
    let metrics = client.get_container_metrics("my-nginx").await?;
    println!("Container uptime: {}", metrics.uptime.as_secs());
    println!("Memory usage: {}", metrics.memory_usage_display());
    println!("CPU usage: {:.1}%", metrics.cpu_percentage);

    Ok(())
}
```

## Core Concepts

### Resource Status

Anchor tracks the lifecycle of Docker resources through the `ResourceStatus` enum:

- **Missing** - Image needs to be downloaded
- **Available** - Image is downloaded but container doesn't exist
- **Built** - Container exists but isn't running
- **Running** - Container is actively running

```rust
let status = client.get_resource_status("nginx:latest", "my-nginx").await?;
match status {
    ResourceStatus::Missing => println!("Need to pull image"),
    ResourceStatus::Available => println!("Ready to build container"),
    ResourceStatus::Built => println!("Ready to start container"),
    ResourceStatus::Running => println!("Container is running"),
}
```

### Mount Types

Anchor supports three types of mounts:

```rust
// Bind mount - mount host directory/file into container
let bind_mount = MountType::bind("/host/data", "/app/data");
let readonly_bind = MountType::bind_ro("/host/config", "/app/config");

// Named volume - use Docker-managed volume
let volume_mount = MountType::volume("my-volume", "/app/storage");
let readonly_volume = MountType::volume_ro("config-vol", "/app/config");

// Anonymous volume - create temporary volume
let anon_volume = MountType::anonymous_volume("/tmp/cache");
let readonly_anon = MountType::anonymous_volume_ro("/app/readonly");
```

### Container Metrics

Get detailed runtime information about your containers:

```rust
let metrics = client.get_container_metrics("my-container").await?;

println!("Uptime: {}", format_duration(metrics.uptime));
println!("Memory: {}", metrics.memory_usage_display());
println!("CPU: {:.1}%", metrics.cpu_percentage);
println!("Network: {}", metrics.network_usage_display());
println!("Disk I/O: {}", metrics.disk_io_display());
println!("Health: {}", metrics.health_status.unwrap_or_default());
```

## AWS ECR Integration

When the `aws_ecr` feature is enabled, you can authenticate with Amazon ECR:

```rust
use anchor::prelude::*;

#[tokio::main]
async fn main() -> AnchorResult<()> {
    // Get ECR credentials (requires AWS credentials in environment)
    let credentials = get_ecr_credentials().await
        .map_err(|e| AnchorError::ECRCredentialsError(e.to_string()))?;

    let client = Client::new(credentials).await?;

    // Pull from ECR
    client.pull_image("123456789012.dkr.ecr.us-west-2.amazonaws.com/my-app:latest").await?;

    Ok(())
}
```

## Advanced Usage

### Custom Error Handling

```rust
use anchor::prelude::*;

async fn handle_container(client: &Client) -> AnchorResult<()> {
    match client.start_container("my-app").await {
        Ok(()) => println!("Container started successfully"),
        Err(AnchorError::ContainerError { container, message }) => {
            eprintln!("Failed to start {}: {}", container, message);
        }
        Err(AnchorError::ConnectionError(msg)) => {
            eprintln!("Docker connection failed: {}", msg);
            start_docker_daemon()?;
        }
        Err(e) => eprintln!("Unexpected error: {}", e),
    }
    Ok(())
}
```

### Health Monitoring

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn monitor_container_health(client: &Client, name: &str) -> AnchorResult<()> {
    loop {
        let status = client.get_container_status(name).await?;

        if status.is_running() {
            let metrics = client.get_container_metrics(name).await?;

            if let Some(health) = metrics.health_status {
                match health {
                    HealthStatus::Healthy => println!("✅ Container is healthy"),
                    HealthStatus::Unhealthy => println!("❌ Container is unhealthy"),
                    HealthStatus::Starting => println!("🔄 Health check starting"),
                    HealthStatus::None => println!("ℹ️ No health check configured"),
                }
            }

            // Alert on high resource usage
            if metrics.cpu_percentage > 80.0 {
                println!("⚠️ High CPU usage: {:.1}%", metrics.cpu_percentage);
            }

            if let Some(mem_pct) = metrics.memory_percentage {
                if mem_pct > 80.0 {
                    println!("⚠️ High memory usage: {:.1}%", mem_pct);
                }
            }
        }

        sleep(Duration::from_secs(30)).await;
    }
}
```

## Platform Support

Anchor automatically detects and adapts to your platform:

- **Linux** - Uses `systemctl` or `service` commands to start Docker daemon
- **macOS** - Integrates with Docker Desktop
- **Windows** - Supports Docker Desktop on Windows

The library automatically starts the Docker daemon when needed:

```rust
if !client.is_docker_running().await {
    start_docker_daemon()?;
}
```

## Error Types

Anchor provides comprehensive error handling:

- `DockerNotInstalled` - Docker is not installed on the system
- `ConnectionError` - Cannot connect to Docker daemon
- `ECRCredentialsError` - AWS ECR authentication failed
- `ImageError` - Image-related operation failed
- `ContainerError` - Container-related operation failed
- `IoStreamError` - I/O operation failed

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Dependencies

- `bollard` - Docker API client
- `tokio` - Async runtime
- `chrono` - Date and time handling
- `aws-sdk-ecr` - AWS ECR integration (optional)
- `base64` - Base64 encoding/decoding

## Minimum Supported Rust Version (MSRV)

Rust 1.70 or higher is required.
