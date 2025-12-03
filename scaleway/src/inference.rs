//! Managed Inference API
//!
//! Scaleway Managed Inference allows you to deploy and manage machine learning models.

use crate::client::{check_api_error, Client, Error};
use reqwest::Url;
use serde::{Deserialize, Serialize};

const INFERENCE_API_URL: &str = "https://api.scaleway.com/inference/v1";

// ============================================================================
// Types
// ============================================================================

/// Deployment endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentEndpoint {
    /// Endpoint ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Disable authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_auth: Option<bool>,
    /// Public endpoint configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<serde_json::Value>,
    /// Private network configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_network: Option<PrivateNetworkConfig>,
}

/// Private network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateNetworkConfig {
    /// Private network ID
    pub private_network_id: String,
}

/// Deployment information
#[derive(Debug, Clone, Deserialize)]
pub struct Deployment {
    /// Deployment ID
    pub id: String,
    /// Deployment name
    pub name: String,
    /// Project ID
    pub project_id: String,
    /// Status
    pub status: String,
    /// Model ID
    pub model_id: String,
    /// Model name
    #[serde(default)]
    pub model_name: String,
    /// Node type
    pub node_type: String,
    /// Minimum number of nodes
    pub min_size: u32,
    /// Maximum number of nodes
    pub max_size: u32,
    /// Current size
    #[serde(default)]
    pub size: u32,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Endpoints
    #[serde(default)]
    pub endpoints: Vec<DeploymentEndpoint>,
    /// Region
    pub region: String,
    /// Creation date
    pub created_at: Option<String>,
    /// Modification date
    pub updated_at: Option<String>,
}

/// List deployments response
#[derive(Debug, Clone, Deserialize)]
pub struct ListDeploymentsResponse {
    /// List of deployments
    pub deployments: Vec<Deployment>,
    /// Total count
    pub total_count: u64,
}

/// Create deployment request
#[derive(Debug, Clone, Serialize)]
pub struct CreateDeploymentRequest {
    /// Project ID
    pub project_id: String,
    /// Deployment name
    pub name: String,
    /// Model ID
    pub model_id: String,
    /// Node type
    pub node_type: String,
    /// Minimum number of nodes
    pub min_size: u32,
    /// Maximum number of nodes
    pub max_size: u32,
    /// Accept EULA
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_eula: Option<bool>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Endpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<DeploymentEndpoint>>,
}

/// Update deployment request
#[derive(Debug, Clone, Serialize)]
pub struct UpdateDeploymentRequest {
    /// Deployment name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Minimum number of nodes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_size: Option<u32>,
    /// Maximum number of nodes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size: Option<u32>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Deployment certificate
#[derive(Debug, Clone, Deserialize)]
pub struct DeploymentCertificate {
    /// Certificate in PEM format
    pub certificate: String,
}

/// Model information
#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    /// Model ID
    pub id: String,
    /// Model name
    pub name: String,
    /// Project ID
    #[serde(default)]
    pub project_id: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Status
    pub status: String,
    /// Size in bytes
    #[serde(default)]
    pub size: u64,
    /// Has EULA
    #[serde(default)]
    pub has_eula: bool,
    /// Region
    pub region: String,
    /// Creation date
    pub created_at: Option<String>,
    /// Modification date
    pub updated_at: Option<String>,
    /// Nodes types compatible with this model
    #[serde(default)]
    pub compatible_node_types: Vec<String>,
}

/// List models response
#[derive(Debug, Clone, Deserialize)]
pub struct ListModelsResponse {
    /// List of models
    pub models: Vec<Model>,
    /// Total count
    pub total_count: u64,
}

