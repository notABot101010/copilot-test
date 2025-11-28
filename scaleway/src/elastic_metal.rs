//! Elastic Metal (Baremetal) API
//!
//! Scaleway Elastic Metal allows you to order dedicated servers on-demand.

use crate::client::{check_api_error, Client, Error};
use reqwest::Url;
use serde::{Deserialize, Serialize};

const BAREMETAL_API_URL: &str = "https://api.scaleway.com/baremetal/v1";

// ============================================================================
// Types
// ============================================================================

/// Elastic Metal server
#[derive(Debug, Clone, Deserialize)]
pub struct BaremetalServer {
    /// Server ID
    pub id: String,
    /// Server name
    pub name: String,
    /// Organization ID
    #[serde(default)]
    pub organization_id: String,
    /// Project ID
    pub project_id: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Status
    pub status: String,
    /// Offer ID
    pub offer_id: String,
    /// Offer name
    #[serde(default)]
    pub offer_name: String,
    /// Zone
    pub zone: String,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// IPs
    #[serde(default)]
    pub ips: Vec<BaremetalIp>,
    /// Install configuration
    pub install: Option<BaremetalInstall>,
    /// Ping status
    #[serde(default)]
    pub ping_status: String,
    /// Options
    #[serde(default)]
    pub options: Vec<ServerOption>,
    /// Creation date
    pub created_at: Option<String>,
    /// Modification date
    pub updated_at: Option<String>,
}

/// Baremetal IP address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaremetalIp {
    /// IP ID
    pub id: String,
    /// IP address
    pub address: String,
    /// Reverse DNS
    #[serde(default)]
    pub reverse: String,
    /// Version (IPv4/IPv6)
    pub version: String,
}

/// Server installation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaremetalInstall {
    /// OS ID
    pub os_id: String,
    /// Hostname
    pub hostname: String,
    /// SSH key IDs
    #[serde(default)]
    pub ssh_key_ids: Vec<String>,
    /// Status
    #[serde(default)]
    pub status: String,
    /// User
    #[serde(default)]
    pub user: String,
    /// Service user
    #[serde(default)]
    pub service_user: String,
}

/// Server option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerOption {
    /// Option ID
    pub id: String,
    /// Option name
    #[serde(default)]
    pub name: String,
    /// Status
    #[serde(default)]
    pub status: String,
    /// Manageable
    #[serde(default)]
    pub manageable: bool,
    /// Expires at
    pub expires_at: Option<String>,
}

/// List servers response
#[derive(Debug, Clone, Deserialize)]
pub struct ListBaremetalServersResponse {
    /// List of servers
    pub servers: Vec<BaremetalServer>,
    /// Total count
    pub total_count: u64,
}

/// Create server request
#[derive(Debug, Clone, Serialize)]
pub struct CreateBaremetalServerRequest {
    /// Offer ID
    pub offer_id: String,
    /// Project ID
    pub project_id: String,
    /// Server name
    pub name: String,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Install configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install: Option<InstallServerConfig>,
    /// Option IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_ids: Option<Vec<String>>,
}

/// Install server configuration
#[derive(Debug, Clone, Serialize)]
pub struct InstallServerConfig {
    /// OS ID
    pub os_id: String,
    /// Hostname
    pub hostname: String,
    /// SSH key IDs
    pub ssh_key_ids: Vec<String>,
    /// User
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Password
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Service user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_user: Option<String>,
    /// Service password
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_password: Option<String>,
}

