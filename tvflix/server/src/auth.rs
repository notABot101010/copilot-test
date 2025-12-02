//! Authentication utilities

use aws_lc_rs::rand::{SecureRandom, SystemRandom};

/// Hash a password using a simple approach
/// In production, use argon2 or bcrypt
pub fn hash_password(password: &str) -> String {
    use aws_lc_rs::digest::{digest, SHA256};
    
    let rng = SystemRandom::new();
    let mut salt = [0u8; 16];
    rng.fill(&mut salt).expect("Failed to generate salt");
    
    let salted = format!("{}{}", hex::encode(salt), password);
    let hash = digest(&SHA256, salted.as_bytes());
    
    format!("{}${}", hex::encode(salt), hex::encode(hash.as_ref()))
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> bool {
    use aws_lc_rs::digest::{digest, SHA256};
    
    let parts: Vec<&str> = hash.split('$').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let salt = parts[0];
    let stored_hash = parts[1];
    
    let salted = format!("{}{}", salt, password);
    let computed_hash = digest(&SHA256, salted.as_bytes());
    
    hex::encode(computed_hash.as_ref()) == stored_hash
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
