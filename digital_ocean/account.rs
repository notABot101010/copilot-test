//! Account API endpoints.
//!
//! Get information about your DigitalOcean account.
//! See: <https://docs.digitalocean.com/reference/api/api-reference/#tag/Account>

use serde::{Deserialize, Serialize};

use crate::{check_api_error, Client, Error, Url, API_BASE_URL};

/// Account information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Account {
    /// Droplet limit for the account.
    pub droplet_limit: u32,
    /// Floating IP limit for the account.
    pub floating_ip_limit: u32,
    /// Reserved IP limit for the account.
    pub reserved_ip_limit: Option<u32>,
    /// Volume limit for the account.
    pub volume_limit: u32,
    /// Email address associated with the account.
    pub email: String,
    /// Account UUID.
    pub uuid: String,
    /// Whether email has been verified.
    pub email_verified: bool,
    /// Account status.
    pub status: String,
    /// Status message.
    pub status_message: String,
    /// Team information.
    pub team: Option<Team>,
}

/// Team information for an account.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Team {
    /// Team UUID.
    pub uuid: String,
    /// Team name.
    pub name: String,
}

/// Response from getting account information.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountResponse {
    /// The account information.
    pub account: Account,
}

impl Client {
    /// Gets information about your account.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use digital_ocean::Client;
    /// # async fn example(client: Client) -> Result<(), digital_ocean::Error> {
    /// let response = client.get_account().await?;
    /// println!("Email: {}", response.account.email);
    /// println!("Droplet limit: {}", response.account.droplet_limit);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_account(&self) -> Result<AccountResponse, Error> {
        let url = Url::parse(&format!("{}/account", API_BASE_URL)).expect("Invalid URL");

        let res = self
            .http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        Ok(check_api_error(res).await?.json().await?)
    }
}