/// Update server request
#[derive(Debug, Clone, Serialize)]
pub struct UpdateBaremetalServerRequest {
    /// Server name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Install server request
#[derive(Debug, Clone, Serialize)]
pub struct InstallBaremetalServerRequest {
    /// OS ID
    pub os_id: String,
    /// Hostname
    pub hostname: String,
    /// SSH key IDs
    pub ssh_key_ids: Vec<String>,
    /// User
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Password
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Service user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_user: Option<String>,
    /// Service password
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_password: Option<String>,
    /// Partitioning schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partitioning_schema: Option<serde_json::Value>,
}

/// Offer information
#[derive(Debug, Clone, Deserialize)]
pub struct Offer {
    /// Offer ID
    pub id: String,
    /// Offer name
    pub name: String,
    /// Stock status
    pub stock: String,
    /// Bandwidth in bps
    #[serde(default)]
    pub bandwidth: u64,
    /// Commercial range
    #[serde(default)]
    pub commercial_range: String,
    /// Price per hour
    pub price_per_hour: Option<serde_json::Value>,
    /// Price per month
    pub price_per_month: Option<serde_json::Value>,
    /// CPUs
    #[serde(default)]
    pub cpus: Vec<serde_json::Value>,
    /// Memories
    #[serde(default)]
    pub memories: Vec<serde_json::Value>,
    /// Disks
    #[serde(default)]
    pub disks: Vec<serde_json::Value>,
    /// Enabled
    pub enabled: bool,
    /// Options
    #[serde(default)]
    pub options: Vec<OfferOption>,
}

/// Offer option
#[derive(Debug, Clone, Deserialize)]
pub struct OfferOption {
    /// Option ID
    pub id: String,
    /// Option name
    pub name: String,
    /// Enabled
    pub enabled: bool,
    /// Manageable
    pub manageable: bool,
    /// Price per hour
    pub price_per_hour: Option<serde_json::Value>,
    /// Price per month
    pub price_per_month: Option<serde_json::Value>,
}

/// List offers response
#[derive(Debug, Clone, Deserialize)]
pub struct ListOffersResponse {
    /// List of offers
    pub offers: Vec<Offer>,
    /// Total count
    pub total_count: u64,
}

/// Option information
#[derive(Debug, Clone, Deserialize)]
pub struct BaremetalOption {
    /// Option ID
    pub id: String,
    /// Option name
    pub name: String,
    /// Manageable
    pub manageable: bool,
}

/// List options response
#[derive(Debug, Clone, Deserialize)]
pub struct ListOptionsResponse {
    /// List of options
    pub options: Vec<BaremetalOption>,
    /// Total count
    pub total_count: u64,
}

/// Operating system information
#[derive(Debug, Clone, Deserialize)]
pub struct BaremetalOS {
    /// OS ID
    pub id: String,
    /// OS name
    pub name: String,
    /// Version
    #[serde(default)]
    pub version: String,
    /// Enabled
    pub enabled: bool,
    /// License required
    #[serde(default)]
    pub license_required: bool,
}

/// List OS response
#[derive(Debug, Clone, Deserialize)]
pub struct ListOSResponse {
    /// List of operating systems
    pub os: Vec<BaremetalOS>,
    /// Total count
    pub total_count: u64,
}

/// BMC access information
#[derive(Debug, Clone, Deserialize)]
pub struct BMCAccess {
    /// URL
    pub url: String,
    /// Login
    pub login: String,
    /// Password
    pub password: String,
    /// Expires at
    pub expires_at: String,
}

/// Server event
#[derive(Debug, Clone, Deserialize)]
pub struct ServerEvent {
    /// Event ID
    pub id: String,
    /// Action
    pub action: String,
    /// Created at
    pub created_at: String,
    /// Updated at
    pub updated_at: Option<String>,
}

/// List server events response
#[derive(Debug, Clone, Deserialize)]
pub struct ListServerEventsResponse {
    /// List of events
    pub events: Vec<ServerEvent>,
    /// Total count
    pub total_count: u64,
}

/// Server metrics
#[derive(Debug, Clone, Deserialize)]
pub struct ServerMetrics {
    /// Pings
    pub pings: Option<serde_json::Value>,
}

/// Update IP request
#[derive(Debug, Clone, Serialize)]
pub struct UpdateBaremetalIpRequest {
    /// Reverse DNS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse: Option<String>,
}

/// Setting information
#[derive(Debug, Clone, Deserialize)]
pub struct Setting {
    /// Setting ID
    pub id: String,
    /// Type
    #[serde(rename = "type")]
    pub setting_type: String,
    /// Project ID
    pub project_id: String,
    /// Enabled
    pub enabled: bool,
}

/// List settings response
#[derive(Debug, Clone, Deserialize)]
pub struct ListSettingsResponse {
    /// List of settings
    pub settings: Vec<Setting>,
    /// Total count
    pub total_count: u64,
}

/// Update setting request
#[derive(Debug, Clone, Serialize)]
pub struct UpdateSettingRequest {
    /// Enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// Partitioning schema
#[derive(Debug, Clone, Deserialize)]
pub struct PartitioningSchema {
    /// Disks
    pub disks: serde_json::Value,
}

/// Validate partitioning schema request
#[derive(Debug, Clone, Serialize)]
pub struct ValidatePartitioningSchemaRequest {
    /// Offer ID
    pub offer_id: String,
    /// OS ID
    pub os_id: String,
    /// Partitioning schema
    pub partitioning_schema: serde_json::Value,
}

// ============================================================================
// Elastic Metal API Implementation
// ============================================================================

impl Client {
    // ========================================================================
    // Servers
    // ========================================================================

