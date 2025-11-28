//! Instances API
//!
//! Scaleway Instances are computing units providing resources to run your applications on.

use crate::client::{check_api_error, Client, Error};
use reqwest::Url;
use serde::{Deserialize, Serialize};

const INSTANCE_API_URL: &str = "https://api.scaleway.com/instance/v1";

// ============================================================================
// Types
// ============================================================================

/// Volume configuration for server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerVolume {
    /// Volume ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Volume name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Volume size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Volume type (l_ssd, sbs_volume)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_type: Option<String>,
    /// Boot volume flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot: Option<bool>,
    /// Base snapshot ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_snapshot: Option<String>,
}

/// Server state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    /// Server ID
    pub id: String,
    /// Server name
    pub name: String,
    /// Organization ID
    #[serde(default)]
    pub organization: String,
    /// Project ID
    pub project: String,
    /// Commercial type
    pub commercial_type: String,
    /// Hostname
    #[serde(default)]
    pub hostname: String,
    /// State
    pub state: String,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Volumes
    #[serde(default)]
    pub volumes: serde_json::Value,
    /// Public IP
    pub public_ip: Option<serde_json::Value>,
    /// Private IP
    pub private_ip: Option<String>,
    /// Zone
    pub zone: String,
    /// Creation date
    pub creation_date: Option<String>,
    /// Modification date
    pub modification_date: Option<String>,
    /// Image
    pub image: Option<serde_json::Value>,
    /// Arch
    #[serde(default)]
    pub arch: String,
    /// Security group
    pub security_group: Option<serde_json::Value>,
}

/// List servers response
#[derive(Debug, Clone, Deserialize)]
pub struct ListServersResponse {
    /// List of servers
    pub servers: Vec<Server>,
}

/// Create server request
#[derive(Debug, Clone, Serialize)]
pub struct CreateServerRequest {
    /// Server name
    pub name: String,
    /// Commercial type
    pub commercial_type: String,
    /// Project ID
    pub project: String,
    /// Image ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Volumes configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<serde_json::Value>,
    /// Enable IPv6
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_ipv6: Option<bool>,
    /// Public IP ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ip: Option<String>,
    /// Security group ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_group: Option<String>,
    /// Placement group ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_group: Option<String>,
}

/// Create server response
#[derive(Debug, Clone, Deserialize)]
pub struct CreateServerResponse {
    /// Created server
    pub server: Server,
}

/// Get server response
#[derive(Debug, Clone, Deserialize)]
pub struct GetServerResponse {
    /// Server details
    pub server: Server,
}

/// Update server request
#[derive(Debug, Clone, Serialize)]
pub struct UpdateServerRequest {
    /// Server name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Enable IPv6
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_ipv6: Option<bool>,
}

/// Update server response
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateServerResponse {
    /// Updated server
    pub server: Server,
}

/// Server action request
#[derive(Debug, Clone, Serialize)]
pub struct ServerActionRequest {
    /// Action to perform
    pub action: String,
}

/// Server action response
#[derive(Debug, Clone, Deserialize)]
pub struct ServerActionResponse {
    /// Task information
    pub task: Option<serde_json::Value>,
}

/// Volume information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    /// Volume ID
    pub id: String,
    /// Volume name
    pub name: String,
    /// Volume size in bytes
    pub size: u64,
    /// Volume type
    pub volume_type: String,
    /// State
    pub state: String,
    /// Zone
    pub zone: String,
    /// Server ID
    pub server: Option<serde_json::Value>,
    /// Creation date
    pub creation_date: Option<String>,
    /// Modification date
    pub modification_date: Option<String>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// List volumes response
#[derive(Debug, Clone, Deserialize)]
pub struct ListVolumesResponse {
    /// List of volumes
    pub volumes: Vec<Volume>,
}

