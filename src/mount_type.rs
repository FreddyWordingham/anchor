use std::fmt::{Display, Formatter, Result};

/// Represents different types of mounts that can be attached to a container
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MountType {
    /// Bind mount: mounts a file or directory from the host
    Bind {
        /// Host path to mount from
        source: String,
        /// Container path to mount to
        target: String,
        /// Whether the mount is read-only
        read_only: bool,
    },
    /// Volume mount: uses a Docker-managed volume
    Volume {
        /// Name of the Docker volume
        source: String,
        /// Container path to mount to
        target: String,
        /// Whether the mount is read-only
        read_only: bool,
    },
    /// Anonymous volume: creates a new anonymous volume
    AnonymousVolume {
        /// Container path to mount to
        target: String,
        /// Whether the mount is read-only
        read_only: bool,
    },
}

impl MountType {
    /// Creates a new bind mount with read-write access
    pub fn bind<S: Into<String>, T: Into<String>>(source: S, target: T) -> Self {
        Self::Bind {
            source: source.into(),
            target: target.into(),
            read_only: false,
        }
    }

    /// Creates a new read-only bind mount
    pub fn bind_ro<S: Into<String>, T: Into<String>>(source: S, target: T) -> Self {
        Self::Bind {
            source: source.into(),
            target: target.into(),
            read_only: true,
        }
    }

    /// Creates a new volume mount with read-write access
    pub fn volume<S: Into<String>, T: Into<String>>(source: S, target: T) -> Self {
        Self::Volume {
            source: source.into(),
            target: target.into(),
            read_only: false,
        }
    }

    /// Creates a new read-only volume mount
    pub fn volume_ro<S: Into<String>, T: Into<String>>(source: S, target: T) -> Self {
        Self::Volume {
            source: source.into(),
            target: target.into(),
            read_only: true,
        }
    }

    /// Creates a new anonymous volume with read-write access
    pub fn anonymous_volume<T: Into<String>>(target: T) -> Self {
        Self::AnonymousVolume {
            target: target.into(),
            read_only: false,
        }
    }

    /// Creates a new read-only anonymous volume
    pub fn anonymous_volume_ro<T: Into<String>>(target: T) -> Self {
        Self::AnonymousVolume {
            target: target.into(),
            read_only: true,
        }
    }

    /// Returns the target path in the container
    pub fn target(&self) -> &str {
        match self {
            Self::Bind { target, .. } | Self::Volume { target, .. } | Self::AnonymousVolume { target, .. } => target,
        }
    }

    /// Returns the source path (if applicable)
    pub fn source(&self) -> Option<&str> {
        match self {
            Self::Bind { source, .. } | Self::Volume { source, .. } => Some(source),
            Self::AnonymousVolume { .. } => None,
        }
    }

    /// Returns whether the mount is read-only
    pub fn is_read_only(&self) -> bool {
        match self {
            Self::Bind { read_only, .. } | Self::Volume { read_only, .. } | Self::AnonymousVolume { read_only, .. } => {
                *read_only
            }
        }
    }

    /// Returns the mount type as a string for Docker API
    pub fn mount_type_str(&self) -> &'static str {
        match self {
            Self::Bind { .. } => "bind",
            Self::Volume { .. } | Self::AnonymousVolume { .. } => "volume",
        }
    }
}

impl Display for MountType {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            Self::Bind {
                source,
                target,
                read_only,
            } => {
                let mode = if *read_only { "ro" } else { "rw" };
                write!(fmt, "{}:{}:{}", source, target, mode)
            }
            Self::Volume {
                source,
                target,
                read_only,
            } => {
                let mode = if *read_only { "ro" } else { "rw" };
                write!(fmt, "{}:{}:{}", source, target, mode)
            }
            Self::AnonymousVolume { target, read_only } => {
                let mode = if *read_only { "ro" } else { "rw" };
                write!(fmt, "{}:{}", target, mode)
            }
        }
    }
}
