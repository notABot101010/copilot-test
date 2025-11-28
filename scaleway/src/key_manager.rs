//! Key Manager API
//!
//! Scaleway's Key Manager allows you to create, manage and use cryptographic keys
//! in a centralized and secure service.

use crate::client::{check_api_error, Client, Error};
use reqwest::Url;
use serde::{Deserialize, Serialize};

const KEY_MANAGER_API_URL: &str = "https://api.scaleway.com/key-manager/v1alpha1";

// ============================================================================
// Types
// ============================================================================

/// Key usage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyUsage {
    /// Symmetric encryption algorithm
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symmetric_encryption: Option<String>,
    /// Asymmetric encryption algorithm
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asymmetric_encryption: Option<String>,
    /// Asymmetric signing algorithm
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asymmetric_signing: Option<String>,
}

/// Key rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationPolicy {
    /// Rotation period in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_period: Option<String>,
    /// Next scheduled rotation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_rotation_at: Option<String>,
}

/// Cryptographic key
#[derive(Debug, Clone, Deserialize)]
pub struct Key {
    /// Key ID
    pub id: String,
    /// Project ID
    pub project_id: String,
    /// Key name
    pub name: String,
    /// Key usage configuration
    pub usage: KeyUsage,
    /// Key state
    pub state: String,
    /// Number of key rotations
    pub rotation_count: u32,
    /// Key creation date
    pub created_at: Option<String>,
    /// Key last modification date
    pub updated_at: Option<String>,
    /// Whether key protection is applied
    pub protected: bool,
    /// Whether the key is locked
    pub locked: bool,
    /// Description of the key
    pub description: Option<String>,
    /// List of tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Key last rotation date
    pub rotated_at: Option<String>,
    /// Key rotation policy
    pub rotation_policy: Option<KeyRotationPolicy>,
    /// Key origin
    pub origin: String,
    /// Deletion request timestamp
    pub deletion_requested_at: Option<String>,
    /// Region where the key is stored
    pub region: String,
}

/// Request to create a new key
#[derive(Debug, Clone, Serialize)]
pub struct CreateKeyRequest {
    /// Project ID
    pub project_id: String,
    /// Key name
    pub name: String,
    /// Key usage
    pub usage: KeyUsage,
    /// Key description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Rotation policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_policy: Option<KeyRotationPolicy>,
    /// Whether the key is unprotected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unprotected: Option<bool>,
    /// Key origin
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// Request to update a key
#[derive(Debug, Clone, Serialize)]
pub struct UpdateKeyRequest {
    /// Key name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Key description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Rotation policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_policy: Option<KeyRotationPolicy>,
}

/// Response for list keys
#[derive(Debug, Clone, Deserialize)]
pub struct ListKeysResponse {
    /// List of keys
    pub keys: Vec<Key>,
    /// Total count
    pub total_count: u64,
}

/// Algorithm information
#[derive(Debug, Clone, Deserialize)]
pub struct Algorithm {
    /// Algorithm usage
    pub usage: String,
    /// Algorithm name
    pub name: String,
    /// Whether this algorithm is recommended
    pub recommended: bool,
}

/// Response for list algorithms
#[derive(Debug, Clone, Deserialize)]
pub struct ListAlgorithmsResponse {
    /// List of algorithms
    pub algorithms: Vec<Algorithm>,
}

/// Data encryption key
#[derive(Debug, Clone, Deserialize)]
pub struct DataKey {
    /// Key ID
    pub key_id: String,
    /// Algorithm
    pub algorithm: String,
    /// Ciphertext
    pub ciphertext: String,
    /// Plaintext (optional)
    pub plaintext: Option<serde_json::Value>,
    /// Creation date
    pub created_at: Option<String>,
}

/// Request to generate a data key
#[derive(Debug, Clone, Serialize)]
pub struct GenerateDataKeyRequest {
    /// Algorithm to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    /// Whether to exclude plaintext from response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub without_plaintext: Option<bool>,
}

/// Request to encrypt data
#[derive(Debug, Clone, Serialize)]
pub struct EncryptRequest {
    /// Plaintext to encrypt (base64 encoded)
    pub plaintext: String,
    /// Associated data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_data: Option<String>,
}

/// Response for encrypt
#[derive(Debug, Clone, Deserialize)]
pub struct EncryptResponse {
    /// Key ID
    pub key_id: String,
    /// Ciphertext
    pub ciphertext: String,
}

/// Request to decrypt data
#[derive(Debug, Clone, Serialize)]
pub struct DecryptRequest {
    /// Ciphertext to decrypt
    pub ciphertext: String,
    /// Associated data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_data: Option<String>,
}

/// Response for decrypt
#[derive(Debug, Clone, Deserialize)]
pub struct DecryptResponse {
    /// Key ID
    pub key_id: String,
    /// Plaintext
    pub plaintext: String,
}

/// Public key in PEM format
#[derive(Debug, Clone, Deserialize)]
pub struct PublicKey {
    /// PEM-encoded public key
    pub pem: String,
}

/// Request to sign a message
#[derive(Debug, Clone, Serialize)]
pub struct SignRequest {
    /// Message digest to sign
    pub digest: String,
}

/// Response for sign
#[derive(Debug, Clone, Deserialize)]
pub struct SignResponse {
    /// Key ID
    pub key_id: String,
    /// Signature
    pub signature: String,
}

/// Request to verify a signature
#[derive(Debug, Clone, Serialize)]
pub struct VerifyRequest {
    /// Message digest
    pub digest: String,
    /// Signature to verify
    pub signature: String,
}

/// Response for verify
#[derive(Debug, Clone, Deserialize)]
pub struct VerifyResponse {
    /// Key ID
    pub key_id: String,
    /// Whether the signature is valid
    pub valid: bool,
}

/// Request to import key material
#[derive(Debug, Clone, Serialize)]
pub struct ImportKeyMaterialRequest {
    /// Key material (base64 encoded)
    pub key_material: String,
    /// Salt value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<serde_json::Value>,
}

// ============================================================================
// Key Manager API Implementation
// ============================================================================

impl Client {
    // ========================================================================
    // Algorithms
    // ========================================================================

