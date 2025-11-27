//! Droplets API endpoints.
//!
//! Droplets are DigitalOcean's virtual machines.
//! See: <https://docs.digitalocean.com/reference/api/api-reference/#tag/Droplets>

use serde::{Deserialize, Serialize};

use crate::{check_api_error, Client, Error, Links, Meta, Url, API_BASE_URL};

/// A DigitalOcean Droplet (virtual machine).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Droplet {
    /// Unique identifier for the Droplet.
    pub id: u64,
    /// Human-readable name for the Droplet.
    pub name: String,
    /// Memory size in megabytes.
    pub memory: u64,
    /// Number of virtual CPUs.
    pub vcpus: u32,
    /// Disk size in gigabytes.
    pub disk: u64,
    /// Whether the Droplet is locked.
    pub locked: bool,
    /// Current status of the Droplet.
    pub status: String,
    /// The region the Droplet is deployed in.
    pub region: Option<Region>,
    /// The base image used to create the Droplet.
    pub image: Option<Image>,
    /// The size configuration of the Droplet.
    pub size: Option<Size>,
    /// The unique slug identifier for the size.
    pub size_slug: String,
    /// Network configuration for the Droplet.
    pub networks: Option<Networks>,
    /// Tags applied to the Droplet.
    pub tags: Vec<String>,
    /// IDs of volumes attached to the Droplet.
    pub volume_ids: Vec<String>,
    /// The VPC UUID the Droplet is in.
    pub vpc_uuid: Option<String>,
    /// When the Droplet was created.
    pub created_at: String,
}

/// Region where resources are deployed.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Region {
    /// Unique slug for the region.
    pub slug: String,
    /// Human-readable name.
    pub name: String,
    /// Available sizes in this region.
    pub sizes: Vec<String>,
    /// Whether the region is available for new Droplets.
    pub available: bool,
    /// Features available in this region.
    pub features: Vec<String>,
}

/// Base image for a Droplet.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Image {
    /// Unique identifier.
    pub id: u64,
    /// Image name.
    pub name: String,
    /// Image type (e.g., "snapshot", "backup").
    #[serde(rename = "type")]
    pub image_type: Option<String>,
    /// Distribution name.
    pub distribution: Option<String>,
    /// Image slug identifier.
    pub slug: Option<String>,
    /// Whether the image is public.
    pub public: bool,
    /// Regions where the image is available.
    pub regions: Vec<String>,
    /// Minimum disk size required.
    pub min_disk_size: Option<u64>,
    /// Size of the image in gigabytes.
    pub size_gigabytes: Option<f64>,
    /// When the image was created.
    pub created_at: Option<String>,
    /// Image description.
    pub description: Option<String>,
    /// Tags applied to the image.
    pub tags: Option<Vec<String>>,
    /// Status of the image.
    pub status: Option<String>,
}

/// Size configuration for a Droplet.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Size {
    /// Unique slug identifier.
    pub slug: String,
    /// Memory in megabytes.
    pub memory: u64,
    /// Number of virtual CPUs.
    pub vcpus: u32,
    /// Disk size in gigabytes.
    pub disk: u64,
    /// Transfer limit in terabytes.
    pub transfer: f64,
    /// Monthly price in USD.
    pub price_monthly: f64,
    /// Hourly price in USD.
    pub price_hourly: f64,
    /// Regions where this size is available.
    pub regions: Vec<String>,
    /// Whether this size is available.
    pub available: bool,
    /// Human-readable description.
    pub description: String,
}

/// Network configuration for a Droplet.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Networks {
    /// IPv4 networks.
    pub v4: Vec<NetworkV4>,
    /// IPv6 networks.
    pub v6: Vec<NetworkV6>,
}

/// IPv4 network configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkV4 {
    /// IP address.
    pub ip_address: String,
    /// Network mask.
    pub netmask: String,
    /// Gateway IP.
    pub gateway: String,
    /// Network type (public or private).
    #[serde(rename = "type")]
    pub network_type: String,
}

/// IPv6 network configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkV6 {
    /// IP address.
    pub ip_address: String,
    /// Network mask prefix length.
    pub netmask: u32,
    /// Gateway IP.
    pub gateway: String,
    /// Network type (public or private).
    #[serde(rename = "type")]
    pub network_type: String,
}

/// Request to create a new Droplet.
#[derive(Debug, Clone, Serialize)]
pub struct CreateDropletRequest {
    /// Name for the new Droplet.
    pub name: String,
    /// Region slug (e.g., "nyc1").
    pub region: String,
    /// Size slug (e.g., "s-1vcpu-1gb").
    pub size: String,
    /// Image ID or slug.
    pub image: DropletImage,
    /// SSH key IDs or fingerprints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_keys: Option<Vec<SshKeyIdentifier>>,
    /// Whether to enable backups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backups: Option<bool>,
    /// Whether to enable IPv6.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<bool>,
    /// Whether to enable monitoring.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitoring: Option<bool>,
    /// User data script.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data: Option<String>,
    /// Tags to apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// VPC UUID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_uuid: Option<String>,
}

