//! Managed Databases API endpoints.
//!
//! Manage database clusters in DigitalOcean.
//! See: <https://docs.digitalocean.com/reference/api/api-reference/#tag/Databases>

use serde::{Deserialize, Serialize};

use crate::{check_api_error, Client, Error, Url, API_BASE_URL};

/// A managed database cluster.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseCluster {
    /// Unique identifier for the database cluster.
    pub id: String,
    /// Human-readable name for the cluster.
    pub name: String,
    /// Database engine (e.g., "pg", "mysql", "redis", "mongodb").
    pub engine: String,
    /// Database engine version.
    pub version: String,
    /// Connection information.
    pub connection: Option<DatabaseConnection>,
    /// Private connection information.
    pub private_connection: Option<DatabaseConnection>,
    /// List of database users.
    pub users: Option<Vec<DatabaseUser>>,
    /// List of databases in the cluster.
    pub db_names: Option<Vec<String>>,
    /// Number of nodes in the cluster.
    pub num_nodes: u32,
    /// Size slug for the nodes.
    pub size: String,
    /// Region slug where the cluster is deployed.
    pub region: String,
    /// Current status of the cluster.
    pub status: String,
    /// When the cluster was created.
    pub created_at: String,
    /// Maintenance window configuration.
    pub maintenance_window: Option<MaintenanceWindow>,
    /// Tags applied to the cluster.
    pub tags: Option<Vec<String>>,
    /// Private network UUID.
    pub private_network_uuid: Option<String>,
}

/// Database connection information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConnection {
    /// Connection URI.
    pub uri: String,
    /// Database name.
    pub database: String,
    /// Hostname.
    pub host: String,
    /// Port number.
    pub port: u16,
    /// Username.
    pub user: String,
    /// Password.
    pub password: String,
    /// Whether SSL is required.
    pub ssl: bool,
}

/// A database user.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseUser {
    /// Username.
    pub name: String,
    /// User role.
    pub role: Option<String>,
    /// User password.
    pub password: Option<String>,
}

/// Maintenance window configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MaintenanceWindow {
    /// Day of the week (e.g., "monday").
    pub day: String,
    /// Hour of the day (0-23).
    pub hour: String,
    /// Whether updates are pending.
    pub pending: Option<bool>,
    /// Description of pending updates.
    pub description: Option<Vec<String>>,
}

/// Request to create a new database cluster.
#[derive(Debug, Clone, Serialize)]
pub struct CreateDatabaseClusterRequest {
    /// Human-readable name for the cluster.
    pub name: String,
    /// Database engine (e.g., "pg", "mysql", "redis", "mongodb").
    pub engine: String,
    /// Engine version.
    pub version: String,
    /// Size slug for the nodes.
    pub size: String,
    /// Region slug where the cluster will be deployed.
    pub region: String,
    /// Number of nodes (minimum 1).
    pub num_nodes: u32,
    /// Tags to apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Private network UUID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_network_uuid: Option<String>,
}

/// Response from listing database clusters.
#[derive(Debug, Clone, Deserialize)]
pub struct ListDatabaseClustersResponse {
    /// List of database clusters.
    pub databases: Vec<DatabaseCluster>,
}

/// Response from getting or creating a single database cluster.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseClusterResponse {
    /// The database cluster.
    pub database: DatabaseCluster,
}

/// Request to create a database user.
#[derive(Debug, Clone, Serialize)]
pub struct CreateDatabaseUserRequest {
    /// Username for the new user.
    pub name: String,
}

/// Response from creating a database user.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseUserResponse {
    /// The database user.
    pub user: DatabaseUser,
}

/// Request to create a database.
#[derive(Debug, Clone, Serialize)]
pub struct CreateDatabaseRequest {
    /// Name for the new database.
    pub name: String,
}

/// A database within a cluster.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Database {
    /// Database name.
    pub name: String,
}

/// Response from creating or getting a database.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseResponse {
    /// The database.
    pub db: Database,
}

/// Response from listing databases.
#[derive(Debug, Clone, Deserialize)]
pub struct ListDatabasesResponse {
    /// List of databases.
    pub dbs: Vec<Database>,
}

/// Response from listing database users.
#[derive(Debug, Clone, Deserialize)]
pub struct ListDatabaseUsersResponse {
    /// List of users.
    pub users: Vec<DatabaseUser>,
}

impl Client {
    /// Lists all database clusters in your account.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::Client;
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let response = client.list_database_clusters().await?;
    /// for cluster in response.databases {
    ///     println!("{}: {} ({})", cluster.id, cluster.name, cluster.engine);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_database_clusters(&self) -> Result<ListDatabaseClustersResponse, Error> {
        let url = Url::parse(&format!("{}/databases", API_BASE_URL))?;

