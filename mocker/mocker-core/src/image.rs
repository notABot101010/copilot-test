//! OCI Image management

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

/// Image configuration metadata (cmd, entrypoint, workdir, env)
type ImageConfig = (Vec<String>, Vec<String>, String, Vec<String>);

/// Represents an OCI image that has been pulled and converted for use with microVMs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciImage {
    /// Full image name (e.g., "docker.io/library/alpine:latest")
    pub name: String,

    /// Local path to the extracted rootfs
    pub rootfs_path: PathBuf,

    /// Image digest/ID
    pub digest: String,

    /// Default command to run
    pub cmd: Vec<String>,

    /// Default entrypoint
    pub entrypoint: Vec<String>,

    /// Default working directory
    pub workdir: String,

    /// Default environment variables
    pub env: Vec<String>,
}

/// Manager for OCI images
pub struct ImageManager {
    /// Base directory for storing images
    images_dir: PathBuf,
}

impl ImageManager {
    /// Create a new ImageManager with the given base directory
    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let images_dir = base_dir.as_ref().join("images");
        fs::create_dir_all(&images_dir)?;
        Ok(Self { images_dir })
    }

    /// Get the path to an image's rootfs
    pub fn get_image_path(&self, image_name: &str) -> PathBuf {
        let safe_name = image_name.replace(['/', ':'], "_");
        self.images_dir.join(&safe_name)
    }

    /// Check if an image exists locally
    pub fn image_exists(&self, image_name: &str) -> bool {
        self.get_image_path(image_name).exists()
    }

    /// Get image metadata
    pub fn get_image(&self, image_name: &str) -> Result<OciImage> {
        let image_path = self.get_image_path(image_name);
        if !image_path.exists() {
            return Err(Error::ImageNotFound(image_name.to_string()));
        }

        let metadata_path = image_path.join("metadata.json");
        if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            // Return a default OciImage if no metadata exists
            Ok(OciImage {
                name: image_name.to_string(),
                rootfs_path: image_path.join("rootfs"),
                digest: String::new(),
                cmd: vec!["/bin/sh".to_string()],
                entrypoint: Vec::new(),
                workdir: "/".to_string(),
                env: Vec::new(),
            })
        }
    }

    /// Pull an OCI image and convert it to a rootfs for microVM use
    ///
    /// This function uses common container tools to pull and extract images.
    /// It tries several methods in order of preference:
    /// 1. skopeo + umoci (if available)
    /// 2. podman (if available)
    /// 3. docker (if available)
    pub fn pull_image(&self, image_name: &str) -> Result<OciImage> {
        let image_path = self.get_image_path(image_name);

        if image_path.exists() {
            // Image already exists
            return self.get_image(image_name);
        }

        fs::create_dir_all(&image_path)?;
        let rootfs_path = image_path.join("rootfs");
        fs::create_dir_all(&rootfs_path)?;

        // Try different methods to pull the image
        let result = self
            .try_skopeo_pull(image_name, &image_path, &rootfs_path)
            .or_else(|_| self.try_podman_pull(image_name, &rootfs_path))
            .or_else(|_| self.try_docker_pull(image_name, &rootfs_path));

        match result {
            Ok(image) => {
                // Save metadata
                let metadata_path = image_path.join("metadata.json");
                fs::write(&metadata_path, serde_json::to_string_pretty(&image)?)?;
                Ok(image)
            }
            Err(e) => {
                // Clean up on failure
                let _ = fs::remove_dir_all(&image_path);
                Err(e)
            }
        }
    }

    /// Try to pull using skopeo and umoci
    fn try_skopeo_pull(
        &self,
        image_name: &str,
        image_path: &Path,
        rootfs_path: &Path,
    ) -> Result<OciImage> {
        // Check if skopeo is available
        if !Self::command_exists("skopeo") || !Self::command_exists("umoci") {
            return Err(Error::OciRegistry(
                "skopeo or umoci not available".to_string(),
            ));
        }

        let oci_dir = image_path.join("oci");
        fs::create_dir_all(&oci_dir)?;

        // Determine full image reference
        let full_image = if image_name.contains('/') {
            format!("docker://{}", image_name)
        } else {
            format!("docker://docker.io/library/{}", image_name)
        };

        // Pull with skopeo
        let output = Command::new("skopeo")
            .args(["copy", "--insecure-policy", &full_image])
            .arg(format!("oci:{}:latest", oci_dir.display()))
            .output()?;

        if !output.status.success() {
            return Err(Error::OciRegistry(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Unpack with umoci
        let bundle_path = image_path.join("bundle");
        let output = Command::new("umoci")
            .args(["unpack", "--image"])
            .arg(format!("{}:latest", oci_dir.display()))
            .arg(&bundle_path)
            .output()?;

        if !output.status.success() {
            return Err(Error::OciRegistry(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Move rootfs to final location
        let bundle_rootfs = bundle_path.join("rootfs");
        if bundle_rootfs.exists() {
            // Remove existing rootfs and rename
            let _ = fs::remove_dir_all(rootfs_path);
            fs::rename(&bundle_rootfs, rootfs_path)?;
        }

        // Parse config.json for metadata
        let config_path = bundle_path.join("config.json");
        let (cmd, entrypoint, workdir, env) = if config_path.exists() {
            Self::parse_oci_config(&config_path)?
        } else {
            (
                vec!["/bin/sh".to_string()],
                Vec::new(),
                "/".to_string(),
                Vec::new(),
            )
        };

        Ok(OciImage {
            name: image_name.to_string(),
            rootfs_path: rootfs_path.to_path_buf(),
            digest: String::new(),
            cmd,
            entrypoint,
            workdir,
            env,
        })
    }

    /// Try to pull using podman
    fn try_podman_pull(&self, image_name: &str, rootfs_path: &Path) -> Result<OciImage> {
        if !Self::command_exists("podman") {
            return Err(Error::OciRegistry("podman not available".to_string()));
        }

        // Generate unique container name to avoid conflicts
        let container_name = format!("mocker_temp_{}", Uuid::new_v4().simple());

        // Pull the image
        let output = Command::new("podman").args(["pull", image_name]).output()?;

        if !output.status.success() {
            return Err(Error::OciRegistry(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Create a container
        let output = Command::new("podman")
            .args(["create", "--name", &container_name, image_name])
            .output()?;

        if !output.status.success() {
            return Err(Error::OciRegistry(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Export the container filesystem
        let tar_path = rootfs_path.parent().unwrap().join("rootfs.tar");
        let output = Command::new("podman")
            .args(["export", &container_name, "-o"])
            .arg(&tar_path)
            .output()?;

        // Clean up container
        let _ = Command::new("podman").args(["rm", &container_name]).output();

        if !output.status.success() {
            return Err(Error::OciRegistry(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Extract tar
        let output = Command::new("tar")
            .args(["-xf"])
            .arg(&tar_path)
            .arg("-C")
            .arg(rootfs_path)
            .output()?;

        let _ = fs::remove_file(&tar_path);

        if !output.status.success() {
            return Err(Error::OciRegistry(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Get image metadata
        let (cmd, entrypoint, workdir, env) = Self::get_podman_image_config(image_name)?;

        Ok(OciImage {
            name: image_name.to_string(),
            rootfs_path: rootfs_path.to_path_buf(),
            digest: String::new(),
            cmd,
            entrypoint,
            workdir,
            env,
        })
    }

    /// Try to pull using docker
    fn try_docker_pull(&self, image_name: &str, rootfs_path: &Path) -> Result<OciImage> {
        if !Self::command_exists("docker") {
            return Err(Error::OciRegistry("docker not available".to_string()));
        }

        // Generate unique container name to avoid conflicts
        let container_name = format!("mocker_temp_{}", Uuid::new_v4().simple());

        // Pull the image
        let output = Command::new("docker").args(["pull", image_name]).output()?;

        if !output.status.success() {
            return Err(Error::OciRegistry(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Create a container
        let output = Command::new("docker")
            .args(["create", "--name", &container_name, image_name])
            .output()?;

        if !output.status.success() {
            return Err(Error::OciRegistry(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Export the container filesystem
        let tar_path = rootfs_path.parent().unwrap().join("rootfs.tar");
        let output = Command::new("docker")
            .args(["export", &container_name, "-o"])
            .arg(&tar_path)
            .output()?;

        // Clean up container
        let _ = Command::new("docker").args(["rm", &container_name]).output();

        if !output.status.success() {
            return Err(Error::OciRegistry(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Extract tar
        let output = Command::new("tar")
            .args(["-xf"])
            .arg(&tar_path)
            .arg("-C")
            .arg(rootfs_path)
            .output()?;

        let _ = fs::remove_file(&tar_path);

        if !output.status.success() {
            return Err(Error::OciRegistry(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Get image metadata
        let (cmd, entrypoint, workdir, env) = Self::get_docker_image_config(image_name)?;

        Ok(OciImage {
            name: image_name.to_string(),
            rootfs_path: rootfs_path.to_path_buf(),
            digest: String::new(),
            cmd,
            entrypoint,
            workdir,
            env,
        })
    }

    /// Check if a command exists
    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Parse OCI config.json for image metadata
    fn parse_oci_config(config_path: &Path) -> Result<ImageConfig> {
        let content = fs::read_to_string(config_path)?;
        let config: serde_json::Value = serde_json::from_str(&content)?;

        let process = config.get("process").unwrap_or(&serde_json::Value::Null);

        let args = process
            .get("args")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_else(|| vec!["/bin/sh".to_string()]);

        let workdir = process
            .get("cwd")
            .and_then(|w| w.as_str())
            .unwrap_or("/")
            .to_string();

        let env = process
            .get("env")
            .and_then(|e| e.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok((args, Vec::new(), workdir, env))
    }

    /// Get image config from podman
    fn get_podman_image_config(image_name: &str) -> Result<ImageConfig> {
        let output = Command::new("podman")
            .args(["inspect", image_name])
            .output()?;

        if !output.status.success() {
            return Ok((
                vec!["/bin/sh".to_string()],
                Vec::new(),
                "/".to_string(),
                Vec::new(),
            ));
        }

        Self::parse_inspect_output(&output.stdout)
    }

    /// Get image config from docker
    fn get_docker_image_config(image_name: &str) -> Result<ImageConfig> {
        let output = Command::new("docker")
            .args(["inspect", image_name])
            .output()?;

        if !output.status.success() {
            return Ok((
                vec!["/bin/sh".to_string()],
                Vec::new(),
                "/".to_string(),
                Vec::new(),
            ));
        }

        Self::parse_inspect_output(&output.stdout)
    }

    /// Parse docker/podman inspect output
    fn parse_inspect_output(output: &[u8]) -> Result<ImageConfig> {
        let content = String::from_utf8_lossy(output);
        let inspect: serde_json::Value = serde_json::from_str(&content)?;

        let config = inspect
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.get("Config"))
            .unwrap_or(&serde_json::Value::Null);

        let cmd = config
            .get("Cmd")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_else(|| vec!["/bin/sh".to_string()]);

        let entrypoint = config
            .get("Entrypoint")
            .and_then(|e| e.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let workdir = config
            .get("WorkingDir")
            .and_then(|w| w.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("/")
            .to_string();

        let env = config
            .get("Env")
            .and_then(|e| e.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok((cmd, entrypoint, workdir, env))
    }

    /// List all available images
    pub fn list_images(&self) -> Result<Vec<OciImage>> {
        let mut images = Vec::new();

        if !self.images_dir.exists() {
            return Ok(images);
        }

        for entry in fs::read_dir(&self.images_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let metadata_path = entry.path().join("metadata.json");
                if metadata_path.exists() {
                    let content = fs::read_to_string(&metadata_path)?;
                    if let Ok(image) = serde_json::from_str(&content) {
                        images.push(image);
                    }
                }
            }
        }

        Ok(images)
    }

    /// Remove an image
    pub fn remove_image(&self, image_name: &str) -> Result<()> {
        let image_path = self.get_image_path(image_name);
        if !image_path.exists() {
            return Err(Error::ImageNotFound(image_name.to_string()));
        }
        fs::remove_dir_all(image_path)?;
        Ok(())
    }
}
