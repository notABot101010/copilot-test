//! Sizes API endpoints.
//!
//! List available Droplet sizes in DigitalOcean.
//! See: <https://docs.digitalocean.com/reference/api/api-reference/#tag/Sizes>

use serde::{Deserialize, Serialize};

use crate::{check_api_error, Client, Error, Links, Meta, Url, API_BASE_URL};

/// A Droplet size configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SizeInfo {
    /// Unique slug identifier for the size.
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

/// Response from listing sizes.
#[derive(Debug, Clone, Deserialize)]
pub struct ListSizesResponse {
    /// List of sizes.
    pub sizes: Vec<SizeInfo>,
    /// Pagination links.
    pub links: Option<Links>,
    /// Metadata about the response.
    pub meta: Option<Meta>,
}

impl Client {
    /// Lists all available Droplet sizes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::Client;
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let response = client.list_sizes(None, None).await?;
    /// for size in response.sizes {
    ///     println!("{}: {} vCPUs, {} MB RAM, ${}/mo",
    ///         size.slug, size.vcpus, size.memory, size.price_monthly);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_sizes(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<ListSizesResponse, Error> {
        let mut url = Url::parse(&format!("{}/sizes", API_BASE_URL)).expect("Invalid URL");

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
}
