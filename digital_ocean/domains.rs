//! Domains API endpoints.
//!
//! Manage domains and DNS records in DigitalOcean.
//! See: <https://docs.digitalocean.com/reference/api/api-reference/#tag/Domains>

use serde::{Deserialize, Serialize};

use crate::{check_api_error, Client, Error, Links, Meta, API_BASE_URL};

/// A domain registered in DigitalOcean DNS.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Domain {
    /// Domain name.
    pub name: String,
    /// TTL (Time To Live) for the domain records.
    pub ttl: Option<u32>,
    /// Zone file content.
    pub zone_file: Option<String>,
}

/// A DNS record for a domain.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DomainRecord {
    /// Unique identifier for the record.
    pub id: u64,
    /// Record type (A, AAAA, CNAME, MX, TXT, NS, SRV, CAA).
    #[serde(rename = "type")]
    pub record_type: String,
    /// Hostname or subdomain.
    pub name: String,
    /// Record value (IP address, hostname, etc.).
    pub data: String,
    /// Priority (for MX and SRV records).
    pub priority: Option<u32>,
    /// Port (for SRV records).
    pub port: Option<u32>,
    /// TTL in seconds.
    pub ttl: u32,
    /// Weight (for SRV records).
    pub weight: Option<u32>,
    /// Flags (for CAA records).
    pub flags: Option<u32>,
    /// Tag (for CAA records).
    pub tag: Option<String>,
}

/// Request to create a new domain.
#[derive(Debug, Clone, Serialize)]
pub struct CreateDomainRequest {
    /// Domain name to create.
    pub name: String,
    /// IP address to create an initial A record for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
}

/// Request to create a DNS record.
#[derive(Debug, Clone, Serialize)]
pub struct CreateDomainRecordRequest {
    /// Record type (A, AAAA, CNAME, MX, TXT, NS, SRV, CAA).
    #[serde(rename = "type")]
    pub record_type: String,
    /// Hostname or subdomain.
    pub name: String,
    /// Record value.
    pub data: String,
    /// Priority (for MX and SRV records).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    /// Port (for SRV records).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u32>,
    /// TTL in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
    /// Weight (for SRV records).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u32>,
    /// Flags (for CAA records).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
    /// Tag (for CAA records).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// Request to update a DNS record.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateDomainRecordRequest {
    /// Record type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub record_type: Option<String>,
    /// Hostname or subdomain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Record value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Priority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    /// Port.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u32>,
    /// TTL in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
    /// Weight.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u32>,
    /// Flags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
    /// Tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// Response from listing domains.
#[derive(Debug, Clone, Deserialize)]
pub struct ListDomainsResponse {
    /// List of domains.
    pub domains: Vec<Domain>,
    /// Pagination links.
    pub links: Option<Links>,
    /// Metadata about the response.
    pub meta: Option<Meta>,
}

/// Response from getting or creating a domain.
#[derive(Debug, Clone, Deserialize)]
pub struct DomainResponse {
    /// The domain.
    pub domain: Domain,
}

/// Response from listing domain records.
#[derive(Debug, Clone, Deserialize)]
pub struct ListDomainRecordsResponse {
    /// List of domain records.
    pub domain_records: Vec<DomainRecord>,
    /// Pagination links.
    pub links: Option<Links>,
    /// Metadata about the response.
    pub meta: Option<Meta>,
}

/// Response from getting or creating a domain record.
#[derive(Debug, Clone, Deserialize)]
pub struct DomainRecordResponse {
    /// The domain record.
    pub domain_record: DomainRecord,
}

impl Client {
    /// Lists all domains in your account.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::Client;
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let response = client.list_domains(None, None).await?;
    /// for domain in response.domains {
    ///     println!("{}", domain.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_domains(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<ListDomainsResponse, Error> {
        let mut url = format!("{}/domains", API_BASE_URL);
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