        let res = self
            .http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Gets a specific database cluster by ID.
    ///
    /// # Arguments
    ///
    /// * `cluster_id` - The ID of the database cluster.
    pub async fn get_database_cluster(
        &self,
        cluster_id: &str,
    ) -> Result<DatabaseClusterResponse, Error> {
        let url = Url::parse(&format!("{}/databases/{}", API_BASE_URL, cluster_id))?;

        let res = self
            .http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Creates a new database cluster.
    ///
    /// # Arguments
    ///
    /// * `request` - The database cluster creation request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::{Client, CreateDatabaseClusterRequest};
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let request = CreateDatabaseClusterRequest {
    ///     name: "my-database".to_string(),
    ///     engine: "pg".to_string(),
    ///     version: "15".to_string(),
    ///     size: "db-s-1vcpu-1gb".to_string(),
    ///     region: "nyc1".to_string(),
    ///     num_nodes: 1,
    ///     tags: Some(vec!["production".to_string()]),
    ///     private_network_uuid: None,
    /// };
    /// let response = client.create_database_cluster(request).await?;
    /// println!("Created database: {}", response.database.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_database_cluster(
        &self,
        request: CreateDatabaseClusterRequest,
    ) -> Result<DatabaseClusterResponse, Error> {
        let url = Url::parse(&format!("{}/databases", API_BASE_URL))?;

        let res = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&request)
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Deletes a database cluster.
    ///
    /// # Arguments
    ///
    /// * `cluster_id` - The ID of the database cluster to delete.
    pub async fn delete_database_cluster(&self, cluster_id: &str) -> Result<(), Error> {
        let url = Url::parse(&format!("{}/databases/{}", API_BASE_URL, cluster_id))?;

        let res = self
            .http_client
            .delete(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        check_api_error(res).await?;
        Ok(())
    }

    /// Lists users for a database cluster.
    ///
    /// # Arguments
    ///
    /// * `cluster_id` - The ID of the database cluster.
    pub async fn list_database_users(
        &self,
        cluster_id: &str,
    ) -> Result<ListDatabaseUsersResponse, Error> {
        let url = Url::parse(&format!("{}/databases/{}/users", API_BASE_URL, cluster_id))?;

        let res = self
            .http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Creates a user for a database cluster.
    ///
    /// # Arguments
    ///
    /// * `cluster_id` - The ID of the database cluster.
    /// * `request` - The user creation request.
    pub async fn create_database_user(
        &self,
        cluster_id: &str,
        request: CreateDatabaseUserRequest,
    ) -> Result<DatabaseUserResponse, Error> {
        let url = Url::parse(&format!("{}/databases/{}/users", API_BASE_URL, cluster_id))?;

        let res = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&request)
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Deletes a user from a database cluster.
    ///
    /// # Arguments
    ///
    /// * `cluster_id` - The ID of the database cluster.
    /// * `username` - The username to delete.
    pub async fn delete_database_user(
        &self,
        cluster_id: &str,
        username: &str,
    ) -> Result<(), Error> {
        let url = Url::parse(&format!("{}/databases/{}/users/{}", API_BASE_URL, cluster_id, username))?;

        let res = self
            .http_client
            .delete(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        check_api_error(res).await?;
        Ok(())
    }

    /// Lists databases in a cluster.
    ///
    /// # Arguments
    ///
    /// * `cluster_id` - The ID of the database cluster.
    pub async fn list_databases(&self, cluster_id: &str) -> Result<ListDatabasesResponse, Error> {
        let url = Url::parse(&format!("{}/databases/{}/dbs", API_BASE_URL, cluster_id))?;

        let res = self
            .http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Creates a database in a cluster.
    ///
    /// # Arguments
    ///
    /// * `cluster_id` - The ID of the database cluster.
    /// * `request` - The database creation request.
    pub async fn create_database(
        &self,
        cluster_id: &str,
        request: CreateDatabaseRequest,
    ) -> Result<DatabaseResponse, Error> {
        let url = Url::parse(&format!("{}/databases/{}/dbs", API_BASE_URL, cluster_id))?;

        let res = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&request)
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Deletes a database from a cluster.
    ///
    /// # Arguments
    ///
    /// * `cluster_id` - The ID of the database cluster.
    /// * `db_name` - The name of the database to delete.
    pub async fn delete_database(&self, cluster_id: &str, db_name: &str) -> Result<(), Error> {
        let url = Url::parse(&format!("{}/databases/{}/dbs/{}", API_BASE_URL, cluster_id, db_name))?;

        let res = self
            .http_client
            .delete(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        check_api_error(res).await?;
        Ok(())
    }
}
