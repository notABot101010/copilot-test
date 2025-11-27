//! SSH Keys API endpoints.
//!
//! Manage SSH keys in your DigitalOcean account.
//! See: <https://docs.digitalocean.com/reference/api/api-reference/#tag/SSH-Keys>

use serde::{Deserialize, Serialize};

use crate::{check_api_error, Client, Error, Links, Meta, API_BASE_URL};

/// An SSH key stored in your DigitalOcean account.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SshKey {
    /// Unique identifier for the key.
    pub id: u64,
    /// Human-readable name for the key.
    pub name: String,
    /// MD5 fingerprint of the key.
    pub fingerprint: String,
    /// Full public key content.
    pub public_key: String,
}

/// Request to create a new SSH key.
#[derive(Debug, Clone, Serialize)]
pub struct CreateSshKeyRequest {
    /// Human-readable name for the key.
    pub name: String,
    /// Full public key content.
    pub public_key: String,
}

/// Request to update an SSH key.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateSshKeyRequest {
    /// New name for the key.
    pub name: String,
}

/// Response from listing SSH keys.
#[derive(Debug, Clone, Deserialize)]
pub struct ListSshKeysResponse {
    /// List of SSH keys.
    pub ssh_keys: Vec<SshKey>,
    /// Pagination links.
    pub links: Option<Links>,
    /// Metadata about the response.
    pub meta: Option<Meta>,
}

/// Response from getting or creating a single SSH key.
#[derive(Debug, Clone, Deserialize)]
pub struct SshKeyResponse {
    /// The SSH key.
    pub ssh_key: SshKey,
}

impl Client {
    /// Lists all SSH keys in your account.
    ///
    /// # Arguments
    ///
    /// * `page` - Page number for pagination.
    /// * `per_page` - Number of items per page.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::Client;
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let response = client.list_ssh_keys(None, None).await?;
    /// for key in response.ssh_keys {
    ///     println!("{}: {}", key.id, key.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_ssh_keys(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<ListSshKeysResponse, Error> {
        let mut url = format!("{}/account/keys", API_BASE_URL);
        let mut query_params = Vec::new();

        if let Some(p) = page {
            query_params.push(format!("page={}", p));
        }
        if let Some(pp) = per_page {
            query_params.push(format!("per_page={}", pp));
        }
        if !query_params.is_empty() {
            url = format!("{}?{}", url, query_params.join("&"));
        }

        let res = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Gets an SSH key by ID.
    ///
    /// # Arguments
    ///
    /// * `key_id` - The ID of the SSH key.
    pub async fn get_ssh_key_by_id(&self, key_id: u64) -> Result<SshKeyResponse, Error> {
        let url = format!("{}/account/keys/{}", API_BASE_URL, key_id);

        let res = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Gets an SSH key by fingerprint.
    ///
    /// # Arguments
    ///
    /// * `fingerprint` - The MD5 fingerprint of the SSH key.
    pub async fn get_ssh_key_by_fingerprint(
        &self,
        fingerprint: &str,
    ) -> Result<SshKeyResponse, Error> {
        let url = format!("{}/account/keys/{}", API_BASE_URL, fingerprint);

        let res = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Creates a new SSH key.
    ///
    /// # Arguments
    ///
    /// * `request` - The SSH key creation request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::{Client, CreateSshKeyRequest};
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let request = CreateSshKeyRequest {
    ///     name: "my-key".to_string(),
    ///     public_key: "ssh-rsa AAAA...".to_string(),
    /// };
    /// let response = client.create_ssh_key(request).await?;
    /// println!("Created key: {}", response.ssh_key.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_ssh_key(
        &self,
        request: CreateSshKeyRequest,
    ) -> Result<SshKeyResponse, Error> {
        let url = format!("{}/account/keys", API_BASE_URL);

        let res = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&request)
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Updates an SSH key by ID.
    ///
    /// # Arguments
    ///
    /// * `key_id` - The ID of the SSH key.
    /// * `request` - The update request.
    pub async fn update_ssh_key_by_id(
        &self,
        key_id: u64,
        request: UpdateSshKeyRequest,
    ) -> Result<SshKeyResponse, Error> {
        let url = format!("{}/account/keys/{}", API_BASE_URL, key_id);

        let res = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&request)
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Deletes an SSH key by ID.
    ///
    /// # Arguments
    ///
    /// * `key_id` - The ID of the SSH key to delete.
    pub async fn delete_ssh_key_by_id(&self, key_id: u64) -> Result<(), Error> {
        let url = format!("{}/account/keys/{}", API_BASE_URL, key_id);

        let res = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        check_api_error(res).await?;
        Ok(())
    }

    /// Deletes an SSH key by fingerprint.
    ///
    /// # Arguments
    ///
    /// * `fingerprint` - The MD5 fingerprint of the SSH key to delete.
    pub async fn delete_ssh_key_by_fingerprint(&self, fingerprint: &str) -> Result<(), Error> {
        let url = format!("{}/account/keys/{}", API_BASE_URL, fingerprint);

        let res = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        check_api_error(res).await?;
        Ok(())
    }
}