    /// List all available algorithms
    pub async fn list_algorithms(
        &self,
        region: &str,
        usages: Option<&[&str]>,
    ) -> Result<ListAlgorithmsResponse, Error> {
        let mut url = Url::parse(&format!(
            "{}/regions/{}/algorithms",
            KEY_MANAGER_API_URL, region
        ))
        .expect("valid URL");

        if let Some(usages) = usages {
            for usage in usages {
                url.query_pairs_mut().append_pair("usages", usage);
            }
        }

        let res = self
            .http_client
            .get(url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListAlgorithmsResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    // ========================================================================
    // Keys - CRUD Operations
    // ========================================================================

    /// List keys
    pub async fn list_keys(
        &self,
        region: &str,
        project_id: Option<&str>,
        organization_id: Option<&str>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<ListKeysResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/regions/{}/keys", KEY_MANAGER_API_URL, region))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = project_id {
                pairs.append_pair("project_id", v);
            }
            if let Some(v) = organization_id {
                pairs.append_pair("organization_id", v);
            }
            if let Some(v) = page {
                pairs.append_pair("page", &v.to_string());
            }
            if let Some(v) = page_size {
                pairs.append_pair("page_size", &v.to_string());
            }
        }

        let res = self
            .http_client
            .get(url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListKeysResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create a new key
    pub async fn create_key(
        &self,
        region: &str,
        request: CreateKeyRequest,
    ) -> Result<Key, Error> {
        let url = format!("{}/regions/{}/keys", KEY_MANAGER_API_URL, region);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let key: Key = res.json().await.map_err(Error::Json)?;
        Ok(key)
    }

    /// Get a key by ID
    pub async fn get_key(&self, region: &str, key_id: &str) -> Result<Key, Error> {
        let url = format!("{}/regions/{}/keys/{}", KEY_MANAGER_API_URL, region, key_id);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let key: Key = res.json().await.map_err(Error::Json)?;
        Ok(key)
    }

    /// Update a key
    pub async fn update_key(
        &self,
        region: &str,
        key_id: &str,
        request: UpdateKeyRequest,
    ) -> Result<Key, Error> {
        let url = format!("{}/regions/{}/keys/{}", KEY_MANAGER_API_URL, region, key_id);

        let res = self
            .http_client
            .patch(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let key: Key = res.json().await.map_err(Error::Json)?;
        Ok(key)
    }

    /// Delete a key
    pub async fn delete_key(&self, region: &str, key_id: &str) -> Result<(), Error> {
        let url = format!("{}/regions/{}/keys/{}", KEY_MANAGER_API_URL, region, key_id);

        let res = self
            .http_client
            .delete(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        check_api_error(res).await?;
        Ok(())
    }

    // ========================================================================
    // Keys - State Management
    // ========================================================================

    /// Enable a key
    pub async fn enable_key(&self, region: &str, key_id: &str) -> Result<Key, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/enable",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let key: Key = res.json().await.map_err(Error::Json)?;
        Ok(key)
    }

    /// Disable a key
    pub async fn disable_key(&self, region: &str, key_id: &str) -> Result<Key, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/disable",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let key: Key = res.json().await.map_err(Error::Json)?;
        Ok(key)
    }

    /// Protect a key (prevents deletion)
    pub async fn protect_key(&self, region: &str, key_id: &str) -> Result<Key, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/protect",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let key: Key = res.json().await.map_err(Error::Json)?;
        Ok(key)
    }

    /// Unprotect a key (allows deletion)
    pub async fn unprotect_key(&self, region: &str, key_id: &str) -> Result<Key, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/unprotect",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let key: Key = res.json().await.map_err(Error::Json)?;
        Ok(key)
    }