/// Create model request
#[derive(Debug, Clone, Serialize)]
pub struct CreateModelRequest {
    /// Project ID
    pub project_id: String,
    /// Model name
    pub name: String,
    /// S3 model object reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3_model: Option<S3Model>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// S3 model reference
#[derive(Debug, Clone, Serialize)]
pub struct S3Model {
    /// S3 URI
    pub s3_uri: String,
}

/// Model EULA
#[derive(Debug, Clone, Deserialize)]
pub struct ModelEula {
    /// EULA content
    pub content: String,
}

/// Node type information
#[derive(Debug, Clone, Deserialize)]
pub struct NodeType {
    /// Node type name
    pub name: String,
    /// Stock status
    pub stock_status: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// VRAM in bytes
    #[serde(default)]
    pub vram: u64,
    /// GPU count
    #[serde(default)]
    pub gpus: u32,
}

/// List node types response
#[derive(Debug, Clone, Deserialize)]
pub struct ListNodeTypesResponse {
    /// List of node types
    pub node_types: Vec<NodeType>,
    /// Total count
    pub total_count: u64,
}

/// Create endpoint request
#[derive(Debug, Clone, Serialize)]
pub struct CreateEndpointRequest {
    /// Deployment ID
    pub deployment_id: String,
    /// Endpoint configuration
    pub endpoint: DeploymentEndpoint,
}

/// Update endpoint request
#[derive(Debug, Clone, Serialize)]
pub struct UpdateEndpointRequest {
    /// Disable authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_auth: Option<bool>,
}

/// Endpoint response
#[derive(Debug, Clone, Deserialize)]
pub struct EndpointResponse {
    /// Endpoint
    pub endpoint: DeploymentEndpoint,
}

/// Verify model request
#[derive(Debug, Clone, Serialize)]
pub struct VerifyModelRequest {
    /// Project ID
    pub project_id: String,
    /// S3 model reference
    pub s3_model: S3Model,
}

/// Verify model response
#[derive(Debug, Clone, Deserialize)]
pub struct VerifyModelResponse {
    /// Whether the model is valid
    pub valid: bool,
    /// Error message if invalid
    #[serde(default)]
    pub error: String,
}

// ============================================================================
// Managed Inference API Implementation
// ============================================================================

impl Client {
    // ========================================================================
    // Deployments
    // ========================================================================