/// Image identifier for creating a Droplet.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum DropletImage {
    /// Image ID.
    Id(u64),
    /// Image slug.
    Slug(String),
}

/// SSH key identifier.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SshKeyIdentifier {
    /// Key ID.
    Id(u64),
    /// Key fingerprint.
    Fingerprint(String),
}

/// Response from creating a Droplet.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDropletResponse {
    /// The created Droplet.
    pub droplet: Droplet,
    /// Links for related actions.
    pub links: Option<ActionLinks>,
}

/// Links to actions.
#[derive(Debug, Clone, Deserialize)]
pub struct ActionLinks {
    /// Actions related to this resource.
    pub actions: Option<Vec<ActionLink>>,
}

/// Link to a single action.
#[derive(Debug, Clone, Deserialize)]
pub struct ActionLink {
    /// Action ID.
    pub id: u64,
    /// Relation type.
    pub rel: String,
    /// URL to the action.
    pub href: String,
}

/// Response from listing Droplets.
#[derive(Debug, Clone, Deserialize)]
pub struct ListDropletsResponse {
    /// List of Droplets.
    pub droplets: Vec<Droplet>,
    /// Pagination links.
    pub links: Option<Links>,
    /// Metadata about the response.
    pub meta: Option<Meta>,
}

/// Response from getting a single Droplet.
#[derive(Debug, Clone, Deserialize)]
pub struct GetDropletResponse {
    /// The Droplet.
    pub droplet: Droplet,
}

impl Client {
    /// Lists all Droplets in your account.
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
    /// let response = client.list_droplets(Some(1), Some(20)).await?;
    /// for droplet in response.droplets {
    ///     println!("{}: {}", droplet.id, droplet.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_droplets(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<ListDropletsResponse, Error> {
        let mut url = Url::parse(&format!("{}/droplets", API_BASE_URL)).expect("Invalid URL");

        {
            let mut query = url.query_pairs_mut();
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

    /// Gets a specific Droplet by ID.
    ///
    /// # Arguments
    ///
    /// * `droplet_id` - The ID of the Droplet to retrieve.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::Client;
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let response = client.get_droplet(12345).await?;
    /// println!("Droplet: {}", response.droplet.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_droplet(&self, droplet_id: u64) -> Result<GetDropletResponse, Error> {
        let url = Url::parse(&format!("{}/droplets/{}", API_BASE_URL, droplet_id)).expect("Invalid URL");

        let res = self
            .http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Creates a new Droplet.
    ///
    /// # Arguments
    ///
    /// * `request` - The Droplet creation request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::{Client, CreateDropletRequest, DropletImage};
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let request = CreateDropletRequest {
    ///     name: "my-droplet".to_string(),
    ///     region: "nyc1".to_string(),
    ///     size: "s-1vcpu-1gb".to_string(),
    ///     image: DropletImage::Slug("ubuntu-22-04-x64".to_string()),
    ///     ssh_keys: None,
    ///     backups: Some(false),
    ///     ipv6: Some(true),
    ///     monitoring: Some(true),
    ///     user_data: None,
    ///     tags: Some(vec!["web".to_string()]),
    ///     vpc_uuid: None,
    /// };
    /// let response = client.create_droplet(request).await?;
    /// println!("Created Droplet: {}", response.droplet.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_droplet(
        &self,
        request: CreateDropletRequest,
    ) -> Result<CreateDropletResponse, Error> {
        let url = build_url(API_BASE_URL, "/droplets");

        let res = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&request)
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Deletes a Droplet by ID.
    ///
    /// # Arguments
    ///
    /// * `droplet_id` - The ID of the Droplet to delete.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::Client;
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// client.delete_droplet(12345).await?;
    /// println!("Droplet deleted");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_droplet(&self, droplet_id: u64) -> Result<(), Error> {
        let url = build_url(API_BASE_URL, &format!("/droplets/{}", droplet_id));

        let res = self
            .http_client
            .delete(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        check_api_error(res).await?;
        Ok(())
    }

    /// Lists Droplets by tag.
    ///
    /// # Arguments
    ///
    /// * `tag_name` - The tag to filter by.
    /// * `page` - Page number for pagination.
    /// * `per_page` - Number of items per page.
    pub async fn list_droplets_by_tag(
        &self,
        tag_name: &str,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<ListDropletsResponse, Error> {
        let mut url = build_url(API_BASE_URL, "/droplets");

        {
            let mut query = url.query_pairs_mut();
            query.append_pair("tag_name", tag_name);
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
}
