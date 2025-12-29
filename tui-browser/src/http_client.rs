use anyhow::{Context, Result};
use reqwest::Client;
use tokio::runtime::Runtime;

pub struct HttpClient {
    client: Client,
    runtime: Runtime,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("TUI-Browser/0.1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;
        
        let runtime = Runtime::new().context("Failed to create tokio runtime")?;
        
        Ok(Self { client, runtime })
    }

    pub fn fetch_page(&self, url: &str) -> Result<String> {
        self.runtime.block_on(async {
            let response = self.client
                .get(url)
                .send()
                .await
                .context("Failed to send request")?;
            
            let content = response
                .text()
                .await
                .context("Failed to read response body")?;
            
            Ok(content)
        })
    }

    pub fn render_html_to_text(&self, html: &str) -> String {
        html2text::from_read(html.as_bytes(), 120)
    }
}