    /// Rotate a key (generate new key material)
    pub async fn rotate_key(&self, region: &str, key_id: &str) -> Result<Key, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/rotate",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let key: Key = res.json().await.map_err(Error::Json)?;
        Ok(key)
    }

    /// Restore a key scheduled for deletion
    pub async fn restore_key(&self, region: &str, key_id: &str) -> Result<Key, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/restore",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let key: Key = res.json().await.map_err(Error::Json)?;
        Ok(key)
    }

    /// Delete key material (for external origin keys)
    pub async fn delete_key_material(&self, region: &str, key_id: &str) -> Result<Key, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/delete-key-material",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let key: Key = res.json().await.map_err(Error::Json)?;
        Ok(key)
    }

    /// Import key material for external origin keys
    pub async fn import_key_material(
        &self,
        region: &str,
        key_id: &str,
        request: ImportKeyMaterialRequest,
    ) -> Result<Key, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/import-key-material",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let key: Key = res.json().await.map_err(Error::Json)?;
        Ok(key)
    }

    // ========================================================================
    // Cryptographic Operations
    // ========================================================================

    /// Encrypt data using a key
    pub async fn encrypt(
        &self,
        region: &str,
        key_id: &str,
        request: EncryptRequest,
    ) -> Result<EncryptResponse, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/encrypt",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: EncryptResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Decrypt data using a key
    pub async fn decrypt(
        &self,
        region: &str,
        key_id: &str,
        request: DecryptRequest,
    ) -> Result<DecryptResponse, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/decrypt",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: DecryptResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Generate a data encryption key
    pub async fn generate_data_key(
        &self,
        region: &str,
        key_id: &str,
        request: GenerateDataKeyRequest,
    ) -> Result<DataKey, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/generate-data-key",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let data_key: DataKey = res.json().await.map_err(Error::Json)?;
        Ok(data_key)
    }

    /// Get the public key in PEM format (for asymmetric keys)
    pub async fn get_public_key(&self, region: &str, key_id: &str) -> Result<PublicKey, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/public-key",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let public_key: PublicKey = res.json().await.map_err(Error::Json)?;
        Ok(public_key)
    }

    /// Sign a message digest
    pub async fn sign(
        &self,
        region: &str,
        key_id: &str,
        request: SignRequest,
    ) -> Result<SignResponse, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/sign",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: SignResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Verify a message signature
    pub async fn verify(
        &self,
        region: &str,
        key_id: &str,
        request: VerifyRequest,
    ) -> Result<VerifyResponse, Error> {
        let url = format!(
            "{}/regions/{}/keys/{}/verify",
            KEY_MANAGER_API_URL, region, key_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: VerifyResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }
}
