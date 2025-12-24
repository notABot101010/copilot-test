use anyhow::{anyhow, Context, Result};
use argon2::{Argon2, ParamsBuilder, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Size constants for cryptographic primitives
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;
const MAGIC_NUMBER: &[u8; 8] = b"TUIPASS1";
const VERSION: u32 = 1;

/// Argon2id parameters
const ARGON2_MEMORY_KB: u32 = 65536; // 64 MB
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_PARALLELISM: u32 = 4;

/// A credential entry in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
}

/// The vault containing all credentials
#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    pub credentials: Vec<Credential>,
}

impl Vault {
    pub fn new() -> Self {
        Self {
            credentials: Vec::new(),
        }
    }
}

/// Master key derived from password, automatically zeroized on drop
#[derive(Zeroize, ZeroizeOnDrop)]
struct MasterKey {
    key: [u8; KEY_SIZE],
}

impl MasterKey {
    fn derive(password: &str, salt: &[u8; SALT_SIZE]) -> Result<Self> {
        let mut key = [0u8; KEY_SIZE];

        let params = ParamsBuilder::new()
            .m_cost(ARGON2_MEMORY_KB)
            .t_cost(ARGON2_ITERATIONS)
            .p_cost(ARGON2_PARALLELISM)
            .build()
            .map_err(|e| anyhow!("Failed to build Argon2 parameters: {}", e))?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| anyhow!("Failed to derive key: {}", e))?;

        Ok(MasterKey { key })
    }

    fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.key
    }
}

/// Vault header containing all metadata needed for decryption
#[derive(Debug)]
struct VaultHeader {
    magic: [u8; 8],
    version: u32,
    salt: [u8; SALT_SIZE],
    nonce: [u8; NONCE_SIZE],
    argon2_memory_kb: u32,
    argon2_iterations: u32,
    argon2_parallelism: u32,
    reserved: u32,
    encrypted_data_length: u64,
}

impl VaultHeader {
    fn new(salt: [u8; SALT_SIZE], nonce: [u8; NONCE_SIZE], data_len: u64) -> Self {
        Self {
            magic: *MAGIC_NUMBER,
            version: VERSION,
            salt,
            nonce,
            argon2_memory_kb: ARGON2_MEMORY_KB,
            argon2_iterations: ARGON2_ITERATIONS,
            argon2_parallelism: ARGON2_PARALLELISM,
            reserved: 0,
            encrypted_data_length: data_len,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.salt);
        bytes.extend_from_slice(&self.nonce);
        bytes.extend_from_slice(&self.argon2_memory_kb.to_le_bytes());
        bytes.extend_from_slice(&self.argon2_iterations.to_le_bytes());
        bytes.extend_from_slice(&self.argon2_parallelism.to_le_bytes());
        bytes.extend_from_slice(&self.reserved.to_le_bytes());
        bytes.extend_from_slice(&self.encrypted_data_length.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 64 {
            return Err(anyhow!("Invalid vault header: too short"));
        }

        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);

        if &magic != MAGIC_NUMBER {
            return Err(anyhow!("Invalid vault file: magic number mismatch"));
        }

        let version = u32::from_le_bytes(bytes[8..12].try_into()?);
        if version != VERSION {
            return Err(anyhow!(
                "Unsupported vault version: {} (expected {})",
                version,
                VERSION
            ));
        }

        let mut salt = [0u8; SALT_SIZE];
        salt.copy_from_slice(&bytes[12..28]);

        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&bytes[28..40]);

        let argon2_memory_kb = u32::from_le_bytes(bytes[40..44].try_into()?);
        let argon2_iterations = u32::from_le_bytes(bytes[44..48].try_into()?);
        let argon2_parallelism = u32::from_le_bytes(bytes[48..52].try_into()?);
        let reserved = u32::from_le_bytes(bytes[52..56].try_into()?);
        let encrypted_data_length = u64::from_le_bytes(bytes[56..64].try_into()?);

        Ok(Self {
            magic,
            version,
            salt,
            nonce,
            argon2_memory_kb,
            argon2_iterations,
            argon2_parallelism,
            reserved,
            encrypted_data_length,
        })
    }
}

