//! FFI bindings for libkrun
//!
//! This module provides Rust bindings for the libkrun C library.
//! libkrun is a library that allows running processes inside lightweight VMs.
//!
//! Note: This module requires the `libkrun` feature to be enabled and
//! the libkrun library to be installed on the system.

#![allow(dead_code)]

use std::ffi::c_char;

#[cfg(feature = "libkrun")]
#[link(name = "krun")]
extern "C" {
    /// Sets the log level for the library.
    ///
    /// ## Arguments
    /// * `level` - The log level (0=Off, 1=Error, 2=Warn, 3=Info, 4=Debug, 5=Trace)
    ///
    /// ## Returns
    /// 0 on success, negative error code on failure.
    pub fn krun_set_log_level(level: u32) -> i32;

    /// Creates a configuration context.
    ///
    /// ## Returns
    /// Context ID on success, negative error code on failure.
    pub fn krun_create_ctx() -> i32;

    /// Frees an existing configuration context.
    ///
    /// ## Arguments
    /// * `ctx_id` - The configuration context ID to free.
    ///
    /// ## Returns
    /// 0 on success, negative error code on failure.
    pub fn krun_free_ctx(ctx_id: u32) -> i32;

    /// Sets the basic configuration parameters for the MicroVm.
    ///
    /// ## Arguments
    /// * `ctx_id` - The configuration context ID.
    /// * `num_vcpus` - The number of vCPUs.
    /// * `ram_mib` - The amount of RAM in MiB.
    ///
    /// ## Returns
    /// 0 on success, negative error code on failure.
    pub fn krun_set_vm_config(ctx_id: u32, num_vcpus: u8, ram_mib: u32) -> i32;

    /// Sets the path to be used as root for the MicroVm.
    ///
    /// ## Arguments
    /// * `ctx_id` - The configuration context ID.
    /// * `root_path` - The path to be used as root.
    ///
    /// ## Returns
    /// 0 on success, negative error code on failure.
    pub fn krun_set_root(ctx_id: u32, root_path: *const c_char) -> i32;

    /// Adds an independent virtio-fs device pointing to a host's directory with a tag.
    ///
    /// ## Arguments
    /// * `ctx_id` - The configuration context ID.
    /// * `tag` - The tag to identify the filesystem in the guest.
    /// * `path` - The full path to the host's directory to be exposed to the guest.
    ///
    /// ## Returns
    /// 0 on success, negative error code on failure.
    pub fn krun_add_virtiofs(ctx_id: u32, tag: *const c_char, path: *const c_char) -> i32;

    /// Configures a map of host to guest TCP ports for the MicroVm.
    ///
    /// ## Arguments
    /// * `ctx_id` - The configuration context ID.
    /// * `port_map` - A null-terminated array of string pointers with format "host_port:guest_port".
    ///
    /// ## Returns
    /// 0 on success, negative error code on failure.
    pub fn krun_set_port_map(ctx_id: u32, port_map: *const *const c_char) -> i32;

    /// Configures a map of rlimits to be set in the guest.
    ///
    /// ## Arguments
    /// * `ctx_id` - The configuration context ID.
    /// * `rlimits` - A null-terminated array of string pointers with format "RESOURCE=SOFT:HARD".
    ///
    /// ## Returns
    /// 0 on success, negative error code on failure.
    pub fn krun_set_rlimits(ctx_id: u32, rlimits: *const *const c_char) -> i32;

    /// Sets the working directory for the executable to be run inside the MicroVm.
    ///
    /// ## Arguments
    /// * `ctx_id` - The configuration context ID.
    /// * `workdir_path` - The path to the working directory, relative to the root.
    ///
    /// ## Returns
    /// 0 on success, negative error code on failure.
    pub fn krun_set_workdir(ctx_id: u32, workdir_path: *const c_char) -> i32;

    /// Sets the path to the executable to be run inside the MicroVm, with arguments and environment.
    ///
    /// ## Arguments
    /// * `ctx_id` - The configuration context ID.
    /// * `exec_path` - The path to the executable, relative to the root.
    /// * `argv` - A null-terminated array of string pointers to be passed as arguments.
    /// * `envp` - A null-terminated array of string pointers for environment variables.
    ///
    /// ## Returns
    /// 0 on success, negative error code on failure.
    pub fn krun_set_exec(
        ctx_id: u32,
        exec_path: *const c_char,
        argv: *const *const c_char,
        envp: *const *const c_char,
    ) -> i32;

    /// Sets the path to the file to write the console output for the MicroVm.
    ///
    /// ## Arguments
    /// * `ctx_id` - The configuration context ID.
    /// * `filepath` - The path of the file to write the console output.
    ///
    /// ## Returns
    /// 0 on success, negative error code on failure.
    pub fn krun_set_console_output(ctx_id: u32, filepath: *const c_char) -> i32;

    /// Starts and enters the MicroVm with the configured parameters.
    ///
    /// This function will attempt to take over stdin/stdout to manage them on behalf of the
    /// process running inside the isolated environment.
    ///
    /// ## Arguments
    /// * `ctx_id` - The configuration context ID.
    ///
    /// ## Returns
    /// This function only returns if an error happens before starting the MicroVm.
    /// Otherwise, the VMM assumes full control of the process and will call exit() once the MicroVm shuts down.
    pub fn krun_start_enter(ctx_id: u32) -> i32;
}

/// Log level for libkrun
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum LogLevel {
    /// No logging
    #[default]
    Off = 0,
    /// Error messages only
    Error = 1,
    /// Warnings and errors
    Warn = 2,
    /// Informational messages
    Info = 3,
    /// Debug messages
    Debug = 4,
    /// Trace messages (most verbose)
    Trace = 5,
}

/// Helper function to convert a Vec<CString> to a null-terminated array of C string pointers
pub fn to_null_terminated_c_array(strings: &[std::ffi::CString]) -> Vec<*const c_char> {
    let mut ptrs: Vec<*const c_char> = strings.iter().map(|s| s.as_ptr()).collect();
    ptrs.push(std::ptr::null());
    ptrs
}

/// Check if libkrun is available at runtime
#[cfg(feature = "libkrun")]
pub fn is_libkrun_available() -> bool {
    true
}

/// Check if libkrun is available at runtime
#[cfg(not(feature = "libkrun"))]
pub fn is_libkrun_available() -> bool {
    false
}
