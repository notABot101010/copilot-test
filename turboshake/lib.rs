//! TurboSHAKE and KangarooTwelve hash functions based on RFC 9861
//!
//! This crate implements:
//! - TurboSHAKE128 (128-bit security)
//! - TurboSHAKE256 (256-bit security)
//! - KT128 (KangarooTwelve with 128-bit security)
//! - KT256 (KangarooTwelve with 256-bit security)
//! - TurboShakeAead (AEAD using duplex construction)
//!
//! All functions use the Keccak-p[1600,12] permutation (12 rounds) for
//! improved performance over SHA-3/SHAKE.

mod turboshake;
mod kangaroo;
mod turboshake_aead;

pub use turboshake::{TurboShake128, TurboShake256};
pub use kangaroo::{KT128, KT256, length_encode};
pub use turboshake_aead::{TurboShakeAead, AeadError};
