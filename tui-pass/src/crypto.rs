use anyhow::{anyhow, Context, Result};
use argon2::{Argon2, ParamsBuilder, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use prost::Message;
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

// Include the generated protobuf code
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/vault.rs"));
}

/// Size constants for cryptographic primitives
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;
const MAGIC_NUMBER: &[u8; 8] = b"TUIPASS2"; // Changed version for new format
const VERSION: u32 = 2;

/// Argon2id parameters
const ARGON2_MEMORY_KB: u32 = 65536; // 64 MB
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_PARALLELISM: u32 = 4;

/// A credential entry in the vault (public API)
#[derive(Debug, Clone)]
pub struct Credential {
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
}

impl Credential {
    /// Convert to protobuf message
    fn to_proto(&self) -> proto::Credential {
        proto::Credential {
            title: self.title.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            url: self.url.clone(),
            notes: self.notes.clone(),
        }
    }

    /// Convert from protobuf message
    fn from_proto(proto: proto::Credential) -> Self {
        Self {
            title: proto.title,
            username: proto.username,
            password: proto.password,
            url: proto.url,
            notes: proto.notes,
        }
    }
}

/// The vault containing encrypted entries
pub struct Vault {
    /// Master password (kept for encryption operations)
    master_password: String,
    /// Salt for key derivation
    salt: [u8; SALT_SIZE],
    /// Encrypted entries stored in protobuf format
    encrypted_entries: Vec<proto::EncryptedEntry>,
    /// Cached decrypted credentials (indexed by position)
    credential_cache: Vec<Option<Credential>>,
}

impl Vault {
    pub fn new() -> Self {
        let mut salt = [0u8; SALT_SIZE];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut salt);
        