    /// List Elastic Metal servers
    pub async fn list_baremetal_servers(
        &self,
        zone: &str,
        project_id: Option<&str>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<ListBaremetalServersResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/zones/{}/servers", BAREMETAL_API_URL, zone))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = project_id {
                pairs.append_pair("project_id", v);
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
        let response: ListBaremetalServersResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create an Elastic Metal server
    pub async fn create_baremetal_server(
        &self,
        zone: &str,
        request: CreateBaremetalServerRequest,
    ) -> Result<BaremetalServer, Error> {
        let url = format!("{}/zones/{}/servers", BAREMETAL_API_URL, zone);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let server: BaremetalServer = res.json().await.map_err(Error::Json)?;
        Ok(server)
    }

    /// Get an Elastic Metal server by ID
    pub async fn get_baremetal_server(
        &self,
        zone: &str,
        server_id: &str,
    ) -> Result<BaremetalServer, Error> {
        let url = format!("{}/zones/{}/servers/{}", BAREMETAL_API_URL, zone, server_id);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let server: BaremetalServer = res.json().await.map_err(Error::Json)?;
        Ok(server)
    }

    /// Update an Elastic Metal server
    pub async fn update_baremetal_server(
        &self,
        zone: &str,
        server_id: &str,
        request: UpdateBaremetalServerRequest,
    ) -> Result<BaremetalServer, Error> {
        let url = format!("{}/zones/{}/servers/{}", BAREMETAL_API_URL, zone, server_id);

        let res = self
            .http_client
            .patch(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let server: BaremetalServer = res.json().await.map_err(Error::Json)?;
        Ok(server)
    }

    /// Delete an Elastic Metal server
    pub async fn delete_baremetal_server(
        &self,
        zone: &str,
        server_id: &str,
    ) -> Result<BaremetalServer, Error> {
        let url = format!("{}/zones/{}/servers/{}", BAREMETAL_API_URL, zone, server_id);

        let res = self
            .http_client
            .delete(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let server: BaremetalServer = res.json().await.map_err(Error::Json)?;
        Ok(server)
    }

    /// Install OS on an Elastic Metal server
    pub async fn install_baremetal_server(
        &self,
        zone: &str,
        server_id: &str,
        request: InstallBaremetalServerRequest,
    ) -> Result<BaremetalServer, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/install",
            BAREMETAL_API_URL, zone, server_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let server: BaremetalServer = res.json().await.map_err(Error::Json)?;
        Ok(server)
    }

    // ========================================================================
    // Server Actions
    // ========================================================================

    /// Start an Elastic Metal server
    pub async fn start_baremetal_server(
        &self,
        zone: &str,
        server_id: &str,
    ) -> Result<BaremetalServer, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/start",
            BAREMETAL_API_URL, zone, server_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let server: BaremetalServer = res.json().await.map_err(Error::Json)?;
        Ok(server)
    }

    /// Stop an Elastic Metal server
    pub async fn stop_baremetal_server(
        &self,
        zone: &str,
        server_id: &str,
    ) -> Result<BaremetalServer, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/stop",
            BAREMETAL_API_URL, zone, server_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let server: BaremetalServer = res.json().await.map_err(Error::Json)?;
        Ok(server)
    }

