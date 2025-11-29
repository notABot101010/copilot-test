//! List commands implementation

use mocker_core::{VmManager, VmStatus};

/// List images
pub fn list_images(manager: &VmManager) -> mocker_core::Result<()> {
    let images = manager.images().list_images()?;

    if images.is_empty() {
        println!("No images found.");
        return Ok(());
    }

    println!("{:<50} {:<20}", "IMAGE", "ROOTFS");
    println!("{}", "-".repeat(70));

    for image in images {
        println!("{:<50} {:<20}", image.name, image.rootfs_path.display());
    }

    Ok(())
}

/// List VMs
pub fn list_vms(manager: &VmManager, show_all: bool) -> mocker_core::Result<()> {
    let vms = manager.list_vms()?;

    // Filter if not showing all
    let vms: Vec<_> = if show_all {
        vms
    } else {
        vms.into_iter()
            .filter(|vm| vm.status == VmStatus::Running)
            .collect()
    };

    if vms.is_empty() {
        if show_all {
            println!("No VMs found.");
        } else {
            println!("No running VMs. Use -a to show all.");
        }
        return Ok(());
    }

    println!(
        "{:<14} {:<30} {:<15} {:<10}",
        "VM ID", "IMAGE", "STATUS", "PID"
    );
    println!("{}", "-".repeat(70));

    for vm in vms {
        println!(
            "{:<14} {:<30} {:<15} {:<10}",
            vm.short_id,
            truncate(&vm.image, 28),
            vm.status.to_string(),
            vm.pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string())
        );
    }

    Ok(())
}

/// Truncate a string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
