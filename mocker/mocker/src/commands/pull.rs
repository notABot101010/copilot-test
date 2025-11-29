//! Pull command implementation

use mocker_core::VmManager;

/// Pull an OCI image
pub fn pull(manager: &VmManager, image: &str) -> mocker_core::Result<()> {
    println!("Pulling image: {}", image);

    let oci_image = manager.images().pull_image(image)?;

    println!("Successfully pulled: {}", oci_image.name);
    println!("Rootfs path: {}", oci_image.rootfs_path.display());

    if !oci_image.cmd.is_empty() {
        println!("Default cmd: {}", oci_image.cmd.join(" "));
    }

    Ok(())
}
