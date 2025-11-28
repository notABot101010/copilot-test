//! State management for running VMs

use crate::{Error, Result, VmConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// State of a running or stopped VM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmState {
    /// Unique VM ID
    pub id: String,

    /// Short ID (first 12 characters)
    pub short_id: String,

    /// Image name used
    pub image: String,

    /// Process ID of the VM (if running)
    pub pid: Option<u32>,

    /// Status (running, stopped, etc.)
    pub status: VmStatus,

    /// Configuration used to start the VM
    pub config: VmConfig,

    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,

    /// Path to the VM's runtime directory
    pub runtime_dir: PathBuf,
}

/// Status of a VM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VmStatus {
    /// VM is running
    Running,
    /// VM has stopped
    Stopped,
    /// VM is being created
    Creating,
    /// VM failed to start
    Failed,
}

impl std::fmt::Display for VmStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmStatus::Running => write!(f, "running"),
            VmStatus::Stopped => write!(f, "stopped"),
            VmStatus::Creating => write!(f, "creating"),
            VmStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Manager for VM state persistence
pub struct StateManager {
    /// Base directory for state storage
    state_dir: PathBuf,
}

impl StateManager {
    /// Create a new StateManager with the given base directory
    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let state_dir = base_dir.as_ref().join("state");
        fs::create_dir_all(&state_dir)?;
        Ok(Self { state_dir })
    }

    /// Create a new VM state entry
    pub fn create_vm_state(&self, config: &VmConfig) -> Result<VmState> {
        let id = Uuid::new_v4().to_string();
        let short_id = id[..12].to_string();

        let runtime_dir = self.state_dir.join(&id);
        fs::create_dir_all(&runtime_dir)?;

        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let state = VmState {
            id,
            short_id,
            image: config.image.clone(),
            pid: None,
            status: VmStatus::Creating,
            config: config.clone(),
            created_at,
            runtime_dir,
        };

        self.save_state(&state)?;
        Ok(state)
    }

    /// Save VM state to disk
    pub fn save_state(&self, state: &VmState) -> Result<()> {
        let state_file = state.runtime_dir.join("state.json");
        let content = serde_json::to_string_pretty(state)?;
        fs::write(state_file, content)?;
        Ok(())
    }

    /// Load VM state from disk
    pub fn load_state(&self, vm_id: &str) -> Result<VmState> {
        let runtime_dir = self.state_dir.join(vm_id);
        let state_file = runtime_dir.join("state.json");

        if !state_file.exists() {
            return Err(Error::VmNotFound(vm_id.to_string()));
        }

        let content = fs::read_to_string(state_file)?;
        let state: VmState = serde_json::from_str(&content)?;
        Ok(state)
    }

    /// Update VM state
    pub fn update_state(&self, state: &mut VmState) -> Result<()> {
        // Check if the process is still running
        if let Some(pid) = state.pid {
            if !Self::process_exists(pid) {
                state.status = VmStatus::Stopped;
                state.pid = None;
            }
        }
        self.save_state(state)?;
        Ok(())
    }

    /// Check if a process exists
    fn process_exists(pid: u32) -> bool {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        // Sending signal 0 checks if the process exists without affecting it
        kill(Pid::from_raw(pid as i32), Signal::SIGCONT).is_ok()
    }

    /// List all VM states
    pub fn list_vms(&self) -> Result<Vec<VmState>> {
        let mut vms = Vec::new();

        if !self.state_dir.exists() {
            return Ok(vms);
        }

        for entry in fs::read_dir(&self.state_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let state_file = entry.path().join("state.json");
                if state_file.exists() {
                    let content = fs::read_to_string(&state_file)?;
                    if let Ok(mut state) = serde_json::from_str::<VmState>(&content) {
                        // Update process status
                        if let Some(pid) = state.pid {
                            if !Self::process_exists(pid) {
                                state.status = VmStatus::Stopped;
                                state.pid = None;
                                let _ = self.save_state(&state);
                            }
                        }
                        vms.push(state);
                    }
                }
            }
        }

        Ok(vms)
    }

    /// Find a VM by ID or short ID
    pub fn find_vm(&self, id_or_short: &str) -> Result<VmState> {
        // Try exact match first
        if let Ok(state) = self.load_state(id_or_short) {
            return Ok(state);
        }

        // Try to find by short ID prefix
        for vm in self.list_vms()? {
            if vm.id.starts_with(id_or_short) || vm.short_id.starts_with(id_or_short) {
                return Ok(vm);
            }
        }

        Err(Error::VmNotFound(id_or_short.to_string()))
    }

    /// Remove a VM state
    pub fn remove_vm(&self, vm_id: &str) -> Result<()> {
        let runtime_dir = self.state_dir.join(vm_id);
        if !runtime_dir.exists() {
            return Err(Error::VmNotFound(vm_id.to_string()));
        }
        fs::remove_dir_all(runtime_dir)?;
        Ok(())
    }

    /// Stop a VM by killing its process
    pub fn stop_vm(&self, vm_id: &str) -> Result<()> {
        let mut state = self.find_vm(vm_id)?;

        if let Some(pid) = state.pid {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            // Try SIGTERM first
            let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);

            // Wait a bit and check if stopped
            std::thread::sleep(std::time::Duration::from_millis(500));

            if Self::process_exists(pid) {
                // Force kill with SIGKILL
                let _ = kill(Pid::from_raw(pid as i32), Signal::SIGKILL);
            }

            state.status = VmStatus::Stopped;
            state.pid = None;
            self.save_state(&state)?;
        }

        Ok(())
    }
}
