//! mocker - A Docker-like CLI for Linux microVMs using libkrun

use clap::{Parser, Subcommand};
use mocker_core::VmManager;
use std::path::PathBuf;

mod commands;

/// A Docker-like CLI for Linux microVMs using libkrun
#[derive(Parser)]
#[command(name = "mocker")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the mocker data directory
    #[arg(long, env = "MOCKER_DATA_DIR")]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a command in a new microVM
    Run {
        /// Image to use
        image: String,

        /// Command to run in the VM
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,

        /// Mount a volume (format: host_path:guest_path)
        #[arg(short = 'v', long = "volume")]
        volumes: Vec<String>,

        /// Run container in background (detached mode)
        #[arg(short = 'd', long = "detach")]
        detach: bool,

        /// Set environment variables
        #[arg(short = 'e', long = "env")]
        env: Vec<String>,

        /// Working directory inside the VM
        #[arg(short = 'w', long = "workdir")]
        workdir: Option<String>,
    },

    /// Pull an OCI image and convert it for microVM use
    Pull {
        /// Image name (e.g., alpine:latest, docker.io/library/nginx:latest)
        image: String,
    },

    /// List images
    Images,

    /// List running VMs
    Ps {
        /// Show all VMs (including stopped)
        #[arg(short = 'a', long = "all")]
        all: bool,
    },

    /// Stop a running VM
    Stop {
        /// VM ID or short ID
        vm_id: String,
    },

    /// Remove a VM
    Rm {
        /// VM ID or short ID
        vm_id: String,

        /// Force removal (stop if running)
        #[arg(short = 'f', long = "force")]
        force: bool,
    },

    /// Remove an image
    Rmi {
        /// Image name
        image: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> mocker_core::Result<()> {
    let cli = Cli::parse();

    // Determine data directory
    let data_dir = cli.data_dir.unwrap_or_else(|| {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mocker")
    });

    // Create VM manager
    let manager = VmManager::new(&data_dir)?;

    match cli.command {
        Commands::Run {
            image,
            command,
            volumes,
            detach,
            env,
            workdir,
        } => commands::run(&manager, image, command, volumes, detach, env, workdir),
        Commands::Pull { image } => commands::pull(&manager, &image),
        Commands::Images => commands::list_images(&manager),
        Commands::Ps { all } => commands::list_vms(&manager, all),
        Commands::Stop { vm_id } => commands::stop_vm(&manager, &vm_id),
        Commands::Rm { vm_id, force } => commands::remove_vm(&manager, &vm_id, force),
        Commands::Rmi { image } => commands::remove_image(&manager, &image),
    }
}

/// Helper to get the default data directory
mod dirs {
    use std::path::PathBuf;

    pub fn data_local_dir() -> Option<PathBuf> {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
    }
}
