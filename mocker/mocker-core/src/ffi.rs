//! Wrapper module for krun-sys bindings
//!
//! This module provides a thin wrapper around the krun-sys crate for libkrun.
//! krun-sys provides Rust bindings for the libkrun C library which allows
//! running processes inside lightweight VMs.
//!
//! Note: This module requires the `libkrun` feature to be enabled and
//! the libkrun library to be installed on the system.

#![allow(dead_code)]

use std::ffi::c_char;

/// Re-export krun-sys when the libkrun feature is enabled
#[cfg(feature = "libkrun")]
pub use krun_sys::{
    krun_add_virtiofs, krun_create_ctx, krun_free_ctx, krun_set_console_output, krun_set_exec,
    krun_set_log_level, krun_set_root, krun_set_vm_config, krun_set_workdir, krun_start_enter,
};

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

/// RAII wrapper for libkrun context that automatically frees the context on drop
#[cfg(feature = "libkrun")]
pub struct KrunContext {
    ctx_id: u32,
    /// If true, the context has been consumed (e.g., by krun_start_enter) and should not be freed
    consumed: bool,
}

#[cfg(feature = "libkrun")]
impl KrunContext {
    /// Create a new libkrun context
    pub fn new() -> Result<Self, i32> {
        // SAFETY: krun_create_ctx is a safe FFI call that creates a new context
        let ctx_id = unsafe { krun_sys::krun_create_ctx() };
        if ctx_id < 0 {
            return Err(ctx_id);
        }
        Ok(Self {
            ctx_id: ctx_id as u32,
            consumed: false,
        })
    }

    /// Get the context ID
    pub fn id(&self) -> u32 {
        self.ctx_id
    }

    /// Mark the context as consumed (will not be freed on drop)
    pub fn consume(mut self) -> u32 {
        self.consumed = true;
        self.ctx_id
    }
}

#[cfg(feature = "libkrun")]
impl Drop for KrunContext {
    fn drop(&mut self) {
        if !self.consumed {
            // SAFETY: krun_free_ctx is a safe FFI call that frees the context
            unsafe {
                krun_sys::krun_free_ctx(self.ctx_id);
            }
        }
    }
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