    /// Gets a specific domain by name.
    ///
    /// # Arguments
    ///
    /// * `domain_name` - The domain name.
    pub async fn get_domain(&self, domain_name: &str) -> Result<DomainResponse, Error> {
        let url = format!("{}/domains/{}", API_BASE_URL, domain_name);

        let res = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Creates a new domain.
    ///
    /// # Arguments
    ///
    /// * `request` - The domain creation request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::{Client, CreateDomainRequest};
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let request = CreateDomainRequest {
    ///     name: "example.com".to_string(),
    ///     ip_address: Some("1.2.3.4".to_string()),
    /// };
    /// let response = client.create_domain(request).await?;
    /// println!("Created domain: {}", response.domain.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_domain(&self, request: CreateDomainRequest) -> Result<DomainResponse, Error> {
        let url = format!("{}/domains", API_BASE_URL);

        let res = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&request)
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Deletes a domain.
    ///
    /// # Arguments
    ///
    /// * `domain_name` - The domain name to delete.
    pub async fn delete_domain(&self, domain_name: &str) -> Result<(), Error> {
        let url = format!("{}/domains/{}", API_BASE_URL, domain_name);

        let res = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        check_api_error(res).await?;
        Ok(())
    }

    /// Lists records for a domain.
    ///
    /// # Arguments
    ///
    /// * `domain_name` - The domain name.
    /// * `page` - Page number for pagination.
    /// * `per_page` - Number of items per page.
    pub async fn list_domain_records(
        &self,
        domain_name: &str,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<ListDomainRecordsResponse, Error> {
        let mut url = format!("{}/domains/{}/records", API_BASE_URL, domain_name);
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

    /// Gets a specific domain record.
    ///
    /// # Arguments
    ///
    /// * `domain_name` - The domain name.
    /// * `record_id` - The record ID.
    pub async fn get_domain_record(
        &self,
        domain_name: &str,
        record_id: u64,
    ) -> Result<DomainRecordResponse, Error> {
        let url = format!("{}/domains/{}/records/{}", API_BASE_URL, domain_name, record_id);

        let res = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Creates a domain record.
    ///
    /// # Arguments
    ///
    /// * `domain_name` - The domain name.
    /// * `request` - The record creation request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::{Client, CreateDomainRecordRequest};
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let request = CreateDomainRecordRequest {
    ///     record_type: "A".to_string(),
    ///     name: "www".to_string(),
    ///     data: "1.2.3.4".to_string(),
    ///     priority: None,
    ///     port: None,
    ///     ttl: Some(3600),
    ///     weight: None,
    ///     flags: None,
    ///     tag: None,
    /// };
    /// let response = client.create_domain_record("example.com", request).await?;
    /// println!("Created record: {}", response.domain_record.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_domain_record(
        &self,
        domain_name: &str,
        request: CreateDomainRecordRequest,
    ) -> Result<DomainRecordResponse, Error> {
        let url = format!("{}/domains/{}/records", API_BASE_URL, domain_name);

        let res = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&request)
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Updates a domain record.
    ///
    /// # Arguments
    ///
    /// * `domain_name` - The domain name.
    /// * `record_id` - The record ID.
    /// * `request` - The record update request.
    pub async fn update_domain_record(
        &self,
        domain_name: &str,
        record_id: u64,
        request: UpdateDomainRecordRequest,
    ) -> Result<DomainRecordResponse, Error> {
        let url = format!("{}/domains/{}/records/{}", API_BASE_URL, domain_name, record_id);

        let res = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&request)
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }

    /// Deletes a domain record.
    ///
    /// # Arguments
    ///
    /// * `domain_name` - The domain name.
    /// * `record_id` - The record ID.
    pub async fn delete_domain_record(&self, domain_name: &str, record_id: u64) -> Result<(), Error> {
        let url = format!("{}/domains/{}/records/{}", API_BASE_URL, domain_name, record_id);

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
