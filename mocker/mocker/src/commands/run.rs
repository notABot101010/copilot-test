//! Run command implementation

use mocker_core::{VmConfig, VmManager, VmStatus, VolumeMount};

/// Run a microVM
pub fn run(
    manager: &VmManager,
    image: String,
    command: Vec<String>,
    volumes: Vec<String>,
    detach: bool,
    env: Vec<String>,
    workdir: Option<String>,
) -> mocker_core::Result<()> {
    // Check if image exists, if not, try to pull it
    if !manager.images().image_exists(&image) {
        eprintln!("Image '{}' not found locally, pulling...", image);
        manager.images().pull_image(&image)?;
    }

    // Parse volume mounts
    let mut parsed_volumes = Vec::new();
    for v in &volumes {
        parsed_volumes.push(VolumeMount::parse(v)?);
    }

    // Parse environment variables
    let mut parsed_env = Vec::new();
    for e in &env {
        if let Some((key, value)) = e.split_once('=') {
            parsed_env.push((key.to_string(), value.to_string()));
        } else {
            // Environment variable without value - check host environment
            if let Ok(value) = std::env::var(e) {
                parsed_env.push((e.clone(), value));
            }
        }
    }

    // Build config
    let mut config = VmConfig::new(&image).detached(detach);

    // Add volumes
    for volume in parsed_volumes {
        config = config.volume(volume);
    }

    // Add environment variables
    for (key, value) in parsed_env {
        config = config.env(key, value);
    }

    // Set workdir if provided
    if let Some(wd) = workdir {
        config = config.workdir(wd);
    }

    // Set command if provided
    if !command.is_empty() {
        config = config.command(&command[0]);
        if command.len() > 1 {
            config = config.args(command[1..].to_vec());
        }
    }

    // Run the VM
    let state = manager.run(&config)?;

    if detach {
        println!("{}", state.short_id);
    } else if state.status == VmStatus::Stopped {
        println!("VM {} exited", state.short_id);
    }

    Ok(())
}
