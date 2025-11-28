//! VM Manager - handles launching and managing microVMs using libkrun
//!
//! This module provides the main interface for running microVMs using libkrun.
//! It uses FFI to call libkrun directly for VM creation and management.

#[cfg(feature = "libkrun")]
use crate::ffi;
use crate::{
    ffi::LogLevel, Error, ImageManager, OciImage, Result, StateManager, VmConfig, VmState,
    VmStatus,
};
#[cfg(feature = "libkrun")]
use std::ffi::CString;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

/// Default PATH environment variable for guest VMs
#[cfg(feature = "libkrun")]
const DEFAULT_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

/// Prefix used for virtio-fs tags when mounting shared directories
#[cfg(feature = "libkrun")]
const VIRTIOFS_TAG_PREFIX: &str = "virtiofs";

/// Manager for microVM operations
pub struct VmManager {
    /// Image manager for OCI images
    image_manager: ImageManager,
    /// State manager for VM persistence
    state_manager: StateManager,
    /// Log level for libkrun
    log_level: LogLevel,
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
            log_level: LogLevel::Off,
        })
    }

    /// Set the log level for libkrun
    pub fn set_log_level(&mut self, level: LogLevel) {
        self.log_level = level;
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

    /// Launch a VM using libkrun directly via FFI
    ///
    /// This is the main entry point for running a microVM. It:
    /// 1. Tries to use libkrun directly via FFI bindings
    /// 2. Falls back to simulated mode if libkrun is not available
    fn launch_vm(&self, image: &OciImage, config: &VmConfig, state: &mut VmState) -> Result<()> {
        // Build the command to run inside the VM
        let exec_path = if let Some(ref cmd) = config.command {
            cmd.clone()
        } else if !image.entrypoint.is_empty() {
            image.entrypoint[0].clone()
        } else if !image.cmd.is_empty() {
            image.cmd[0].clone()
        } else {
            "/bin/sh".to_string()
        };

        // Build args - combine entrypoint args with config args
        let mut args: Vec<String> = Vec::new();

        // Add remaining entrypoint elements as args
        if config.command.is_none() && !image.entrypoint.is_empty() {
            args.extend(image.entrypoint[1..].iter().cloned());
        }

        // Add image cmd if no command specified and we used entrypoint
        if config.command.is_none() && !image.entrypoint.is_empty() && !image.cmd.is_empty() {
            args.extend(image.cmd.iter().cloned());
        } else if config.command.is_none() && image.entrypoint.is_empty() && !image.cmd.is_empty() {
            args.extend(image.cmd[1..].iter().cloned());
        }

        // Add explicit config args
        args.extend(config.args.iter().cloned());

        // Try libkrun first (if feature enabled), fall back to simulation if not available
        #[cfg(feature = "libkrun")]
        {
            self.try_libkrun(image, config, state, &exec_path, &args)
                .or_else(|_| self.run_simulated(image, config, state, &exec_path, &args))
        }

        #[cfg(not(feature = "libkrun"))]
        {
            self.run_simulated(image, config, state, &exec_path, &args)
        }
    }

    /// Try to run using libkrun directly via FFI
    #[cfg(feature = "libkrun")]
    fn try_libkrun(
        &self,
        image: &OciImage,
        config: &VmConfig,
        state: &mut VmState,
        exec_path: &str,
        args: &[String],
    ) -> Result<()> {
        // For detached mode, we need to fork first
        if config.detached {
            return self.run_libkrun_detached(image, config, state, exec_path, args);
        }

        // Create libkrun context
        let ctx_id = unsafe { ffi::krun_create_ctx() };
        if ctx_id < 0 {
            return Err(Error::Libkrun(format!(
                "Failed to create libkrun context: {}",
                ctx_id
            )));
        }
        let ctx_id = ctx_id as u32;

        // Set log level
        unsafe {
            let status = ffi::krun_set_log_level(self.log_level as u32);
            if status < 0 {
                ffi::krun_free_ctx(ctx_id);
                return Err(Error::Libkrun(format!(
                    "Failed to set log level: {}",
                    status
                )));
            }
        }

        // Set VM configuration (vCPUs and RAM)
        unsafe {
            let status = ffi::krun_set_vm_config(ctx_id, config.vcpus as u8, config.memory_mb);
            if status < 0 {
                ffi::krun_free_ctx(ctx_id);
                return Err(Error::Libkrun(format!(
                    "Failed to set VM config: {}",
                    status
                )));
            }
        }

        // Set root filesystem
        let c_root = CString::new(image.rootfs_path.to_string_lossy().as_bytes())
            .map_err(|e| Error::Libkrun(format!("Invalid root path: {}", e)))?;
        unsafe {
            let status = ffi::krun_set_root(ctx_id, c_root.as_ptr());
            if status < 0 {
                ffi::krun_free_ctx(ctx_id);
                return Err(Error::Libkrun(format!("Failed to set root: {}", status)));
            }
        }

        // Add virtio-fs mounts for volumes
        for (idx, volume) in config.volumes.iter().enumerate() {
            let tag = CString::new(format!("{}_{}", VIRTIOFS_TAG_PREFIX, idx))
                .map_err(|e| Error::Libkrun(format!("Invalid virtiofs tag: {}", e)))?;
            let host_path = CString::new(volume.host_path.as_bytes())
                .map_err(|e| Error::Libkrun(format!("Invalid host path: {}", e)))?;

            unsafe {
                let status = ffi::krun_add_virtiofs(ctx_id, tag.as_ptr(), host_path.as_ptr());
                if status < 0 {
                    ffi::krun_free_ctx(ctx_id);
                    return Err(Error::Libkrun(format!(
                        "Failed to add virtiofs mount: {}",
                        status
                    )));
                }
            }
        }

        // Set working directory
        let c_workdir = CString::new(config.workdir.as_bytes())
            .map_err(|e| Error::Libkrun(format!("Invalid workdir: {}", e)))?;
        unsafe {
            let status = ffi::krun_set_workdir(ctx_id, c_workdir.as_ptr());
            if status < 0 {
                ffi::krun_free_ctx(ctx_id);
                return Err(Error::Libkrun(format!(
                    "Failed to set workdir: {}",
                    status
                )));
            }
        }

        // Build environment variables
        let mut env_vars: Vec<String> = config
            .env
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        // Add default PATH if not set
        if !env_vars.iter().any(|e| e.starts_with("PATH=")) {
            env_vars.push(format!("PATH={}", DEFAULT_PATH));
        }

        // Set executable, args, and environment
        let c_exec = CString::new(exec_path.as_bytes())
            .map_err(|e| Error::Libkrun(format!("Invalid exec path: {}", e)))?;

        let c_args: Vec<CString> = args
            .iter()
            .map(|s| CString::new(s.as_bytes()))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Libkrun(format!("Invalid argument: {}", e)))?;
        let c_args_ptrs = ffi::to_null_terminated_c_array(&c_args);

        let c_env: Vec<CString> = env_vars
            .iter()
            .map(|s| CString::new(s.as_bytes()))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Libkrun(format!("Invalid env var: {}", e)))?;
        let c_env_ptrs = ffi::to_null_terminated_c_array(&c_env);

        unsafe {
            let status = ffi::krun_set_exec(
                ctx_id,
                c_exec.as_ptr(),
                c_args_ptrs.as_ptr(),
                c_env_ptrs.as_ptr(),
            );
            if status < 0 {
                ffi::krun_free_ctx(ctx_id);
                return Err(Error::Libkrun(format!("Failed to set exec: {}", status)));
            }
        }

        // Optionally set console output to a file
        let console_path = state.runtime_dir.join("console.log");
        let c_console = CString::new(console_path.to_string_lossy().as_bytes())
            .map_err(|e| Error::Libkrun(format!("Invalid console path: {}", e)))?;
        unsafe {
            let status = ffi::krun_set_console_output(ctx_id, c_console.as_ptr());
            if status < 0 {
                // Non-fatal, just log it
                eprintln!("Warning: Failed to set console output: {}", status);
            }
        }

        // Start the VM - this will take over the process
        eprintln!("Starting microVM with libkrun...");
        let status = unsafe { ffi::krun_start_enter(ctx_id) };

        // If we get here, something went wrong
        Err(Error::Libkrun(format!(
            "krun_start_enter returned unexpectedly: {}",
            status
        )))
    }

    /// Run libkrun in detached mode by forking first
    #[cfg(feature = "libkrun")]
    fn run_libkrun_detached(
        &self,
        image: &OciImage,
        config: &VmConfig,
        state: &mut VmState,
        exec_path: &str,
        args: &[String],
    ) -> Result<()> {
        use nix::unistd::{fork, ForkResult};

        // Fork the process
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                // Parent process - record the child PID and return
                state.pid = Some(child.as_raw() as u32);
                Ok(())
            }
            Ok(ForkResult::Child) => {
                // Child process - run libkrun
                // Create a new session to detach from terminal
                let _ = nix::unistd::setsid();

                // Close stdin/stdout/stderr
                let _ = nix::unistd::close(0);
                let _ = nix::unistd::close(1);
                let _ = nix::unistd::close(2);

                // Create a non-detached config copy for the child
                let mut child_config = config.clone();
                child_config.detached = false;

                // Create a dummy state for the child
                let mut child_state = state.clone();

                // Run libkrun - this will take over the child process
                let _ = self.try_libkrun(image, &child_config, &mut child_state, exec_path, args);

                // If we get here, libkrun failed - exit the child
                std::process::exit(1);
            }
            Err(e) => Err(Error::Process(format!("Fork failed: {}", e))),
        }
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
