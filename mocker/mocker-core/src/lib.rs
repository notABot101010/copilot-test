//! mocker-core: Core library for managing microVMs with libkrun
//!
//! This library provides the core functionality for running and managing
//! microVMs using libkrun as the Virtual Machine Monitor.

mod config;
mod error;
mod ffi;
mod image;
mod state;
mod vm;

pub use config::{VmConfig, VolumeMount};
pub use error::{Error, Result};
pub use ffi::LogLevel;
pub use image::{ImageManager, OciImage};
pub use state::{StateManager, VmState, VmStatus};
pub use vm::VmManager;
