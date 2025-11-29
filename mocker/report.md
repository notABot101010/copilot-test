# Mocker - MicroVM Manager Report

## Overview

Mocker is a Docker-like command-line interface for Linux microVMs using libkrun. It provides a familiar Docker-style workflow for running containers in isolated microVMs instead of using traditional Linux container technologies.

## Architecture

The project is split into two crates as requested:

### mocker-core

The core library that handles:
- **krun-sys Integration**: Uses the official `krun-sys` crate for libkrun Rust bindings
- **VmConfig**: Configuration for microVMs (vCPUs, memory, volumes, environment variables)
- **ImageManager**: OCI image pulling and management
- **StateManager**: Persistence of VM state (running, stopped, failed)
- **VmManager**: High-level VM lifecycle management using krun-sys

### mocker (CLI)

The command-line interface using clap derive:
- `mocker run [image]` - Run a microVM
- `mocker pull [image]` - Pull an OCI image
- `mocker images` - List local images
- `mocker ps` - List running/all VMs
- `mocker stop` - Stop a VM
- `mocker rm` - Remove a VM
- `mocker rmi` - Remove an image

## Implementation Details

### libkrun Integration (krun-sys)

The implementation uses the official `krun-sys` crate (v1.10.1) from crates.io for libkrun bindings. The crate provides auto-generated Rust bindings to the libkrun C library. Key functions used:

- `krun_sys::krun_create_ctx()` - Create a VM context
- `krun_sys::krun_set_vm_config()` - Configure vCPUs and RAM
- `krun_sys::krun_set_root()` - Set the root filesystem
- `krun_sys::krun_add_virtiofs()` - Add virtio-fs mounts for volumes
- `krun_sys::krun_set_workdir()` - Set the working directory
- `krun_sys::krun_set_exec()` - Set the executable, arguments, and environment
- `krun_sys::krun_set_console_output()` - Configure console output
- `krun_sys::krun_start_enter()` - Start the VM

The krun-sys dependency is optional and guarded by the `libkrun` feature flag. When the feature is not enabled, the code falls back to a simulation mode using `unshare`.

### Image Pulling

The image pulling system supports multiple container tools in order of preference:
1. **skopeo + umoci** - The preferred method for OCI-native image handling
2. **podman** - Falls back to Podman if available
3. **docker** - Falls back to Docker as last resort

Images are extracted to a rootfs directory structure that can be used directly by libkrun.

### Volume Mounting

Volume mounting uses the Docker-style `-v host_path:guest_path` syntax. When running with libkrun, these are exposed via virtio-fs using `krun_add_virtiofs()` for efficient host-guest file sharing.

### Detached Mode

The `-d` flag allows running VMs in the background. For libkrun, this is handled by forking the process before calling `krun_start_enter()`. The VM process is daemonized using `setsid()` and its PID is tracked in the state file.

### State Management

VM state is persisted in JSON files under `~/.local/share/mocker/state/<vm-id>/`. This includes:
- VM configuration
- Process ID (if running)
- Status (creating, running, stopped, failed)
- Creation timestamp

## Building with libkrun

To build with libkrun support, you need:

1. Install libkrun and libkrunfw on your system
2. Build with the `libkrun` feature:

```bash
cd mocker
cargo build --release --features libkrun
```

Without the `libkrun` feature, the tool falls back to simulation mode using `unshare`.

## Limitations

### libkrun Installation

libkrun must be installed on the system for VM execution. On most systems, you can:
- On Fedora/RHEL: `dnf install libkrun libkrunfw`
- On macOS: Build from source with HVF support
- From source: Follow instructions at https://github.com/containers/libkrun

### Root Privileges

Running actual microVMs or even the simulated mode typically requires:
- Root privileges, OR
- User namespace support (`/proc/sys/kernel/unprivileged_userns_clone = 1`)

### macOS Support

While the code is designed to be cross-platform:
- libkrun requires the EFI variant on macOS
- Image pulling works on macOS if Docker/podman is available
- VM execution requires libkrun to be properly installed with HVF entitlements
- Binary must be signed with hypervisor entitlements

### No Native Networking

The current implementation uses TSI (Transparent Socket Impersonation) by default when libkrun is available. For more complex networking, users would need to set up passt or gvproxy.

### Single Command Model

Unlike Docker, there's currently no concept of a "container" that can be started/stopped multiple times. Each `run` creates a new VM instance.

## Dependencies

All dependencies are well-maintained crates with high download counts:
- **clap** (4.5.x) - Command-line parsing with derive macros
- **serde** (1.0.x) - Serialization/deserialization
- **serde_json** (1.0.x) - JSON support
- **thiserror** (2.0.x) - Error handling derive macro
- **uuid** (1.18.x) - UUID generation for VM IDs
- **nix** (0.30.x) - Unix system call bindings
- **libc** (0.2.x) - C library bindings for FFI

## Future Improvements

### Short-term

1. **Better error messages**: Provide more helpful guidance when libkrun is not installed
2. **Logging**: Add structured logging with configurable verbosity
3. **Port mapping**: Implement `-p` flag for port forwarding using `krun_set_port_map()`

### Medium-term

1. **Image layers**: Support proper OCI image layer caching to avoid re-downloading
2. **Build support**: Add `mocker build` command for building images
3. **Network modes**: Support different networking backends (TSI, passt, gvproxy)
4. **Resource limits**: Better control over CPU and memory limits using `krun_set_rlimits()`

### Long-term

1. **GPU passthrough**: Support virtio-gpu using `krun_set_gpu_options()`
2. **Compose support**: A Docker Compose-like multi-VM orchestration
3. **Remote API**: HTTP API for remote VM management
4. **Checkpoint/restore**: Save and restore VM state

## Testing

The project includes basic unit tests for configuration parsing. To run tests:

```bash
cd mocker
cargo test
```

## Usage Examples

```bash
# Pull an image
mocker pull alpine:latest

# Run a command
mocker run alpine:latest /bin/ls

# Run in background
mocker run -d alpine:latest /bin/sh

# Mount a volume
mocker run -v /host/data:/data alpine:latest /bin/sh

# Set environment variables
mocker run -e FOO=bar alpine:latest /bin/sh

# List VMs
mocker ps -a

# Stop a VM
mocker stop <vm-id>

# Remove a VM
mocker rm <vm-id>
```

## Building

```bash
cd mocker

# Without libkrun (simulation mode only)
cargo build --release

# With libkrun support
cargo build --release --features libkrun
```

The binary will be at `target/release/mocker`.

## Platform Notes

### Linux
- Works best with libkrun and libkrunfw installed
- Enable the `libkrun` feature flag when building
- Fallback simulation mode uses unshare (requires root or user namespaces)

### macOS
- Requires libkrun-efi variant
- Requires macOS 14+ for HVF support
- Binary must be signed with hypervisor entitlements

## References

- [libkrun](https://github.com/containers/libkrun) - The virtualization library used
- [microsandbox](https://github.com/zerocore-ai/microsandbox) - Reference implementation for libkrun FFI usage

## Conclusion

This implementation provides a Docker-like microVM manager using direct libkrun FFI bindings. The architecture allows for:
- Native libkrun integration via FFI (no CLI shelling out)
- Feature-flag controlled compilation for systems without libkrun
- Fallback simulation mode for development/testing

The split architecture (core library + CLI) makes it easy to add alternative interfaces (TUI, GUI, API) in the future while reusing the core functionality.
