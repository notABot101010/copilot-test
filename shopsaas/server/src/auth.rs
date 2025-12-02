//! Authentication utilities for ShopSaaS

use aws_lc_rs::rand::{SecureRandom, SystemRandom};

/// Generate a random session token
pub fn generate_token() -> String {
    let rng = SystemRandom::new();
    let mut token_bytes = [0u8; 32];
    if rng.fill(&mut token_bytes).is_err() {
        // Fallback to uuid if random fails
        return uuid::Uuid::new_v4().to_string();
    }
    hex::encode(token_bytes)
}

/// Hash a password using SHA-256 with salt
pub fn hash_password(password: &str) -> String {
    let rng = SystemRandom::new();
    let mut salt = [0u8; 16];
    if rng.fill(&mut salt).is_err() {
        // Fallback salt
        salt = [0u8; 16];
    }
    
    let salt_hex = hex::encode(salt);
    let salted = format!("{}{}", salt_hex, password);
    
    use aws_lc_rs::digest::{digest, SHA256};
    let hash = digest(&SHA256, salted.as_bytes());
    let hash_hex = hex::encode(hash.as_ref());
    
    format!("{}${}", salt_hex, hash_hex)
}

/// Verify a password against a stored hash
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    let parts: Vec<&str> = stored_hash.split('$').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let salt_hex = parts[0];
    let stored_hash_hex = parts[1];
    
    let salted = format!("{}{}", salt_hex, password);
    
    use aws_lc_rs::digest::{digest, SHA256};
    let hash = digest(&SHA256, salted.as_bytes());
    let hash_hex = hex::encode(hash.as_ref());
    
    hash_hex == stored_hash_hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password123";
        let hash = hash_password(password);
        
        assert!(verify_password(password, &hash));
        assert!(!verify_password("wrong_password", &hash));
    }

    #[test]
    fn test_token_generation() {
        let token1 = generate_token();
        let token2 = generate_token();
        
        assert_ne!(token1, token2);
        assert!(!token1.is_empty());
    }
}
