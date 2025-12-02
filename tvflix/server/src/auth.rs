//! Authentication utilities

use aws_lc_rs::rand::{SecureRandom, SystemRandom};
use aws_lc_rs::pbkdf2;
use std::num::NonZeroU32;

// PBKDF2 parameters
const PBKDF2_ITERATIONS: u32 = 100_000;
static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;

/// Hash a password using PBKDF2-HMAC-SHA256
/// Uses 100,000 iterations for security
pub fn hash_password(password: &str) -> String {
    let rng = SystemRandom::new();
    let mut salt = [0u8; 16];
    rng.fill(&mut salt).expect("Failed to generate salt");
    
    let mut hash = [0u8; 32];
    let iterations = NonZeroU32::new(PBKDF2_ITERATIONS).expect("iterations should be non-zero");
    
    pbkdf2::derive(
        PBKDF2_ALG,
        iterations,
        &salt,
        password.as_bytes(),
        &mut hash,
    );
    
    format!("{}${}", hex::encode(salt), hex::encode(hash))
}

/// Verify a password against a PBKDF2 hash
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    let parts: Vec<&str> = stored_hash.split('$').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let salt = match hex::decode(parts[0]) {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    let expected_hash = match hex::decode(parts[1]) {
        Ok(h) => h,
        Err(_) => return false,
    };
    
    let iterations = NonZeroU32::new(PBKDF2_ITERATIONS).expect("iterations should be non-zero");
    
    pbkdf2::verify(
        PBKDF2_ALG,
        iterations,
        &salt,
        password.as_bytes(),
        &expected_hash,
    ).is_ok()
}

/// Generate a random session token
pub fn generate_token() -> String {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes).expect("Failed to generate token");
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "test_password_123";
        let hash = hash_password(password);
        
        assert!(verify_password(password, &hash));
        assert!(!verify_password("wrong_password", &hash));
    }

    #[test]
    fn test_generate_token() {
        let token1 = generate_token();
        let token2 = generate_token();
        
        assert_eq!(token1.len(), 64); // 32 bytes = 64 hex chars
        assert_ne!(token1, token2);
    }
}