    /// List deployments
    pub async fn list_deployments(
        &self,
        region: &str,
        project_id: Option<&str>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<ListDeploymentsResponse, Error> {
        let mut url = Url::parse(&format!(
            "{}/regions/{}/deployments",
            INFERENCE_API_URL, region
        ))
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
        let response: ListDeploymentsResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create a new deployment
    pub async fn create_deployment(
        &self,
        region: &str,
        request: CreateDeploymentRequest,
    ) -> Result<Deployment, Error> {
        let url = format!("{}/regions/{}/deployments", INFERENCE_API_URL, region);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let deployment: Deployment = res.json().await.map_err(Error::Json)?;
        Ok(deployment)
    }

    /// Get a deployment by ID
    pub async fn get_deployment(
        &self,
        region: &str,
        deployment_id: &str,
    ) -> Result<Deployment, Error> {
        let url = format!(
            "{}/regions/{}/deployments/{}",
            INFERENCE_API_URL, region, deployment_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let deployment: Deployment = res.json().await.map_err(Error::Json)?;
        Ok(deployment)
    }

    /// Update a deployment
    pub async fn update_deployment(
        &self,
        region: &str,
        deployment_id: &str,
        request: UpdateDeploymentRequest,
    ) -> Result<Deployment, Error> {
        let url = format!(
            "{}/regions/{}/deployments/{}",
            INFERENCE_API_URL, region, deployment_id
        );

        let res = self
            .http_client
            .patch(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let deployment: Deployment = res.json().await.map_err(Error::Json)?;
        Ok(deployment)
    }

    /// Delete a deployment
    pub async fn delete_deployment(
        &self,
        region: &str,
        deployment_id: &str,
    ) -> Result<Deployment, Error> {
        let url = format!(
            "{}/regions/{}/deployments/{}",
            INFERENCE_API_URL, region, deployment_id
        );

        let res = self
            .http_client
            .delete(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let deployment: Deployment = res.json().await.map_err(Error::Json)?;
        Ok(deployment)
    }

    /// Get deployment certificate
    pub async fn get_deployment_certificate(
        &self,
        region: &str,
        deployment_id: &str,
    ) -> Result<DeploymentCertificate, Error> {
        let url = format!(
            "{}/regions/{}/deployments/{}/certificate",
            INFERENCE_API_URL, region, deployment_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let certificate: DeploymentCertificate = res.json().await.map_err(Error::Json)?;
        Ok(certificate)
    }

    // ========================================================================
    // Endpoints
    // ========================================================================

    /// Create an endpoint for a deployment
    pub async fn create_inference_endpoint(
        &self,
        region: &str,
        request: CreateEndpointRequest,
    ) -> Result<EndpointResponse, Error> {
        let url = format!("{}/regions/{}/endpoints", INFERENCE_API_URL, region);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: EndpointResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Update an endpoint
    pub async fn update_inference_endpoint(
        &self,
        region: &str,
        endpoint_id: &str,
        request: UpdateEndpointRequest,
    ) -> Result<EndpointResponse, Error> {
        let url = format!(
            "{}/regions/{}/endpoints/{}",
            INFERENCE_API_URL, region, endpoint_id
        );

        let res = self
            .http_client
            .patch(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: EndpointResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Delete an endpoint
    pub async fn delete_inference_endpoint(
        &self,
        region: &str,
        endpoint_id: &str,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/regions/{}/endpoints/{}",
            INFERENCE_API_URL, region, endpoint_id
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
    // Models
    // ========================================================================

    /// List models
    pub async fn list_models(
        &self,
        region: &str,
        project_id: Option<&str>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<ListModelsResponse, Error> {
        let mut url = Url::parse(&format!("{}/regions/{}/models", INFERENCE_API_URL, region))
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
        let response: ListModelsResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    /// Create a model
    pub async fn create_model(
        &self,
        region: &str,
        request: CreateModelRequest,
    ) -> Result<Model, Error> {
        let url = format!("{}/regions/{}/models", INFERENCE_API_URL, region);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let model: Model = res.json().await.map_err(Error::Json)?;
        Ok(model)
    }

    /// Get a model by ID
    pub async fn get_model(&self, region: &str, model_id: &str) -> Result<Model, Error> {
        let url = format!(
            "{}/regions/{}/models/{}",
            INFERENCE_API_URL, region, model_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let model: Model = res.json().await.map_err(Error::Json)?;
        Ok(model)
    }

    /// Delete a model
    pub async fn delete_model(&self, region: &str, model_id: &str) -> Result<Model, Error> {
        let url = format!(
            "{}/regions/{}/models/{}",
            INFERENCE_API_URL, region, model_id
        );

        let res = self
            .http_client
            .delete(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let model: Model = res.json().await.map_err(Error::Json)?;
        Ok(model)
    }

    /// Get model EULA
    pub async fn get_model_eula(&self, region: &str, model_id: &str) -> Result<ModelEula, Error> {
        let url = format!(
            "{}/regions/{}/models/{}/eula",
            INFERENCE_API_URL, region, model_id
        );

        let res = self
            .http_client
            .get(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let eula: ModelEula = res.json().await.map_err(Error::Json)?;
        Ok(eula)
    }

    // ========================================================================
    // Node Types
    // ========================================================================

    /// List available node types
    pub async fn list_inference_node_types(
        &self,
        region: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<ListNodeTypesResponse, Error> {
        let mut url = Url::parse(&format!(
            "{}/regions/{}/node-types",
            INFERENCE_API_URL, region
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
        let response: ListNodeTypesResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }

    // ========================================================================
    // Model Verification
    // ========================================================================

    /// Verify a model
    pub async fn verify_model(
        &self,
        region: &str,
        request: VerifyModelRequest,
    ) -> Result<VerifyModelResponse, Error> {
        let url = format!("{}/regions/{}/verify-model", INFERENCE_API_URL, region);

        let res = self
            .http_client
            .post(&url)
            .header("X-Auth-Token", &self.secret_access_key)
            .json(&request)
            .send()
            .await?;

        let res = check_api_error(res).await?;
        let response: VerifyModelResponse = res.json().await.map_err(Error::Json)?;
        Ok(response)
    }
}
