use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

use crate::{container::Container, manifest_error::ManifestError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub containers: HashMap<String, Container>,
}

impl Manifest {
    /// Construct a new `Manifest` with the given `Container`s configurations.
    pub fn new(containers: HashMap<String, Container>) -> Result<Self, ManifestError> {
        let manifest = Manifest { containers };
        manifest.validate()?;
        Ok(manifest)
    }

    /// Construct a new empty `Manifest`.
    pub fn empty() -> Self {
        Manifest {
            containers: HashMap::new(),
        }
    }

    /// Validate that the `Manifest` is well-formed.
    /// All host ports must be unique across all containers.
    pub fn validate(&self) -> Result<(), ManifestError> {
        // Check for that all ports are unique
        let mut seen_ports = HashSet::new();
        for (id, container) in &self.containers {
            for (_port, host_port) in &container.port_mappings {
                if !seen_ports.insert(*host_port) {
                    return Err(ManifestError::ValidationError(format!(
                        "Host port {} for container '{}' is used multiple times",
                        host_port, id
                    )));
                }
            }
        }
        Ok(())
    }

    /// Access the `Container`s in the `Manifest`.
    pub fn containers(&self) -> &HashMap<String, Container> {
        &self.containers
    }

    /// Add a `Container` to the `Manifest`.
    /// Returns an error if a container with the same ID already exists.
    /// Returns an error if the container's port mappings are not unique across the manifest.
    pub fn add_container(&mut self, id: String, container: Container) -> Result<(), ManifestError> {
        if self.containers.contains_key(&id) {
            return Err(ManifestError::ValidationError(format!(
                "Container with ID '{}' already exists",
                id
            )));
        }
        self.containers.insert(id, container);
        self.validate()?;
        Ok(())
    }
}

// (De)Serialisation.
impl Manifest {
    /// Serialize to a JSON string.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize an instance from a JSON string.
    pub fn from_json(s: &str) -> Result<Self, ManifestError> {
        let manifest: Self = serde_json::from_str(s)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Save (serialize) to the given file path (overwrites if exists).
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        let json = self.to_json().map_err(io::Error::other)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())
    }

    /// Load (deserialize) an instance from the given file path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ManifestError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::from_json(&contents)
    }
}
