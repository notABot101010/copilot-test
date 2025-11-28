//! Block Storage Volumes API endpoints.
//!
//! Manage block storage volumes in DigitalOcean.
//! See: <https://docs.digitalocean.com/reference/api/api-reference/#tag/Block-Storage>

use serde::{Deserialize, Serialize};

use crate::{check_api_error, Client, Error, Links, Meta, Url, API_BASE_URL};

/// A block storage volume.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Volume {
    /// Unique identifier for the volume.
    pub id: String,
    /// Human-readable name for the volume.
    pub name: String,
    /// Description of the volume.
    pub description: String,
    /// Size of the volume in GiB.
    pub size_gigabytes: u64,
    /// When the volume was created.
    pub created_at: String,
    /// The region the volume is in.
    pub region: VolumeRegion,
    /// IDs of Droplets the volume is attached to.
    pub droplet_ids: Vec<u64>,
    /// Tags applied to the volume.
    pub tags: Vec<String>,
    /// Filesystem type (e.g., "ext4").
    pub filesystem_type: Option<String>,
    /// Filesystem label.
    pub filesystem_label: Option<String>,
}

/// Region information for a volume.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VolumeRegion {
    /// Region slug.
    pub slug: String,
    /// Human-readable region name.
    pub name: String,
    /// Available sizes in this region.
    pub sizes: Vec<String>,
    /// Whether the region is available.
    pub available: bool,
    /// Features available in this region.
    pub features: Vec<String>,
}

/// Request to create a new volume.
#[derive(Debug, Clone, Serialize)]
pub struct CreateVolumeRequest {
    /// Size of the volume in GiB (minimum 1).
    pub size_gigabytes: u64,
    /// Human-readable name for the volume.
    pub name: String,
    /// Description of the volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Region slug where the volume will be created.
    pub region: String,
    /// Filesystem type (e.g., "ext4", "xfs").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesystem_type: Option<String>,
    /// Filesystem label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesystem_label: Option<String>,
    /// Tags to apply to the volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Snapshot ID to create the volume from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
}

/// Response from listing volumes.
#[derive(Debug, Clone, Deserialize)]
pub struct ListVolumesResponse {
    /// List of volumes.
    pub volumes: Vec<Volume>,
    /// Pagination links.
    pub links: Option<Links>,
    /// Metadata about the response.
    pub meta: Option<Meta>,
}

/// Response from getting or creating a single volume.
#[derive(Debug, Clone, Deserialize)]
pub struct VolumeResponse {
    /// The volume.
    pub volume: Volume,
}

/// Request to attach a volume to a Droplet.
#[derive(Debug, Clone, Serialize)]
pub struct AttachVolumeRequest {
    /// Type of action.
    #[serde(rename = "type")]
    pub action_type: String,
    /// ID of the Droplet to attach to.
    pub droplet_id: u64,
    /// Region slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// Action performed on a volume.
#[derive(Debug, Clone, Deserialize)]
pub struct VolumeAction {
    /// Unique identifier for the action.
    pub id: u64,
    /// Current status of the action.
    pub status: String,
    /// Type of action.
    #[serde(rename = "type")]
    pub action_type: String,
    /// When the action started.
    pub started_at: String,
    /// When the action completed.
    pub completed_at: Option<String>,
    /// Resource ID the action is on.
    pub resource_id: Option<u64>,
    /// Resource type.
    pub resource_type: String,
    /// Region slug.
    pub region_slug: Option<String>,
}

/// Response from a volume action.
#[derive(Debug, Clone, Deserialize)]
pub struct VolumeActionResponse {
    /// The action.
    pub action: VolumeAction,
}

impl Client {
    /// Lists all volumes in your account.
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
    /// let response = client.list_volumes(None, None, None).await?;
    /// for volume in response.volumes {
    ///     println!("{}: {} GiB", volume.name, volume.size_gigabytes);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_volumes(
        &self,
        name: Option<&str>,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<ListVolumesResponse, Error> {
        let mut url = Url::parse(&format!("{}/volumes", API_BASE_URL)).expect("Invalid URL");

        {
            let mut query = url.query_pairs_mut();
            if let Some(n) = name {
                query.append_pair("name", n);
            }
            if let Some(p) = page {
                query.append_pair("page", &p.to_string());
            }
            if let Some(pp) = per_page {
                query.append_pair("per_page", &pp.to_string());
            }
        }

        let res = self
            .http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Gets a specific volume by ID.
    ///
    /// # Arguments
    ///
    /// * `volume_id` - The ID of the volume.
    pub async fn get_volume(&self, volume_id: &str) -> Result<VolumeResponse, Error> {
        let url = Url::parse(&format!("{}/volumes/{}", API_BASE_URL, volume_id)).expect("Invalid URL");

        let res = self
            .http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Creates a new volume.
    ///
    /// # Arguments
    ///
    /// * `request` - The volume creation request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::{Client, CreateVolumeRequest};
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let request = CreateVolumeRequest {
    ///     size_gigabytes: 100,
    ///     name: "my-volume".to_string(),
    ///     description: Some("A test volume".to_string()),
    ///     region: "nyc1".to_string(),
    ///     filesystem_type: Some("ext4".to_string()),
    ///     filesystem_label: Some("my-volume".to_string()),
    ///     tags: Some(vec!["database".to_string()]),
    ///     snapshot_id: None,
    /// };
    /// let response = client.create_volume(request).await?;
    /// println!("Created volume: {}", response.volume.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_volume(&self, request: CreateVolumeRequest) -> Result<VolumeResponse, Error> {
        let url = Url::parse(&format!("{}/volumes", API_BASE_URL)).expect("Invalid URL");

        let res = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&request)
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Deletes a volume by ID.
    ///
    /// # Arguments
    ///
    /// * `volume_id` - The ID of the volume to delete.
    pub async fn delete_volume(&self, volume_id: &str) -> Result<(), Error> {
        let url = Url::parse(&format!("{}/volumes/{}", API_BASE_URL, volume_id)).expect("Invalid URL");

        let res = self
            .http_client
            .delete(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        check_api_error(res).await?;
        Ok(())
    }

    /// Attaches a volume to a Droplet.
    ///
    /// # Arguments
    ///
    /// * `volume_id` - The ID of the volume.
    /// * `droplet_id` - The ID of the Droplet to attach to.
    /// * `region` - Optional region slug.
    pub async fn attach_volume(
        &self,
        volume_id: &str,
        droplet_id: u64,
        region: Option<&str>,
    ) -> Result<VolumeActionResponse, Error> {
        let url = Url::parse(&format!("{}/volumes/{}/actions", API_BASE_URL, volume_id)).expect("Invalid URL");

        let request = AttachVolumeRequest {
            action_type: "attach".to_string(),
            droplet_id,
            region: region.map(|s| s.to_string()),
        };

        let res = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&request)
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Detaches a volume from a Droplet.
    ///
    /// # Arguments
    ///
    /// * `volume_id` - The ID of the volume.
    /// * `droplet_id` - The ID of the Droplet to detach from.
    /// * `region` - Optional region slug.
    pub async fn detach_volume(
        &self,
        volume_id: &str,
        droplet_id: u64,
        region: Option<&str>,
    ) -> Result<VolumeActionResponse, Error> {
        let url = Url::parse(&format!("{}/volumes/{}/actions", API_BASE_URL, volume_id)).expect("Invalid URL");

        let request = AttachVolumeRequest {
            action_type: "detach".to_string(),
            droplet_id,
            region: region.map(|s| s.to_string()),
        };

        let res = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&request)
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }
}
