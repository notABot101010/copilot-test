//! Management commands (stop, rm, rmi)

use mocker_core::{VmManager, VmStatus};

/// Stop a VM
pub fn stop_vm(manager: &VmManager, vm_id: &str) -> mocker_core::Result<()> {
    let state = manager.state().find_vm(vm_id)?;

    if state.status != VmStatus::Running {
        eprintln!("VM {} is not running", state.short_id);
        return Ok(());
    }

    manager.stop_vm(&state.id)?;
    println!("Stopped VM: {}", state.short_id);

    Ok(())
}

/// Remove a VM
pub fn remove_vm(manager: &VmManager, vm_id: &str, force: bool) -> mocker_core::Result<()> {
    let state = manager.state().find_vm(vm_id)?;

    if state.status == VmStatus::Running {
        if force {
            manager.stop_vm(&state.id)?;
        } else {
            eprintln!(
                "Error: VM {} is still running. Use -f to force removal.",
                state.short_id
            );
            return Ok(());
        }
    }

    manager.remove_vm(&state.id)?;
    println!("Removed VM: {}", state.short_id);

    Ok(())
}

/// Remove an image
pub fn remove_image(manager: &VmManager, image: &str) -> mocker_core::Result<()> {
    manager.images().remove_image(image)?;
    println!("Removed image: {}", image);
    Ok(())
}
