use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

use crate::{container::Container, manifest_error::ManifestError};

/// Declarative configuration defining a cluster of Docker containers.
///
/// A manifest specifies the complete configuration for a multi-container
/// application, including image sources, port mappings, and lifecycle commands.
/// Can be serialized to/from JSON for persistent storage and sharing.
///
/// # Example
/// ```json
/// {
///   "containers": {
///     "web": {
///       "uri": "nginx:latest",
///       "port_mappings": [[80, 8080]],
///       "command": "Run"
///     }
///   }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    /// Map of container names to their configuration.
    /// Container names must be unique and serve as identifiers throughout the cluster.
    pub containers: HashMap<String, Container>,
}

impl Manifest {
    /// Creates a new manifest with the given container configurations.
    ///
    /// Validates that all host port mappings are unique across containers.
    ///
    /// # Arguments
    /// * `containers` - Map of container names to their configurations
    ///
    /// # Errors
    /// Returns `ManifestError::ValidationError` if port conflicts are detected.
    pub fn new(containers: HashMap<String, Container>) -> Result<Self, ManifestError> {
        let manifest = Manifest { containers };
        manifest.validate()?;
        Ok(manifest)
    }

    /// Creates an empty manifest with no containers.
    ///
    /// Useful as a starting point for programmatically building manifests.
    pub fn empty() -> Self {
        Manifest {
            containers: HashMap::new(),
        }
    }

    /// Validates the manifest for structural correctness and port uniqueness.
    ///
    /// Ensures all host ports are unique across all containers to prevent conflicts.
    ///
    /// # Errors
    /// Returns `ManifestError::ValidationError` if validation fails.
    pub fn validate(&self) -> Result<(), ManifestError> {
        // Check for that all ports are unique
        let mut seen_ports = HashSet::new();
        for (name, container) in &self.containers {
            for (_port, host_port) in &container.port_mappings {
                if !seen_ports.insert(*host_port) {
                    return Err(ManifestError::ValidationError(format!(
                        "Host port {} for container '{}' is used multiple times",
                        host_port, name
                    )));
                }
            }
        }
        Ok(())
    }

    /// Returns a reference to the containers map.
    ///
    /// Provides read-only access to all container configurations in the manifest.
    pub fn containers(&self) -> &HashMap<String, Container> {
        &self.containers
    }

    /// Adds a new container to the manifest.
    ///
    /// Validates that the container name is unique and port mappings don't conflict
    /// with existing containers.
    ///
    /// # Arguments
    /// * `name` - Unique identifier for the container
    /// * `container` - Container configuration
    ///
    /// # Errors
    /// * `ManifestError::ValidationError` - If name exists or ports conflict
    pub fn add_container(&mut self, name: String, container: Container) -> Result<(), ManifestError> {
        if self.containers.contains_key(&name) {
            return Err(ManifestError::ValidationError(format!(
                "Container with name '{}' already exists",
                name
            )));
        }
        self.containers.insert(name, container);
        self.validate()?;
        Ok(())
    }

    /// Serializes the manifest to a pretty-printed JSON string.
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Deserializes a manifest from a JSON string.
    ///
    /// Validates the resulting manifest after deserialization.
    ///
    /// # Arguments
    /// * `s` - JSON string containing manifest data
    ///
    /// # Errors
    /// * `ManifestError::SerializationError` - If JSON parsing fails
    /// * `ManifestError::ValidationError` - If the parsed manifest is invalid
    pub fn from_json(s: &str) -> Result<Self, ManifestError> {
        let manifest: Self = serde_json::from_str(s)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Saves the manifest to a file as JSON.
    ///
    /// Overwrites the file if it already exists.
    ///
    /// # Arguments
    /// * `path` - File path where the manifest should be saved
    ///
    /// # Errors
    /// Returns `io::Error` if file operations fail.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        let json = self.to_json().map_err(io::Error::other)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())
    }

    /// Loads a manifest from a JSON file.
    ///
    /// # Arguments
    /// * `path` - File path to read the manifest from
    ///
    /// # Errors
    /// * `ManifestError::IoError` - If file cannot be read
    /// * `ManifestError::SerializationError` - If JSON parsing fails
    /// * `ManifestError::ValidationError` - If the loaded manifest is invalid
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ManifestError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::from_json(&contents)
    }
}