/// Encrypt a vault with the given password
pub fn encrypt_vault(vault: &Vault, password: &str) -> Result<Vec<u8>> {
    // Generate random salt and nonce
    let mut salt = [0u8; SALT_SIZE];
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce_bytes);

    // Derive master key from password
    let master_key = MasterKey::derive(password, &salt)?;

    // Serialize vault to JSON
    let plaintext = serde_json::to_vec(vault).context("Failed to serialize vault")?;

    // Encrypt with ChaCha20Poly1305
    let cipher = ChaCha20Poly1305::new(master_key.as_bytes().into());
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    // Create header
    let header = VaultHeader::new(salt, nonce_bytes, ciphertext.len() as u64);

    // Combine header and ciphertext
    let mut output = header.to_bytes();
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypt a vault with the given password
pub fn decrypt_vault(data: &[u8], password: &str) -> Result<Vault> {
    // Parse header
    let header = VaultHeader::from_bytes(data).context("Failed to parse vault header")?;

    // Verify we have enough data
    let expected_total_len = 64 + header.encrypted_data_length as usize;
    if data.len() != expected_total_len {
        return Err(anyhow!(
            "Invalid vault file: expected {} bytes, got {}",
            expected_total_len,
            data.len()
        ));
    }

    // Derive master key from password
    let master_key = MasterKey::derive(password, &header.salt)?;

    // Extract ciphertext
    let ciphertext = &data[64..];

    // Decrypt with ChaCha20Poly1305
    let cipher = ChaCha20Poly1305::new(master_key.as_bytes().into());
    let nonce = Nonce::from_slice(&header.nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow!("Decryption failed: incorrect password or corrupted vault"))?;

    // Deserialize vault from JSON
    let vault: Vault =
        serde_json::from_slice(&plaintext).context("Failed to deserialize vault")?;

    Ok(vault)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_empty_vault() {
        let vault = Vault::new();
        let password = "test-password-123";

        let encrypted = encrypt_vault(&vault, password).unwrap();
        let decrypted = decrypt_vault(&encrypted, password).unwrap();

        assert_eq!(decrypted.credentials.len(), 0);
    }

    #[test]
    fn test_encrypt_decrypt_with_credentials() {
        let mut vault = Vault::new();
        vault.credentials.push(Credential {
            title: "Test Account".to_string(),
            username: "user@example.com".to_string(),
            password: "secret123".to_string(),
            url: "https://example.com".to_string(),
            notes: "Test notes".to_string(),
        });

        let password = "test-password-123";

        let encrypted = encrypt_vault(&vault, password).unwrap();
        let decrypted = decrypt_vault(&encrypted, password).unwrap();

        assert_eq!(decrypted.credentials.len(), 1);
        assert_eq!(decrypted.credentials[0].title, "Test Account");
        assert_eq!(decrypted.credentials[0].username, "user@example.com");
        assert_eq!(decrypted.credentials[0].password, "secret123");
    }

    #[test]
    fn test_wrong_password_fails() {
        let vault = Vault::new();
        let password = "correct-password";

        let encrypted = encrypt_vault(&vault, password).unwrap();
        let result = decrypt_vault(&encrypted, "wrong-password");

        assert!(result.is_err());
    }

    #[test]
    fn test_corrupted_data_fails() {
        let vault = Vault::new();
        let password = "test-password";

        let mut encrypted = encrypt_vault(&vault, password).unwrap();
        
        // Corrupt the ciphertext
        if encrypted.len() > 70 {
            encrypted[70] ^= 0xFF;
        }

        let result = decrypt_vault(&encrypted, password);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_passwords_produce_different_ciphertext() {
        let vault = Vault::new();

        let encrypted1 = encrypt_vault(&vault, "password1").unwrap();
        let encrypted2 = encrypt_vault(&vault, "password2").unwrap();

        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_same_password_produces_different_ciphertext() {
        // Due to random salt and nonce
        let vault = Vault::new();
        let password = "test-password";

        let encrypted1 = encrypt_vault(&vault, password).unwrap();
        let encrypted2 = encrypt_vault(&vault, password).unwrap();

        assert_ne!(encrypted1, encrypted2);
    }
}
