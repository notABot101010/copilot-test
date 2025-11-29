//! VM configuration types

use serde::{Deserialize, Serialize};

/// Volume mount configuration (host_path:guest_path)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Path on the host system
    pub host_path: String,
    /// Path inside the guest VM
    pub guest_path: String,
    /// Optional tag for virtio-fs
    pub tag: String,
}

impl VolumeMount {
    /// Parse a volume mount string in the format "host_path:guest_path"
    pub fn parse(s: &str) -> crate::Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(crate::Error::InvalidVolumeMount(format!(
                "Expected format 'host_path:guest_path', got '{}'",
                s
            )));
        }

        let host_path = parts[0].to_string();
        let guest_path = parts[1].to_string();

        // Generate a unique tag for this mount based on guest path
        let tag = guest_path
            .replace('/', "_")
            .trim_start_matches('_')
            .to_string();

        Ok(Self {
            host_path,
            guest_path,
            tag,
        })
    }
}

/// Configuration for a microVM instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmConfig {
    /// Name/ID of the image to use
    pub image: String,

    /// Number of vCPUs (default: 2)
    pub vcpus: u32,

    /// Amount of RAM in MB (default: 512)
    pub memory_mb: u32,

    /// Volume mounts (host:guest paths)
    pub volumes: Vec<VolumeMount>,

    /// Run in detached (background) mode
    pub detached: bool,

    /// Command to execute inside the VM
    pub command: Option<String>,

    /// Arguments for the command
    pub args: Vec<String>,

    /// Working directory inside the VM
    pub workdir: String,

    /// Environment variables
    pub env: Vec<(String, String)>,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            image: String::new(),
            vcpus: 2,
            memory_mb: 512,
            volumes: Vec::new(),
            detached: false,
            command: None,
            args: Vec::new(),
            workdir: "/".to_string(),
            env: Vec::new(),
        }
    }
}

impl VmConfig {
    /// Create a new VM configuration with the given image
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            ..Default::default()
        }
    }

    /// Set the number of vCPUs
    pub fn vcpus(mut self, vcpus: u32) -> Self {
        self.vcpus = vcpus;
        self
    }

    /// Set the amount of RAM in MB
    pub fn memory_mb(mut self, memory_mb: u32) -> Self {
        self.memory_mb = memory_mb;
        self
    }

    /// Add a volume mount
    pub fn volume(mut self, mount: VolumeMount) -> Self {
        self.volumes.push(mount);
        self
    }

    /// Set detached mode
    pub fn detached(mut self, detached: bool) -> Self {
        self.detached = detached;
        self
    }

    /// Set the command to execute
    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set command arguments
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Set working directory
    pub fn workdir(mut self, workdir: impl Into<String>) -> Self {
        self.workdir = workdir.into();
        self
    }

    /// Add an environment variable
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_mount_parse() {
        let mount = VolumeMount::parse("/host/path:/guest/path").unwrap();
        assert_eq!(mount.host_path, "/host/path");
        assert_eq!(mount.guest_path, "/guest/path");
        assert_eq!(mount.tag, "guest_path");
    }

    #[test]
    fn test_volume_mount_parse_invalid() {
        let result = VolumeMount::parse("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_vm_config_builder() {
        let config = VmConfig::new("alpine:latest")
            .vcpus(4)
            .memory_mb(1024)
            .detached(true)
            .command("/bin/sh")
            .workdir("/app")
            .env("FOO", "bar");

        assert_eq!(config.image, "alpine:latest");
        assert_eq!(config.vcpus, 4);
        assert_eq!(config.memory_mb, 1024);
        assert!(config.detached);
        assert_eq!(config.command, Some("/bin/sh".to_string()));
        assert_eq!(config.workdir, "/app");
        assert_eq!(config.env, vec![("FOO".to_string(), "bar".to_string())]);
    }

    #[test]
    fn test_vm_config_default() {
        let config = VmConfig::default();
        assert_eq!(config.vcpus, 2);
        assert_eq!(config.memory_mb, 512);
        assert!(!config.detached);
        assert_eq!(config.workdir, "/");
    }
}