/// Create volume request
#[derive(Debug, Clone, Serialize)]
pub struct CreateVolumeRequest {
    /// Volume name
    pub name: String,
    /// Volume size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Volume type
    pub volume_type: String,
    /// Project ID
    pub project: String,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Create volume response
#[derive(Debug, Clone, Deserialize)]
pub struct CreateVolumeResponse {
    /// Created volume
    pub volume: Volume,
}

/// Get volume response
#[derive(Debug, Clone, Deserialize)]
pub struct GetVolumeResponse {
    /// Volume details
    pub volume: Volume,
}

/// Update volume request
#[derive(Debug, Clone, Serialize)]
pub struct UpdateVolumeRequest {
    /// Volume name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    /// Image ID
    pub id: String,
    /// Image name
    pub name: String,
    /// Architecture
    pub arch: String,
    /// State
    pub state: String,
    /// Zone
    pub zone: String,
    /// Public image flag
    #[serde(default)]
    pub public: bool,
    /// Creation date
    pub creation_date: Option<String>,
    /// Modification date
    pub modification_date: Option<String>,
    /// Root volume
    pub root_volume: Option<serde_json::Value>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// List images response
#[derive(Debug, Clone, Deserialize)]
pub struct ListImagesResponse {
    /// List of images
    pub images: Vec<Image>,
}

/// Create image request
#[derive(Debug, Clone, Serialize)]
pub struct CreateImageRequest {
    /// Image name
    pub name: String,
    /// Architecture
    pub arch: String,
    /// Root volume ID
    pub root_volume: String,
    /// Project ID
    pub project: String,
    /// Public image flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Create image response
#[derive(Debug, Clone, Deserialize)]
pub struct CreateImageResponse {
    /// Created image
    pub image: Image,
}

/// Get image response
#[derive(Debug, Clone, Deserialize)]
pub struct GetImageResponse {
    /// Image details
    pub image: Image,
}

/// Snapshot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Snapshot ID
    pub id: String,
    /// Snapshot name
    pub name: String,
    /// Size in bytes
    pub size: u64,
    /// State
    pub state: String,
    /// Volume type
    pub volume_type: String,
    /// Zone
    pub zone: String,
    /// Base volume
    pub base_volume: Option<serde_json::Value>,
    /// Creation date
    pub creation_date: Option<String>,
    /// Modification date
    pub modification_date: Option<String>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// List snapshots response
#[derive(Debug, Clone, Deserialize)]
pub struct ListSnapshotsResponse {
    /// List of snapshots
    pub snapshots: Vec<Snapshot>,
}

/// Create snapshot request
#[derive(Debug, Clone, Serialize)]
pub struct CreateSnapshotRequest {
    /// Snapshot name
    pub name: String,
    /// Volume ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_id: Option<String>,
    /// Project ID
    pub project: String,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Create snapshot response
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSnapshotResponse {
    /// Created snapshot
    pub snapshot: Snapshot,
}

/// Get snapshot response
#[derive(Debug, Clone, Deserialize)]
pub struct GetSnapshotResponse {
    /// Snapshot details
    pub snapshot: Snapshot,
}

/// IP address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ip {
    /// IP ID
    pub id: String,
    /// IP address
    pub address: String,
    /// Zone
    pub zone: String,
    /// Project ID
    pub project: String,
    /// Reverse DNS
    pub reverse: Option<String>,
    /// Server ID
    pub server: Option<serde_json::Value>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// List IPs response
#[derive(Debug, Clone, Deserialize)]
pub struct ListIpsResponse {
    /// List of IPs
    pub ips: Vec<Ip>,
}

/// Create IP request
#[derive(Debug, Clone, Serialize)]
pub struct CreateIpRequest {
    /// Project ID
    pub project: String,
    /// Server ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// IP type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ip_type: Option<String>,
}

/// Create IP response
#[derive(Debug, Clone, Deserialize)]
pub struct CreateIpResponse {
    /// Created IP
    pub ip: Ip,
}

/// Get IP response
#[derive(Debug, Clone, Deserialize)]
pub struct GetIpResponse {
    /// IP details
    pub ip: Ip,
}

/// Update IP request
#[derive(Debug, Clone, Serialize)]
pub struct UpdateIpRequest {
    /// Reverse DNS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse: Option<String>,
    /// Server ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Security group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGroup {
    /// Security group ID
    pub id: String,
    /// Security group name
    pub name: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Inbound default policy
    pub inbound_default_policy: String,
    /// Outbound default policy
    pub outbound_default_policy: String,
    /// Stateful flag
    pub stateful: bool,
    /// Zone
    pub zone: String,
    /// Project ID
    pub project: String,
    /// Creation date
    pub creation_date: Option<String>,
    /// Modification date
    pub modification_date: Option<String>,
}

/// List security groups response
#[derive(Debug, Clone, Deserialize)]
pub struct ListSecurityGroupsResponse {
    /// List of security groups
    pub security_groups: Vec<SecurityGroup>,
}

/// Create security group request
#[derive(Debug, Clone, Serialize)]
pub struct CreateSecurityGroupRequest {
    /// Security group name
    pub name: String,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Inbound default policy
    pub inbound_default_policy: String,
    /// Outbound default policy
    pub outbound_default_policy: String,
    /// Stateful flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stateful: Option<bool>,
    /// Project ID
    pub project: String,
}

/// Create security group response
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSecurityGroupResponse {
    /// Created security group
    pub security_group: SecurityGroup,
}

/// Get security group response
#[derive(Debug, Clone, Deserialize)]
pub struct GetSecurityGroupResponse {
    /// Security group details
    pub security_group: SecurityGroup,
}

/// Security group rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGroupRule {
    /// Rule ID
    pub id: String,
    /// Protocol
    pub protocol: String,
    /// Direction (inbound/outbound)
    pub direction: String,
    /// Action (accept/drop)
    pub action: String,
    /// IP range
    pub ip_range: String,
    /// Destination port from
    pub dest_port_from: Option<u32>,
    /// Destination port to
    pub dest_port_to: Option<u32>,
    /// Position
    pub position: u32,
}

/// List security group rules response
#[derive(Debug, Clone, Deserialize)]
pub struct ListSecurityGroupRulesResponse {
    /// List of rules
    pub rules: Vec<SecurityGroupRule>,
}

/// Create security group rule request
#[derive(Debug, Clone, Serialize)]
pub struct CreateSecurityGroupRuleRequest {
    /// Protocol
    pub protocol: String,
    /// Direction
    pub direction: String,
    /// Action
    pub action: String,
    /// IP range
    pub ip_range: String,
    /// Destination port from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest_port_from: Option<u32>,
    /// Destination port to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest_port_to: Option<u32>,
    /// Position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,
}

/// Create security group rule response
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSecurityGroupRuleResponse {
    /// Created rule
    pub rule: SecurityGroupRule,
}

/// Placement group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementGroup {
    /// Placement group ID
    pub id: String,
    /// Placement group name
    pub name: String,
    /// Policy mode
    pub policy_mode: String,
    /// Policy type
    pub policy_type: String,
    /// Zone
    pub zone: String,
    /// Project ID
    pub project: String,
}

/// List placement groups response
#[derive(Debug, Clone, Deserialize)]
pub struct ListPlacementGroupsResponse {
    /// List of placement groups
    pub placement_groups: Vec<PlacementGroup>,
}

/// Create placement group request
#[derive(Debug, Clone, Serialize)]
pub struct CreatePlacementGroupRequest {
    /// Placement group name
    pub name: String,
    /// Policy mode
    pub policy_mode: String,
    /// Policy type
    pub policy_type: String,
    /// Project ID
    pub project: String,
}

/// Create placement group response
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePlacementGroupResponse {
    /// Created placement group
    pub placement_group: PlacementGroup,
}

/// Get placement group response
#[derive(Debug, Clone, Deserialize)]
pub struct GetPlacementGroupResponse {
    /// Placement group details
    pub placement_group: PlacementGroup,
}

/// Private NIC information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateNIC {
    /// Private NIC ID
    pub id: String,
    /// Server ID
    pub server_id: String,
    /// Private network ID
    pub private_network_id: String,
    /// MAC address
    pub mac_address: String,
    /// State
    pub state: String,
}

/// List private NICs response
#[derive(Debug, Clone, Deserialize)]
pub struct ListPrivateNICsResponse {
    /// List of private NICs
    pub private_nics: Vec<PrivateNIC>,
}

/// Create private NIC request
#[derive(Debug, Clone, Serialize)]
pub struct CreatePrivateNICRequest {
    /// Private network ID
    pub private_network_id: String,
}

/// Create private NIC response
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePrivateNICResponse {
    /// Created private NIC
    pub private_nic: PrivateNIC,
}

/// Server type information
#[derive(Debug, Clone, Deserialize)]
pub struct ServerType {
    /// Monthly price
    #[serde(default)]
    pub monthly_price: f64,
    /// Hourly price
    #[serde(default)]
    pub hourly_price: f64,
    /// Available RAM in bytes
    #[serde(default)]
    pub ram: u64,
    /// Number of CPU cores
    #[serde(default)]
    pub ncpus: u32,
    /// GPU information
    pub gpu: Option<u64>,
    /// Architecture
    #[serde(default)]
    pub arch: String,
}

/// List server types response
#[derive(Debug, Clone, Deserialize)]
pub struct ListServerTypesResponse {
    /// Map of server types
    pub servers: serde_json::Value,
}

/// Volume types response
#[derive(Debug, Clone, Deserialize)]
pub struct ListVolumeTypesResponse {
    /// Map of volume types
    pub volumes: serde_json::Value,
}

/// Attach volume request
#[derive(Debug, Clone, Serialize)]
pub struct AttachVolumeRequest {
    /// Volume ID
    pub volume_id: String,
}

/// Attach volume response
#[derive(Debug, Clone, Deserialize)]
pub struct AttachVolumeResponse {
    /// Server
    pub server: Server,
}

/// Detach volume request
#[derive(Debug, Clone, Serialize)]
pub struct DetachVolumeRequest {
    /// Volume ID
    pub volume_id: String,
}

/// Detach volume response
#[derive(Debug, Clone, Deserialize)]
pub struct DetachVolumeResponse {
    /// Server
    pub server: Server,
}

/// Dashboard information
#[derive(Debug, Clone, Deserialize)]
pub struct Dashboard {
    /// Running servers count
    pub running_servers_count: u32,
    /// Total servers count
    pub servers_count: u32,
    /// Images count
    pub images_count: u32,
    /// Snapshots count
    pub snapshots_count: u32,
    /// Volumes l_ssd count
    pub volumes_l_ssd_count: u32,
    /// Volumes b_ssd count
    pub volumes_b_ssd_count: u32,
    /// IPs count
    pub ips_count: u32,
    /// Security groups count
    pub security_groups_count: u32,
    /// Private NICs count
    #[serde(default)]
    pub private_nics_count: u32,
}

/// Get dashboard response
#[derive(Debug, Clone, Deserialize)]
pub struct GetDashboardResponse {
    /// Dashboard
    pub dashboard: Dashboard,
}

// ============================================================================
// Instances API Implementation
// ============================================================================

impl Client {
    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get dashboard information for a zone
    pub async fn get_instance_dashboard(&self, zone: &str) -> Result<GetDashboardResponse, Error> {
        let url = format!("{}/zones/{}/dashboard", INSTANCE_API_URL, zone);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: GetDashboardResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    // ========================================================================
    // Servers
    // ========================================================================

    /// List all servers in a zone
    pub async fn list_servers(
        &self,
        zone: &str,
        project: Option<&str>,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<ListServersResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/zones/{}/servers", INSTANCE_API_URL, zone))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = project {
                pairs.append_pair("project", v);
            }
            if let Some(v) = per_page {
                pairs.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = page {
                pairs.append_pair("page", &v.to_string());
            }
        }

        let res = self
            .http_client
            .get(url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListServersResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create a new server
    pub async fn create_server(
        &self,
        zone: &str,
        request: CreateServerRequest,
    ) -> Result<CreateServerResponse, Error> {
        let url = format!("{}/zones/{}/servers", INSTANCE_API_URL, zone);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: CreateServerResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Get a server by ID
    pub async fn get_server(&self, zone: &str, server_id: &str) -> Result<GetServerResponse, Error> {
        let url = format!("{}/zones/{}/servers/{}", INSTANCE_API_URL, zone, server_id);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: GetServerResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Update a server
    pub async fn update_server(
        &self,
        zone: &str,
        server_id: &str,
        request: UpdateServerRequest,
    ) -> Result<UpdateServerResponse, Error> {
        let url = format!("{}/zones/{}/servers/{}", INSTANCE_API_URL, zone, server_id);

        let res = self
            .http_client
            .patch(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: UpdateServerResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Delete a server
    pub async fn delete_server(&self, zone: &str, server_id: &str) -> Result<(), Error> {
        let url = format!("{}/zones/{}/servers/{}", INSTANCE_API_URL, zone, server_id);

        let res = self
            .http_client
            .delete(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        check_api_error(res).await?;
        Ok(())
    }

    /// Perform a server action (poweron, poweroff, stop_in_place, reboot, backup, terminate)
    pub async fn server_action(
        &self,
        zone: &str,
        server_id: &str,
        action: &str,
    ) -> Result<ServerActionResponse, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/action",
            INSTANCE_API_URL, zone, server_id
        );

        let request = ServerActionRequest {
            action: action.to_string(),
        };

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ServerActionResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Attach a volume to a server
    pub async fn attach_server_volume(
        &self,
        zone: &str,
        server_id: &str,
        volume_id: &str,
    ) -> Result<AttachVolumeResponse, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/attach-volume",
            INSTANCE_API_URL, zone, server_id
        );

        let request = AttachVolumeRequest {
            volume_id: volume_id.to_string(),
        };

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: AttachVolumeResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Detach a volume from a server
    pub async fn detach_server_volume(
        &self,
        zone: &str,
        server_id: &str,
        volume_id: &str,
    ) -> Result<DetachVolumeResponse, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/detach-volume",
            INSTANCE_API_URL, zone, server_id
        );

        let request = DetachVolumeRequest {
            volume_id: volume_id.to_string(),
        };

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: DetachVolumeResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    // ========================================================================
    // Volumes
    // ========================================================================

    /// List all volumes in a zone
    pub async fn list_volumes(
        &self,
        zone: &str,
        project: Option<&str>,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<ListVolumesResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/zones/{}/volumes", INSTANCE_API_URL, zone))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = project {
                pairs.append_pair("project", v);
            }
            if let Some(v) = per_page {
                pairs.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = page {
                pairs.append_pair("page", &v.to_string());
            }
        }

        let res = self
            .http_client
            .get(url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListVolumesResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create a new volume
    pub async fn create_volume(
        &self,
        zone: &str,
        request: CreateVolumeRequest,
    ) -> Result<CreateVolumeResponse, Error> {
        let url = format!("{}/zones/{}/volumes", INSTANCE_API_URL, zone);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: CreateVolumeResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Get a volume by ID
    pub async fn get_volume(&self, zone: &str, volume_id: &str) -> Result<GetVolumeResponse, Error> {
        let url = format!("{}/zones/{}/volumes/{}", INSTANCE_API_URL, zone, volume_id);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: GetVolumeResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Update a volume
    pub async fn update_volume(
        &self,
        zone: &str,
        volume_id: &str,
        request: UpdateVolumeRequest,
    ) -> Result<Volume, Error> {
        let url = format!("{}/zones/{}/volumes/{}", INSTANCE_API_URL, zone, volume_id);

        let res = self
            .http_client
            .patch(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: GetVolumeResponse = res.json().await.map_err(Error::Json)?;
        Ok(response.volume)
    }

    /// Delete a volume
    pub async fn delete_volume(&self, zone: &str, volume_id: &str) -> Result<(), Error> {
        let url = format!("{}/zones/{}/volumes/{}", INSTANCE_API_URL, zone, volume_id);

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
    // Images
    // ========================================================================

    /// List all images in a zone
    pub async fn list_images(
        &self,
        zone: &str,
        project: Option<&str>,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<ListImagesResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/zones/{}/images", INSTANCE_API_URL, zone))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = project {
                pairs.append_pair("project", v);
            }
            if let Some(v) = per_page {
                pairs.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = page {
                pairs.append_pair("page", &v.to_string());
            }
        }

        let res = self
            .http_client
            .get(url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListImagesResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create a new image
    pub async fn create_image(
        &self,
        zone: &str,
        request: CreateImageRequest,
    ) -> Result<CreateImageResponse, Error> {
        let url = format!("{}/zones/{}/images", INSTANCE_API_URL, zone);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: CreateImageResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Get an image by ID
    pub async fn get_image(&self, zone: &str, image_id: &str) -> Result<GetImageResponse, Error> {
        let url = format!("{}/zones/{}/images/{}", INSTANCE_API_URL, zone, image_id);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: GetImageResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Delete an image
    pub async fn delete_image(&self, zone: &str, image_id: &str) -> Result<(), Error> {
        let url = format!("{}/zones/{}/images/{}", INSTANCE_API_URL, zone, image_id);

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
    // Snapshots
    // ========================================================================

    /// List all snapshots in a zone
    pub async fn list_snapshots(
        &self,
        zone: &str,
        project: Option<&str>,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<ListSnapshotsResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/zones/{}/snapshots", INSTANCE_API_URL, zone))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = project {
                pairs.append_pair("project", v);
            }
            if let Some(v) = per_page {
                pairs.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = page {
                pairs.append_pair("page", &v.to_string());
            }
        }

        let res = self
            .http_client
            .get(url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListSnapshotsResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create a new snapshot
    pub async fn create_snapshot(
        &self,
        zone: &str,
        request: CreateSnapshotRequest,
    ) -> Result<CreateSnapshotResponse, Error> {
        let url = format!("{}/zones/{}/snapshots", INSTANCE_API_URL, zone);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: CreateSnapshotResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Get a snapshot by ID
    pub async fn get_snapshot(
        &self,
        zone: &str,
        snapshot_id: &str,
    ) -> Result<GetSnapshotResponse, Error> {
        let url = format!("{}/zones/{}/snapshots/{}", INSTANCE_API_URL, zone, snapshot_id);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: GetSnapshotResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Delete a snapshot
    pub async fn delete_snapshot(&self, zone: &str, snapshot_id: &str) -> Result<(), Error> {
        let url = format!("{}/zones/{}/snapshots/{}", INSTANCE_API_URL, zone, snapshot_id);

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
    // IPs
    // ========================================================================

    /// List all IPs in a zone
    pub async fn list_ips(
        &self,
        zone: &str,
        project: Option<&str>,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<ListIpsResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/zones/{}/ips", INSTANCE_API_URL, zone))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = project {
                pairs.append_pair("project", v);
            }
            if let Some(v) = per_page {
                pairs.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = page {
                pairs.append_pair("page", &v.to_string());
            }
        }

        let res = self
            .http_client
            .get(url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListIpsResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create a new IP
    pub async fn create_ip(
        &self,
        zone: &str,
        request: CreateIpRequest,
    ) -> Result<CreateIpResponse, Error> {
        let url = format!("{}/zones/{}/ips", INSTANCE_API_URL, zone);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: CreateIpResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Get an IP by ID
    pub async fn get_ip(&self, zone: &str, ip_id: &str) -> Result<GetIpResponse, Error> {
        let url = format!("{}/zones/{}/ips/{}", INSTANCE_API_URL, zone, ip_id);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: GetIpResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Update an IP
    pub async fn update_ip(
        &self,
        zone: &str,
        ip_id: &str,
        request: UpdateIpRequest,
    ) -> Result<Ip, Error> {
        let url = format!("{}/zones/{}/ips/{}", INSTANCE_API_URL, zone, ip_id);

        let res = self
            .http_client
            .patch(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: GetIpResponse = res.json().await.map_err(Error::Json)?;
        Ok(response.ip)
    }

    /// Delete an IP
    pub async fn delete_ip(&self, zone: &str, ip_id: &str) -> Result<(), Error> {
        let url = format!("{}/zones/{}/ips/{}", INSTANCE_API_URL, zone, ip_id);

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
    // Security Groups
    // ========================================================================

    /// List all security groups in a zone
    pub async fn list_security_groups(
        &self,
        zone: &str,
        project: Option<&str>,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<ListSecurityGroupsResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/zones/{}/security_groups", INSTANCE_API_URL, zone))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = project {
                pairs.append_pair("project", v);
            }
            if let Some(v) = per_page {
                pairs.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = page {
                pairs.append_pair("page", &v.to_string());
            }
        }

        let res = self
            .http_client
            .get(url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListSecurityGroupsResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create a new security group
    pub async fn create_security_group(
        &self,
        zone: &str,
        request: CreateSecurityGroupRequest,
    ) -> Result<CreateSecurityGroupResponse, Error> {
        let url = format!("{}/zones/{}/security_groups", INSTANCE_API_URL, zone);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: CreateSecurityGroupResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Get a security group by ID
    pub async fn get_security_group(
        &self,
        zone: &str,
        security_group_id: &str,
    ) -> Result<GetSecurityGroupResponse, Error> {
        let url = format!(
            "{}/zones/{}/security_groups/{}",
            INSTANCE_API_URL, zone, security_group_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: GetSecurityGroupResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Delete a security group
    pub async fn delete_security_group(
        &self,
        zone: &str,
        security_group_id: &str,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/zones/{}/security_groups/{}",
            INSTANCE_API_URL, zone, security_group_id
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

    /// List security group rules
    pub async fn list_security_group_rules(
        &self,
        zone: &str,
        security_group_id: &str,
    ) -> Result<ListSecurityGroupRulesResponse, Error> {
        let url = format!(
            "{}/zones/{}/security_groups/{}/rules",
            INSTANCE_API_URL, zone, security_group_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListSecurityGroupRulesResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create a security group rule
    pub async fn create_security_group_rule(
        &self,
        zone: &str,
        security_group_id: &str,
        request: CreateSecurityGroupRuleRequest,
    ) -> Result<CreateSecurityGroupRuleResponse, Error> {
        let url = format!(
            "{}/zones/{}/security_groups/{}/rules",
            INSTANCE_API_URL, zone, security_group_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: CreateSecurityGroupRuleResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Delete a security group rule
    pub async fn delete_security_group_rule(
        &self,
        zone: &str,
        security_group_id: &str,
        rule_id: &str,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/zones/{}/security_groups/{}/rules/{}",
            INSTANCE_API_URL, zone, security_group_id, rule_id
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
    // Placement Groups
    // ========================================================================

    /// List all placement groups in a zone
    pub async fn list_placement_groups(
        &self,
        zone: &str,
        project: Option<&str>,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<ListPlacementGroupsResponse, Error> {
        let mut url =
            Url::parse(&format!("{}/zones/{}/placement_groups", INSTANCE_API_URL, zone))
                .expect("valid URL");

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = project {
                pairs.append_pair("project", v);
            }
            if let Some(v) = per_page {
                pairs.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = page {
                pairs.append_pair("page", &v.to_string());
            }
        }

        let res = self
            .http_client
            .get(url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListPlacementGroupsResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create a new placement group
    pub async fn create_placement_group(
        &self,
        zone: &str,
        request: CreatePlacementGroupRequest,
    ) -> Result<CreatePlacementGroupResponse, Error> {
        let url = format!("{}/zones/{}/placement_groups", INSTANCE_API_URL, zone);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: CreatePlacementGroupResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Get a placement group by ID
    pub async fn get_placement_group(
        &self,
        zone: &str,
        placement_group_id: &str,
    ) -> Result<GetPlacementGroupResponse, Error> {
        let url = format!(
            "{}/zones/{}/placement_groups/{}",
            INSTANCE_API_URL, zone, placement_group_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: GetPlacementGroupResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Delete a placement group
    pub async fn delete_placement_group(
        &self,
        zone: &str,
        placement_group_id: &str,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/zones/{}/placement_groups/{}",
            INSTANCE_API_URL, zone, placement_group_id
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
    // Private NICs
    // ========================================================================

    /// List private NICs for a server
    pub async fn list_private_nics(
        &self,
        zone: &str,
        server_id: &str,
    ) -> Result<ListPrivateNICsResponse, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/private_nics",
            INSTANCE_API_URL, zone, server_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListPrivateNICsResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create a private NIC
    pub async fn create_private_nic(
        &self,
        zone: &str,
        server_id: &str,
        request: CreatePrivateNICRequest,
    ) -> Result<CreatePrivateNICResponse, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/private_nics",
            INSTANCE_API_URL, zone, server_id
        );

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: CreatePrivateNICResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Delete a private NIC
    pub async fn delete_private_nic(
        &self,
        zone: &str,
        server_id: &str,
        private_nic_id: &str,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/private_nics/{}",
            INSTANCE_API_URL, zone, server_id, private_nic_id
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
    // Server Types & Volume Types
    // ========================================================================

    /// List available server types in a zone
    pub async fn list_server_types(&self, zone: &str) -> Result<ListServerTypesResponse, Error> {
        let url = format!("{}/zones/{}/products/servers", INSTANCE_API_URL, zone);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListServerTypesResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// List available volume types in a zone
    pub async fn list_volume_types(&self, zone: &str) -> Result<ListVolumeTypesResponse, Error> {
        let url = format!("{}/zones/{}/products/volumes", INSTANCE_API_URL, zone);

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: ListVolumeTypesResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    // ========================================================================
    // User Data
    // ========================================================================

    /// List user data keys for a server
    pub async fn list_server_user_data(
        &self,
        zone: &str,
        server_id: &str,
    ) -> Result<serde_json::Value, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/user_data",
            INSTANCE_API_URL, zone, server_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: serde_json::Value = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Get user data value for a key
    pub async fn get_server_user_data(
        &self,
        zone: &str,
        server_id: &str,
        key: &str,
    ) -> Result<String, Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/user_data/{}",
            INSTANCE_API_URL, zone, server_id, key
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response = res.text().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Set user data value for a key
    pub async fn set_server_user_data(
        &self,
        zone: &str,
        server_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/user_data/{}",
            INSTANCE_API_URL, zone, server_id, key
        );

        let res = self
            .http_client
            .patch(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .header("Content-Type", "text/plain")
            .body(value.to_string())
            .send()
            .await?;

        check_api_error(res).await?;
        Ok(())
    }

    /// Delete user data key
    pub async fn delete_server_user_data(
        &self,
        zone: &str,
        server_id: &str,
        key: &str,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/zones/{}/servers/{}/user_data/{}",
            INSTANCE_API_URL, zone, server_id, key
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
}