    /// Reboot an Elastic Metal server
    pub async fn reboot_baremetal_server(
        &self,
        zone: &str,
        server_id: &str,
    ) -> Result<BaremetalServer, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/reboot",
            BAREMETAL_API_URL, zone, server_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let server: BaremetalServer = res.json().await.map_err(Error::Json)?;
        Ok(server)
    }

    // ========================================================================
    // BMC Access
    // ========================================================================

    /// Get BMC access for a server
    pub async fn get_bmc_access(
        &self,
        zone: &str,
        server_id: &str,
    ) -> Result<BMCAccess, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/bmc-access",
            BAREMETAL_API_URL, zone, server_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let access: BMCAccess = res.json().await.map_err(Error::Json)?;
        Ok(access)
    }

    /// Start BMC access for a server
    pub async fn start_bmc_access(
        &self,
        zone: &str,
        server_id: &str,
    ) -> Result<BMCAccess, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/bmc-access",
            BAREMETAL_API_URL, zone, server_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let access: BMCAccess = res.json().await.map_err(Error::Json)?;
        Ok(access)
    }

    /// Stop BMC access for a server
    pub async fn stop_bmc_access(&self, zone: &str, server_id: &str) -> Result<(), Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/bmc-access",
            BAREMETAL_API_URL, zone, server_id
        );

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
    // Server Events & Metrics
    // ========================================================================

    /// List server events
    pub async fn list_baremetal_server_events(
        &self,
        zone: &str,
        server_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<ListServerEventsResponse, Error> {
        let mut url = Url::parse(&format!(
            "{}/zones/{}/servers/{}/events",
            BAREMETAL_API_URL, zone, server_id
        ))
        .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
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
        let response: ListServerEventsResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Get server metrics
    pub async fn get_baremetal_server_metrics(
        &self,
        zone: &str,
        server_id: &str,
    ) -> Result<ServerMetrics, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/metrics",
            BAREMETAL_API_URL, zone, server_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let metrics: ServerMetrics = res.json().await.map_err(Error::Json)?;
        Ok(metrics)
    }

    // ========================================================================
    // Server Options
    // ========================================================================

    /// Add an option to a server
    pub async fn add_baremetal_server_option(
        &self,
        zone: &str,
        server_id: &str,
        option_id: &str,
    ) -> Result<BaremetalServer, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/options/{}",
            BAREMETAL_API_URL, zone, server_id, option_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let server: BaremetalServer = res.json().await.map_err(Error::Json)?;
        Ok(server)
    }

    /// Remove an option from a server
    pub async fn delete_baremetal_server_option(
        &self,
        zone: &str,
        server_id: &str,
        option_id: &str,
    ) -> Result<BaremetalServer, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/options/{}",
            BAREMETAL_API_URL, zone, server_id, option_id
        );

        let res = self
            .http_client
            .delete(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let server: BaremetalServer = res.json().await.map_err(Error::Json)?;
        Ok(server)
    }

    // ========================================================================
    // Server IP
    // ========================================================================

    /// Update server IP (reverse DNS)
    pub async fn update_baremetal_ip(
        &self,
        zone: &str,
        server_id: &str,
        ip_id: &str,
        request: UpdateBaremetalIpRequest,
    ) -> Result<BaremetalIp, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/ips/{}",
            BAREMETAL_API_URL, zone, server_id, ip_id
        );

        let res = self
            .http_client
            .patch(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let ip: BaremetalIp = res.json().await.map_err(Error::Json)?;
        Ok(ip)
    }

    // ========================================================================
    // Offers
    // ========================================================================

    /// List available offers
    pub async fn list_baremetal_offers(
        &self,
        zone: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<ListOffersResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/zones/{}/offers", BAREMETAL_API_URL, zone))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
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
        let response: ListOffersResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Get an offer by ID
    pub async fn get_baremetal_offer(&self, zone: &str, offer_id: &str) -> Result<Offer, Error> {
        let url = format!("{}/zones/{}/offers/{}", BAREMETAL_API_URL, zone, offer_id);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let offer: Offer = res.json().await.map_err(Error::Json)?;
        Ok(offer)
    }

    // ========================================================================
    // Options
    // ========================================================================

    /// List available options
    pub async fn list_baremetal_options(
        &self,
        zone: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<ListOptionsResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/zones/{}/options", BAREMETAL_API_URL, zone))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
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
        let response: ListOptionsResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Get an option by ID
    pub async fn get_baremetal_option(
        &self,
        zone: &str,
        option_id: &str,
    ) -> Result<BaremetalOption, Error> {
        let url = format!("{}/zones/{}/options/{}", BAREMETAL_API_URL, zone, option_id);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let option: BaremetalOption = res.json().await.map_err(Error::Json)?;
        Ok(option)
    }

    // ========================================================================
    // Operating Systems
    // ========================================================================

    /// List available operating systems
    pub async fn list_baremetal_os(
        &self,
        zone: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<ListOSResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/zones/{}/os", BAREMETAL_API_URL, zone))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
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
        let response: ListOSResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Get an OS by ID
    pub async fn get_baremetal_os(&self, zone: &str, os_id: &str) -> Result<BaremetalOS, Error> {
        let url = format!("{}/zones/{}/os/{}", BAREMETAL_API_URL, zone, os_id);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let os: BaremetalOS = res.json().await.map_err(Error::Json)?;
        Ok(os)
    }

    // ========================================================================
    // Partitioning
    // ========================================================================

    /// Get default partitioning schema
    pub async fn get_default_partitioning_schema(
        &self,
        zone: &str,
        offer_id: &str,
        os_id: &str,
    ) -> Result<PartitioningSchema, Error> {
        let url = format!(
            "{}/zones/{}/partitioning-schemas/default?offer_id={}&os_id={}",
            BAREMETAL_API_URL, zone, offer_id, os_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let schema: PartitioningSchema = res.json().await.map_err(Error::Json)?;
        Ok(schema)
    }

    /// Validate a partitioning schema
    pub async fn validate_partitioning_schema(
        &self,
        zone: &str,
        request: ValidatePartitioningSchemaRequest,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/zones/{}/partitioning-schemas/validate",
            BAREMETAL_API_URL, zone
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        check_api_error(res).await?;
        Ok(())
    }

    // ========================================================================
    // Settings
    // ========================================================================

    /// List settings
    pub async fn list_baremetal_settings(
        &self,
        zone: &str,
        project_id: Option<&str>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<ListSettingsResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/zones/{}/settings", BAREMETAL_API_URL, zone))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = project_id {
                pairs.append_pair("project_id", v);
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
        let response: ListSettingsResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Update a setting
    pub async fn update_baremetal_setting(
        &self,
        zone: &str,
        setting_id: &str,
        request: UpdateSettingRequest,
    ) -> Result<Setting, Error> {
        let url = format!("{}/zones/{}/settings/{}", BAREMETAL_API_URL, zone, setting_id);

        let res = self
            .http_client
            .patch(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let setting: Setting = res.json().await.map_err(Error::Json)?;
        Ok(setting)
    }
}
