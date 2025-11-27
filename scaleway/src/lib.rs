//! Scaleway API Client
//!
//! A Rust client library for the Scaleway API, providing access to:
//! - Key Manager API
//! - Instances API
//! - Managed Inference API
//! - Elastic Metal API

mod client;
mod elastic_metal;
mod inference;
mod instances;
mod key_manager;

pub use client::{ApiError, Client, Error};
pub use elastic_metal::*;
pub use inference::*;
pub use instances::*;
pub use key_manager::*;

#[cfg(test)]
mod tests;