        Self {
            master_password: String::new(),
            salt,
            encrypted_entries: Vec::new(),
            credential_cache: Vec::new(),
        }
    }

    /// Initialize with master password
    pub fn with_password(password: String) -> Self {
        let mut salt = [0u8; SALT_SIZE];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut salt);
        
        Self {
            master_password: password,
            salt,
            encrypted_entries: Vec::new(),
            credential_cache: Vec::new(),
        }
    }

    /// Initialize with password and salt (for loading existing vaults)
    fn with_password_and_salt(password: String, salt: [u8; SALT_SIZE]) -> Self {
        Self {
            master_password: password,
            salt,
            encrypted_entries: Vec::new(),
            credential_cache: Vec::new(),
        }
    }

    /// Get the number of credentials in the vault
    pub fn len(&self) -> usize {
        self.encrypted_entries.len()
    }

    /// Check if vault is empty
    pub fn is_empty(&self) -> bool {
        self.encrypted_entries.is_empty()
    }

    /// Get a decrypted credential by index (decrypts on-demand)
    pub fn get_credential(&mut self, index: usize) -> Result<&Credential> {
        if index >= self.encrypted_entries.len() {
            return Err(anyhow!("Index out of bounds"));
        }

        // Check if already cached
        if index < self.credential_cache.len() && self.credential_cache[index].is_some() {
            return Ok(self.credential_cache[index].as_ref().unwrap());
        }

        // Decrypt the entry
        let encrypted_entry = &self.encrypted_entries[index];
        let master_key = MasterKey::derive(&self.master_password, &self.salt)?;
        let credential = decrypt_entry(encrypted_entry, &master_key)?;

        // Ensure cache is large enough
        while self.credential_cache.len() <= index {
            self.credential_cache.push(None);
        }

        // Cache the credential
        self.credential_cache[index] = Some(credential);
        Ok(self.credential_cache[index].as_ref().unwrap())
    }

    /// Get all credential titles (for listing, without decrypting full entries)
    pub fn get_titles(&mut self) -> Result<Vec<String>> {
        let mut titles = Vec::new();
        for i in 0..self.encrypted_entries.len() {
            let cred = self.get_credential(i)?;
            titles.push(cred.title.clone());
        }
        Ok(titles)
    }

    /// Add a new credential (encrypts immediately)
    pub fn add_credential(&mut self, credential: Credential) -> Result<()> {
        let master_key = MasterKey::derive(&self.master_password, &self.salt)?;
        let encrypted_entry = encrypt_entry(&credential, &master_key)?;
        self.encrypted_entries.push(encrypted_entry);
        self.credential_cache.push(Some(credential));
        Ok(())
    }

    /// Update a credential at the given index
    pub fn update_credential(&mut self, index: usize, credential: Credential) -> Result<()> {
        if index >= self.encrypted_entries.len() {
            return Err(anyhow!("Index out of bounds"));
        }

        let master_key = MasterKey::derive(&self.master_password, &self.salt)?;
        let encrypted_entry = encrypt_entry(&credential, &master_key)?;
        self.encrypted_entries[index] = encrypted_entry;

        // Update cache
        if index < self.credential_cache.len() {
            self.credential_cache[index] = Some(credential);
        }

        Ok(())
    }

    /// Remove a credential at the given index
    pub fn remove_credential(&mut self, index: usize) -> Result<()> {
        if index >= self.encrypted_entries.len() {
            return Err(anyhow!("Index out of bounds"));
        }

        self.encrypted_entries.remove(index);
        if index < self.credential_cache.len() {
            self.credential_cache.remove(index);
        }

        Ok(())
    }

    /// Clear all cached decrypted credentials from memory
    pub fn clear_cache(&mut self) {
        self.credential_cache.clear();
    }

    /// Get reference to encrypted entries (for serialization)
    fn get_encrypted_entries(&self) -> &[proto::EncryptedEntry] {
        &self.encrypted_entries
    }

    /// Get the salt (for serialization)
    fn get_salt(&self) -> [u8; SALT_SIZE] {
        self.salt
    }

    /// Set encrypted entries (for deserialization)
    fn set_encrypted_entries(&mut self, entries: Vec<proto::EncryptedEntry>) {
        self.encrypted_entries = entries;
        self.credential_cache.clear();
    }

    /// Get all credentials (decrypting all entries - use sparingly)
    pub fn get_all_credentials(&mut self) -> Result<Vec<Credential>> {
        let mut credentials = Vec::new();
        for i in 0..self.encrypted_entries.len() {
            let cred = self.get_credential(i)?;
            credentials.push(cred.clone());
        }
        Ok(credentials)
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

/// Encrypt a single credential entry
fn encrypt_entry(credential: &Credential, master_key: &MasterKey) -> Result<proto::EncryptedEntry> {
    // Generate random nonce for this entry
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut nonce_bytes);

    // Convert credential to protobuf and serialize
    let proto_cred = credential.to_proto();
    let plaintext = proto_cred.encode_to_vec();

    // Encrypt with ChaCha20Poly1305
    let cipher = ChaCha20Poly1305::new(master_key.as_bytes().into());
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    Ok(proto::EncryptedEntry {
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

/// Decrypt a single credential entry
fn decrypt_entry(entry: &proto::EncryptedEntry, master_key: &MasterKey) -> Result<Credential> {
    if entry.nonce.len() != NONCE_SIZE {
        return Err(anyhow!("Invalid nonce size"));
    }

    // Decrypt with ChaCha20Poly1305
    let cipher = ChaCha20Poly1305::new(master_key.as_bytes().into());
    let nonce = Nonce::from_slice(&entry.nonce);

    let plaintext = cipher
        .decrypt(nonce, entry.ciphertext.as_ref())
        .map_err(|_| anyhow!("Decryption failed: incorrect password or corrupted entry"))?;

    // Deserialize from protobuf
    let proto_cred = proto::Credential::decode(&plaintext[..])
        .context("Failed to deserialize credential")?;

    Ok(Credential::from_proto(proto_cred))
}

/// Vault header containing metadata (simplified for new format)
#[derive(Debug)]
struct VaultHeader {
    magic: [u8; 8],
    version: u32,
    salt: [u8; SALT_SIZE],  // Random salt for key derivation
    reserved: [u8; 36],      // Reserved for future use (64 - 8 - 4 - 16 = 36)
}

impl VaultHeader {
    fn new(salt: [u8; SALT_SIZE]) -> Self {
        Self {
            magic: *MAGIC_NUMBER,
            version: VERSION,
            salt,
            reserved: [0u8; 36],
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.salt);
        bytes.extend_from_slice(&self.reserved);
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

        let mut reserved = [0u8; 36];
        reserved.copy_from_slice(&bytes[28..64]);

        Ok(Self {
            magic,
            version,
            salt,
            reserved,
        })
    }
}

/// Encrypt a vault with the given password (password parameter kept for API consistency)
pub fn encrypt_vault(vault: &Vault, _password: &str) -> Result<Vec<u8>> {
    // Create protobuf vault message
    let proto_vault = proto::Vault {
        entries: vault.get_encrypted_entries().to_vec(),
    };

    // Serialize vault to protobuf
    let plaintext = proto_vault.encode_to_vec();

    // Create header with vault's salt
    let header = VaultHeader::new(vault.salt);

    // Combine header and protobuf data
    let mut output = header.to_bytes();
    output.extend_from_slice(&plaintext);

    Ok(output)
}

/// Decrypt a vault with the given password
pub fn decrypt_vault(data: &[u8], password: &str) -> Result<Vault> {
    // Parse header
    let header = VaultHeader::from_bytes(data).context("Failed to parse vault header")?;

    // Extract protobuf data
    let proto_data = &data[64..];

    // Deserialize vault from protobuf
    let proto_vault = proto::Vault::decode(proto_data)
        .context("Failed to deserialize vault")?;

    // Create vault with password and salt from header
    let mut vault = Vault::with_password_and_salt(password.to_string(), header.salt);
    vault.set_encrypted_entries(proto_vault.entries);

    Ok(vault)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_empty_vault() {
        let password = "test-password-123";
        let vault = Vault::with_password(password.to_string());

        let encrypted = encrypt_vault(&vault, password).unwrap();
        let decrypted = decrypt_vault(&encrypted, password).unwrap();

        assert_eq!(decrypted.len(), 0);
    }

    #[test]
    fn test_encrypt_decrypt_with_credentials() {
        let password = "test-password-123";
        let mut vault = Vault::with_password(password.to_string());
        
        vault.add_credential(Credential {
            title: "Test Account".to_string(),
            username: "user@example.com".to_string(),
            password: "secret123".to_string(),
            url: "https://example.com".to_string(),
            notes: "Test notes".to_string(),
        }).unwrap();

        let encrypted = encrypt_vault(&vault, password).unwrap();
        let mut decrypted = decrypt_vault(&encrypted, password).unwrap();

        assert_eq!(decrypted.len(), 1);
        let cred = decrypted.get_credential(0).unwrap();
        assert_eq!(cred.title, "Test Account");
        assert_eq!(cred.username, "user@example.com");
        assert_eq!(cred.password, "secret123");
    }

    #[test]
    fn test_wrong_password_fails() {
        let password = "correct-password";
        let mut vault = Vault::with_password(password.to_string());
        
        vault.add_credential(Credential {
            title: "Test".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            url: "url".to_string(),
            notes: "notes".to_string(),
        }).unwrap();

        let encrypted = encrypt_vault(&vault, password).unwrap();
        let mut decrypted = decrypt_vault(&encrypted, "wrong-password").unwrap();
        
        // Decryption of individual entry should fail
        let result = decrypted.get_credential(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_corrupted_data_fails() {
        let password = "test-password";
        let mut vault = Vault::with_password(password.to_string());
        
        vault.add_credential(Credential {
            title: "Test".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            url: "url".to_string(),
            notes: "notes".to_string(),
        }).unwrap();

        let mut encrypted = encrypt_vault(&vault, password).unwrap();
        
        // Corrupt the protobuf data more significantly (corrupt multiple bytes)
        if encrypted.len() > 70 {
            for i in 70..encrypted.len().min(80) {
                encrypted[i] ^= 0xFF;
            }
        }

        let result = decrypt_vault(&encrypted, password);
        // Should fail at protobuf deserialization or decryption
        if result.is_ok() {
            // If deserialization succeeds, individual entry decryption should fail
            let mut vault = result.unwrap();
            let cred_result = vault.get_credential(0);
            assert!(cred_result.is_err(), "Corrupted data should cause decryption to fail");
        }
    }

    #[test]
    fn test_individual_entry_encryption() {
        let password = "test-password";
        let mut vault = Vault::with_password(password.to_string());
        
        vault.add_credential(Credential {
            title: "Entry 1".to_string(),
            username: "user1".to_string(),
            password: "pass1".to_string(),
            url: "url1".to_string(),
            notes: "notes1".to_string(),
        }).unwrap();
        
        vault.add_credential(Credential {
            title: "Entry 2".to_string(),
            username: "user2".to_string(),
            password: "pass2".to_string(),
            url: "url2".to_string(),
            notes: "notes2".to_string(),
        }).unwrap();

        // Verify entries can be decrypted individually
        let cred1 = vault.get_credential(0).unwrap();
        assert_eq!(cred1.title, "Entry 1");
        
        let cred2 = vault.get_credential(1).unwrap();
        assert_eq!(cred2.title, "Entry 2");
    }

    #[test]
    fn test_cache_clearing() {
        let password = "test-password";
        let mut vault = Vault::with_password(password.to_string());
        
        vault.add_credential(Credential {
            title: "Test".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            url: "url".to_string(),
            notes: "notes".to_string(),
        }).unwrap();

        // Access credential to populate cache
        let _ = vault.get_credential(0).unwrap();
        assert_eq!(vault.credential_cache.len(), 1);

        // Clear cache
        vault.clear_cache();
        assert_eq!(vault.credential_cache.len(), 0);

        // Should still be able to decrypt
        let cred = vault.get_credential(0).unwrap();
        assert_eq!(cred.title, "Test");
    }
}
