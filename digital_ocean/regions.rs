//! Regions API endpoints.
//!
//! List available regions in DigitalOcean.
//! See: <https://docs.digitalocean.com/reference/api/api-reference/#tag/Regions>

use serde::{Deserialize, Serialize};

use crate::{check_api_error, Client, Error, Links, Meta, API_BASE_URL};

/// A DigitalOcean region.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegionInfo {
    /// Unique slug identifier for the region.
    pub slug: String,
    /// Human-readable name for the region.
    pub name: String,
    /// Available Droplet sizes in this region.
    pub sizes: Vec<String>,
    /// Whether the region is available for new Droplets.
    pub available: bool,
    /// Features available in this region.
    pub features: Vec<String>,
}

/// Response from listing regions.
#[derive(Debug, Clone, Deserialize)]
pub struct ListRegionsResponse {
    /// List of regions.
    pub regions: Vec<RegionInfo>,
    /// Pagination links.
    pub links: Option<Links>,
    /// Metadata about the response.
    pub meta: Option<Meta>,
}

impl Client {
    /// Lists all available regions.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::Client;
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let response = client.list_regions(None, None).await?;
    /// for region in response.regions {
    ///     println!("{}: {} (available: {})", region.slug, region.name, region.available);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_regions(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<ListRegionsResponse, Error> {
        let mut url = format!("{}/regions", API_BASE_URL);
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
}
