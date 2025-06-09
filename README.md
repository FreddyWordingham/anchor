# Anchor

An improved Rust library for Docker operations with comprehensive async support and observability.

## 🚀 Key Features

### Enhanced Async Support

- **Configurable timeouts** for all Docker operations
- **Retry logic** with exponential backoff for connection issues
- **Concurrent task management** for running multiple operations simultaneously
- **Graceful cancellation** of long-running operations

### Comprehensive Observability

- **Real-time progress reporting** via broadcast channels
- **Structured progress events** covering all operation types
- **Task lifecycle tracking** with detailed metadata
- **Statistics and metrics** for monitoring performance

### Production-Ready Features

- **Timeout protection** for all Docker API calls
- **Memory-efficient progress parsing**
- **Automatic cleanup** of completed tasks
- **Comprehensive error handling** with context

## 📦 Usage Examples

### Basic Usage with Progress Reporting

```rust
use std::time::Duration;
use bollard::auth::DockerCredentials;
use anchor::prelude::*;

#[tokio::main]
async fn main() -> Result<(), DockerError> {
    // Create progress reporter
    let (progress_reporter, mut progress_receiver) = ProgressReporter::new();

    // Monitor progress in background
    tokio::spawn(async move {
        while let Ok(event) = progress_receiver.recv().await {
            match event {
                ProgressEvent::ImageDownload { image_name, status, progress, .. } => {
                    if let Some(prog) = progress {
                        println!("📦 {}: {} - {:.1}%", image_name, status, prog.percentage);
                    } else {
                        println!("📦 {}: {}", image_name, status);
                    }
                }
                ProgressEvent::Operation { message, level, .. } => {
                    println!("{:?}: {}", level, message);
                }
                _ => {}
            }
        }
    });

    // Create Docker client with custom configuration
    let config = DockerConfig {
        operation_timeout: Duration::from_secs(300),
        connection_timeout: Duration::from_secs(10),
        retry_attempts: 3,
        retry_delay: Duration::from_secs(2),
    };

    let client = DockerClient::with_config(DockerCredentials::default(), config)
        .await?
        .with_progress_reporter(progress_reporter);

    // Download image with progress tracking
    client.download_image("nginx:alpine").await?;

    Ok(())
}
```

### Concurrent Task Management

```rust
use anchor::prelude::*;

async fn concurrent_operations() -> Result<(), DockerError> {
    let (progress_reporter, _) = ProgressReporter::new();

    let client = DockerClient::new(DockerCredentials::default())
        .await?
        .with_progress_reporter(progress_reporter.clone());

    let task_manager = TaskManager::new(client)
        .with_progress_reporter(progress_reporter)
        .with_max_concurrent_tasks(5);

    // Submit multiple download tasks
    let images = vec!["nginx:alpine", "redis:alpine", "postgres:alpine"];
    let mut handles = Vec::new();

    for image in &images {
        let task = TaskType::ImageDownload {
            image_name: image.to_string(),
        };
        let handle = task_manager.submit_task(task).await?;
        handles.push(handle);
    }

    // Wait for all downloads
    for handle in handles {
        handle.wait().await?;
    }

    // Show statistics
    let stats = task_manager.get_task_statistics().await;
    println!("Completed: {}/{}", stats.completed, stats.total);

    Ok(())
}
```

### Manifest-Based Operations

```rust
use std::collections::HashMap;
use anchor::prelude::*;

async fn manifest_operations() -> Result<(), DockerError> {
    // Create manifest
    let mut containers = HashMap::new();
    containers.insert("web".to_string(), Container {
        uri: "nginx:latest".to_string(),
        port_mappings: vec![(80, 8080)],
        command: Command::Run,
    });

    let manifest = Manifest::new(containers)?;
    manifest.save("docker-manifest.json")?;

    // Process manifest with task manager
    let task_manager = TaskManager::new(
        DockerClient::new(DockerCredentials::default()).await?
    );

    for (name, container) in manifest.containers() {
        if matches!(container.command, Command::Run | Command::Download) {
            let task = TaskType::ImageDownload {
                image_name: container.uri.clone(),
            };
            task_manager.submit_task(task).await?;
        }
    }

    Ok(())
}
```

## 🔧 Configuration

### DockerConfig Options

```rust
let config = DockerConfig {
    operation_timeout: Duration::from_secs(600),  // 10 minutes
    connection_timeout: Duration::from_secs(30),  // 30 seconds
    retry_attempts: 3,                            // 3 retries
    retry_delay: Duration::from_secs(1),          // 1 second between retries
};
```

### Task Manager Options

```rust
let task_manager = TaskManager::new(client)
    .with_progress_reporter(reporter)
    .with_max_concurrent_tasks(10);  // Max 10 concurrent operations
```

## 📊 Progress Events

The library emits structured progress events:

- **ImageDownload**: Real-time download progress with layer information
- **ContainerCreate**: Container creation status updates
- **ContainerLifecycle**: Start/stop/remove operations
- **Operation**: General operation messages with log levels

## 🛠 Task Management

### Task Types

- `ImageDownload`: Download Docker images
- `ContainerCreate`: Create containers from images
- `ContainerStart/Stop/Remove`: Container lifecycle operations
- `ImageRemove`: Remove Docker images

### Task Status Tracking

- **Pending**: Task queued but not started
- **Running**: Task currently executing
- **Completed**: Task finished successfully
- **Failed**: Task encountered an error
- **Cancelled**: Task was cancelled

### Statistics

```rust
let stats = task_manager.get_task_statistics().await;
println!("Total: {}, Completed: {}, Failed: {}",
         stats.total, stats.completed, stats.failed);
if let Some(avg) = stats.average_duration {
    println!("Average duration: {:?}", avg);
}
```

## 🎯 Benefits

1. **Better UX**: Real-time progress feedback for long operations
2. **Scalability**: Handle multiple concurrent Docker operations efficiently
3. **Reliability**: Timeout protection and retry logic prevent hanging
4. **Observability**: Detailed tracking of operations and performance
5. **Flexibility**: Configurable behavior for different use cases
6. **Production-Ready**: Comprehensive error handling and resource management

## 🔍 Error Handling

The library provides structured error types:

- `ConnectionError`: Docker daemon connection issues
- `ImageError`: Image-related operations (download, remove)
- `ContainerError`: Container lifecycle operations
- `ECRCredentialsError`: Authentication issues
- `NotInstalled`: Docker not available

All operations include timeout protection and retry logic for transient failures.
