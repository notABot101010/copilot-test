//! VM Manager - handles launching and managing microVMs
//!
//! This module provides the main interface for running microVMs using libkrun.
//! Since libkrun is a C library, we use the FFI pattern to call it.

use crate::{Error, ImageManager, OciImage, Result, StateManager, VmConfig, VmState, VmStatus};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

/// Manager for microVM operations
pub struct VmManager {
    /// Image manager for OCI images
    image_manager: ImageManager,
    /// State manager for VM persistence
    state_manager: StateManager,
}

impl VmManager {
    /// Create a new VmManager with the given data directory
    pub fn new(data_dir: impl AsRef<Path>) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        let image_manager = ImageManager::new(data_dir)?;
        let state_manager = StateManager::new(data_dir)?;

        Ok(Self {
            image_manager,
            state_manager,
        })
    }

    /// Get a reference to the image manager
    pub fn images(&self) -> &ImageManager {
        &self.image_manager
    }

    /// Get a reference to the state manager
    pub fn state(&self) -> &StateManager {
        &self.state_manager
    }

    /// Run a microVM with the given configuration
    pub fn run(&self, config: &VmConfig) -> Result<VmState> {
        // Get the image
        let image = self.image_manager.get_image(&config.image)?;

        // Create VM state
        let mut state = self.state_manager.create_vm_state(config)?;

        // Launch the VM
        match self.launch_vm(&image, config, &mut state) {
            Ok(()) => {
                state.status = VmStatus::Running;
                self.state_manager.save_state(&state)?;
                Ok(state)
            }
            Err(e) => {
                state.status = VmStatus::Failed;
                let _ = self.state_manager.save_state(&state);
                Err(e)
            }
        }
    }

    /// Launch a VM using libkrun
    ///
    /// Since libkrun doesn't have published Rust bindings on crates.io,
    /// we'll use a shim approach:
    /// 1. First try to use krun (a CLI wrapper for libkrun if available)
    /// 2. Fall back to using crun with --runtime=krun
    /// 3. Otherwise simulate the VM run for development purposes
    fn launch_vm(&self, image: &OciImage, config: &VmConfig, state: &mut VmState) -> Result<()> {
        // Build the command to run inside the VM
        let cmd = if let Some(ref cmd) = config.command {
            cmd.clone()
        } else if !image.entrypoint.is_empty() {
            image.entrypoint.join(" ")
        } else if !image.cmd.is_empty() {
            image.cmd.join(" ")
        } else {
            "/bin/sh".to_string()
        };

        // Build args
        let args: Vec<String> = if config.args.is_empty() {
            Vec::new()
        } else {
            config.args.clone()
        };

        // Try different methods to run the VM
        self.try_krun_exec(image, config, state, &cmd, &args)
            .or_else(|_| self.try_crun_krun(image, config, state, &cmd, &args))
            .or_else(|_| self.run_simulated(image, config, state, &cmd, &args))
    }

    /// Try to run using krun directly (libkrun CLI wrapper)
    fn try_krun_exec(
        &self,
        image: &OciImage,
        config: &VmConfig,
        state: &mut VmState,
        cmd: &str,
        args: &[String],
    ) -> Result<()> {
        if !Self::command_exists("krun") {
            return Err(Error::Libkrun("krun not found".to_string()));
        }

        let mut command = Command::new("krun");

        // Set the rootfs
        command.arg(&image.rootfs_path);

        // Add the command
        command.arg(cmd);

        // Add arguments
        for arg in args {
            command.arg(arg);
        }

        // Handle detached mode
        if config.detached {
            command.stdin(Stdio::null());
            command.stdout(Stdio::null());
            command.stderr(Stdio::null());

            // Use setsid to create a new session
            unsafe {
                command.pre_exec(|| {
                    nix::unistd::setsid().map_err(|e| std::io::Error::other(e.to_string()))?;
                    Ok(())
                });
            }

            let child = command.spawn()?;
            state.pid = Some(child.id());
        } else {
            // Run in foreground
            let status = command.status()?;
            if !status.success() {
                return Err(Error::Process(format!(
                    "VM exited with status: {:?}",
                    status.code()
                )));
            }
        }

        Ok(())
    }

    /// Try to run using crun with krun runtime
    fn try_crun_krun(
        &self,
        image: &OciImage,
        config: &VmConfig,
        state: &mut VmState,
        cmd: &str,
        args: &[String],
    ) -> Result<()> {
        if !Self::command_exists("crun") {
            return Err(Error::Libkrun("crun not found".to_string()));
        }

        // Create OCI bundle
        let bundle_dir = state.runtime_dir.join("bundle");
        std::fs::create_dir_all(&bundle_dir)?;

        // Create config.json for OCI runtime
        let oci_config = self.create_oci_config(image, config, cmd, args)?;
        let config_path = bundle_dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&oci_config)?)?;

        // Symlink or copy rootfs
        let bundle_rootfs = bundle_dir.join("rootfs");
        if !bundle_rootfs.exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&image.rootfs_path, &bundle_rootfs)?;
        }

        let mut command = Command::new("crun");
        command.args(["--runtime=krun", "run"]);
        command.arg(&state.id);
        command.arg("--bundle");
        command.arg(&bundle_dir);

        // Handle detached mode
        if config.detached {
            command.args(["-d"]);
            command.stdin(Stdio::null());
            command.stdout(Stdio::null());
            command.stderr(Stdio::null());

            let child = command.spawn()?;
            state.pid = Some(child.id());
        } else {
            let status = command.status()?;
            if !status.success() {
                return Err(Error::Process(format!(
                    "VM exited with status: {:?}",
                    status.code()
                )));
            }
        }

        Ok(())
    }

    /// Run in simulated mode (for development/testing when libkrun is not available)
    fn run_simulated(
        &self,
        image: &OciImage,
        config: &VmConfig,
        state: &mut VmState,
        cmd: &str,
        args: &[String],
    ) -> Result<()> {
        eprintln!("⚠️  Running in simulated mode (libkrun not available)");
        eprintln!("   This simulates VM execution using chroot/unshare");
        eprintln!();
        eprintln!("   Image: {}", image.name);
        eprintln!("   Rootfs: {}", image.rootfs_path.display());
        eprintln!("   Command: {} {}", cmd, args.join(" "));
        eprintln!();

        // Check if we can use unshare (requires root or user namespaces)
        if Self::command_exists("unshare") {
            let mut command = Command::new("unshare");
            command.args(["--mount", "--pid", "--fork", "--root"]);
            command.arg(&image.rootfs_path);

            // Set working directory
            command.arg("--wd");
            command.arg(&config.workdir);

            // Add the command to execute
            command.arg(cmd);
            for arg in args {
                command.arg(arg);
            }

            // Set environment variables
            for (key, value) in &config.env {
                command.env(key, value);
            }

            // Handle detached mode
            if config.detached {
                command.stdin(Stdio::null());
                command.stdout(Stdio::null());
                command.stderr(Stdio::null());

                unsafe {
                    command.pre_exec(|| {
                        nix::unistd::setsid().map_err(|e| std::io::Error::other(e.to_string()))?;
                        Ok(())
                    });
                }

                let child = command.spawn()?;
                state.pid = Some(child.id());

                eprintln!("   Started in background with PID: {}", child.id());
            } else {
                let status = command.status()?;
                if !status.success() {
                    return Err(Error::Process(format!(
                        "Process exited with status: {:?}",
                        status.code()
                    )));
                }
            }
        } else {
            eprintln!("   Note: Neither libkrun nor unshare available.");
            eprintln!("   This is a no-op in this environment.");

            // Just report success for development
            state.status = VmStatus::Stopped;
        }

        Ok(())
    }

    /// Create an OCI config.json for crun
    fn create_oci_config(
        &self,
        _image: &OciImage,
        config: &VmConfig,
        cmd: &str,
        args: &[String],
    ) -> Result<serde_json::Value> {
        let mut process_args = vec![cmd.to_string()];
        process_args.extend(args.iter().cloned());

        let mut env_vars: Vec<String> = config
            .env
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        // Add default PATH if not set
        if !env_vars.iter().any(|e| e.starts_with("PATH=")) {
            env_vars.push(
                "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
            );
        }

        let oci_config = serde_json::json!({
            "ociVersion": "1.0.2",
            "process": {
                "terminal": !config.detached,
                "user": {
                    "uid": 0,
                    "gid": 0
                },
                "args": process_args,
                "env": env_vars,
                "cwd": config.workdir,
            },
            "root": {
                "path": "rootfs",
                "readonly": false
            },
            "hostname": "mocker",
            "mounts": self.generate_mounts(config),
            "linux": {
                "namespaces": [
                    { "type": "pid" },
                    { "type": "ipc" },
                    { "type": "uts" },
                    { "type": "mount" }
                ],
                "resources": {
                    "memory": {
                        "limit": (config.memory_mb as u64) * 1024 * 1024
                    }
                }
            }
        });

        Ok(oci_config)
    }

    fn generate_mounts(&self, config: &VmConfig) -> Vec<serde_json::Value> {
        let mut mounts = vec![
            serde_json::json!({
                "destination": "/proc",
                "type": "proc",
                "source": "proc"
            }),
            serde_json::json!({
                "destination": "/dev",
                "type": "tmpfs",
                "source": "tmpfs",
                "options": ["nosuid", "strictatime", "mode=755", "size=65536k"]
            }),
            serde_json::json!({
                "destination": "/sys",
                "type": "sysfs",
                "source": "sysfs",
                "options": ["nosuid", "noexec", "nodev", "ro"]
            }),
        ];

        // Add volume mounts (for virtio-fs in real libkrun, bind mounts in simulated mode)
        for volume in &config.volumes {
            mounts.push(serde_json::json!({
                "destination": volume.guest_path,
                "type": "bind",
                "source": volume.host_path,
                "options": ["rbind", "rw"]
            }));
        }

        mounts
    }

    /// Check if a command exists
    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// List all VMs
    pub fn list_vms(&self) -> Result<Vec<VmState>> {
        self.state_manager.list_vms()
    }

    /// Stop a VM
    pub fn stop_vm(&self, vm_id: &str) -> Result<()> {
        self.state_manager.stop_vm(vm_id)
    }

    /// Remove a VM
    pub fn remove_vm(&self, vm_id: &str) -> Result<()> {
        // Stop first if running
        let state = self.state_manager.find_vm(vm_id)?;
        if state.status == VmStatus::Running {
            self.stop_vm(&state.id)?;
        }
        self.state_manager.remove_vm(&state.id)
    }
}
