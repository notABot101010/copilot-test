use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};

pub mod js_runtime;

pub use js_runtime::ChallengeSolver;

const STATUS_URL: &str = "https://duckduckgo.com/duckchat/v1/status";
const CHAT_URL: &str = "https://duckduckgo.com/duckchat/v1/chat";

// Static headers based on reverse engineering
const X_FE_SIGNALS: &str = "eyJzdGFydCI6MTc1MjE1NTc3NzQ4MCwiZXZlbnRzIjpbeyJuYW1lIjoic3RhcnROZXdDaGF0IiwiZGVsdGEiOjc1fSx7Im5hbWUiOiJyZWNlbnRDaGF0c0xpc3RJbXByZXNzaW9uIiwiZGVsdGEiOjEyNH1dLCJlbmQiOjQzNDN9";
const X_FE_VERSION: &str = "serp_20250710_090702_ET-70eaca6aea2948b0bb60";
const USER_AGENT_STRING: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
struct ToolChoice {
    #[serde(rename = "NewsSearch")]
    news_search: bool,
    #[serde(rename = "VideosSearch")]
    videos_search: bool,
    #[serde(rename = "LocalSearch")]
    local_search: bool,
    #[serde(rename = "WeatherForecast")]
    weather_forecast: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Metadata {
    #[serde(rename = "toolChoice")]
    tool_choice: ToolChoice,
}

#[derive(Debug, Clone, Serialize)]
struct ChatRequest {
    model: String,
    metadata: Metadata,
    messages: Vec<ChatMessage>,
    #[serde(rename = "canUseTools")]
    can_use_tools: bool,
    #[serde(rename = "canUseApproxLocation")]
    can_use_approx_location: bool,
}

#[derive(Debug, Clone)]
pub struct DuckDuckGoClient {
    client: reqwest::Client,
    vqd: Option<String>,
}

impl DuckDuckGoClient {
    pub fn new() -> Result<Self> {
        let jar = reqwest::cookie::Jar::default();

        let client = reqwest::Client::builder()
            .cookie_provider(jar.into())
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, vqd: None })
    }

    fn build_headers(&self, vqd: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STRING));
        headers.insert("x-vqd-hash-1", HeaderValue::from_str(vqd)?);
        headers.insert("x-fe-signals", HeaderValue::from_static(X_FE_SIGNALS));
        headers.insert("x-fe-version", HeaderValue::from_static(X_FE_VERSION));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("Accept", HeaderValue::from_static("text/event-stream"));
        headers.insert(
            "Sec-CH-UA",
            HeaderValue::from_static(
                "\"Chromium\";v=\"142\", \"Not=A?Brand\";v=\"24\", \"Google Chrome\";v=\"142\"",
            ),
        );
        headers.insert("Sec-CH-UA-Mobile", HeaderValue::from_static("?0"));
        headers.insert("Sec-CH-UA-Platform", HeaderValue::from_static("\"macOS\""));
        headers.insert("Referer", HeaderValue::from_static("https://duckduckgo.com/"));

        Ok(headers)
    }

    fn build_status_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STRING));
        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate, br, zstd"));
        headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.7"));
        headers.insert("Cache-Control", HeaderValue::from_static("no-store"));
        headers.insert("Priority", HeaderValue::from_static("u=1, i"));
        headers.insert("Referer", HeaderValue::from_static("https://duckduckgo.com/"));
        headers.insert("Sec-CH-UA", HeaderValue::from_static("\"Chromium\";v=\"142\", \"Brave\";v=\"142\", \"Not_A Brand\";v=\"99\""));
        headers.insert("Sec-CH-UA-Mobile", HeaderValue::from_static("?0"));
        headers.insert("Sec-CH-UA-Platform", HeaderValue::from_static("\"macOS\""));
        headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
        headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
        headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
        headers.insert("Sec-GPC", HeaderValue::from_static("1"));
        headers.insert("x-vqd-accept", HeaderValue::from_static("1"));

        Ok(headers)
    }

    async fn fetch_vqd(&mut self) -> Result<String> {
        tracing::debug!("Fetching VQD token from status endpoint");

        let response = self
            .client
            .get(STATUS_URL)
            .headers(self.build_status_headers()?)
            .send()
            .await
            .context("Failed to fetch status")?;

        if !response.status().is_success() {
            return Err(anyhow!("Status endpoint returned {}", response.status()));
        }

        // Extract challenge from x-vqd-hash-1 header
        let challenge = response
            .headers()
            .get("x-vqd-hash-1")
            .ok_or_else(|| anyhow!("x-vqd-hash-1 header is missing"))?
            .to_str()
            .context("Failed to parse x-vqd-hash-1 header")?
            .to_string();

        tracing::debug!("Received challenge: {}", &challenge[..challenge.len().min(50)]);

        // Solve the challenge using the JavaScript runtime
        let solver = ChallengeSolver::new().context("Failed to create challenge solver")?;
        let vqd = solver
            .solve(&challenge)
            .await
            .context("Failed to solve VQD challenge")?;

        tracing::debug!("Solved VQD token: {}", &vqd[..vqd.len().min(50)]);
        self.vqd = Some(vqd.clone());

        Ok(vqd)
    }

    pub async fn chat(&mut self, message: &str, model: Option<&str>) -> Result<String> {
        // Retry logic for VQD token refresh
        for attempt in 0..3 {
            if attempt > 0 {
                tracing::debug!("Retry attempt {} after VQD refresh", attempt);
            }

            // Ensure we have a VQD token
            if self.vqd.is_none() {
                self.fetch_vqd().await?;
            }

            let vqd = self.vqd.as_ref().unwrap().clone();

            let request = ChatRequest {
                model: model.unwrap_or("gpt-4o-mini").to_string(),
                metadata: Metadata {
                    tool_choice: ToolChoice {
                        news_search: false,
                        videos_search: false,
                        local_search: false,
                        weather_forecast: false,
                    },
                },
                messages: vec![ChatMessage {
                    role: "user".to_string(),
                    content: message.to_string(),
                }],
                can_use_tools: true,
                can_use_approx_location: true,
            };

            tracing::debug!("Sending chat request with model: {}", request.model);

            let headers = self.build_headers(&vqd)?;

            let response = self
                .client
                .post(CHAT_URL)
                .headers(headers)
                .json(&request)
                .send()
                .await
                .context("Failed to send chat request")?;

            let status = response.status();

            if status == 418 || status == 429 {
                tracing::warn!("Received status {}, refreshing VQD token", status);
                self.fetch_vqd().await?;
                if attempt < 2 {
                    continue; // Retry
                }
                return Err(anyhow!("VQD token expired or rate limited after retries"));
            }

            if !status.is_success() {
                return Err(anyhow!("Chat endpoint returned {}", status));
            }

            // Parse SSE stream
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut full_response = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        let text = String::from_utf8_lossy(&chunk);
                        buffer.push_str(&text);

                        // Process complete SSE events
                        while let Some(event_end) = buffer.find("\n\n") {
                            let event = buffer[..event_end].to_string();
                            buffer.drain(..event_end + 2);

                            // Parse SSE event
                            for line in event.lines() {
                                if let Some(data) = line.strip_prefix("data: ") {
                                    if data == "[DONE]" {
                                        return Ok(full_response);
                                    }

                                    // Try to parse the message data as JSON
                                    if let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(data)
                                    {
                                        if let Some(message) = json.get("message") {
                                            if let Some(content) = message.as_str() {
                                                full_response.push_str(content);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        tracing::warn!("Stream error: {}", err);
                        break;
                    }
                }
            }

            return Ok(full_response);
        }

        Err(anyhow!("Failed to complete chat request after retries"))
    }

    pub async fn chat_stream<F>(
        &mut self,
        message: &str,
        model: Option<&str>,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(&str),
    {
        // Retry logic for VQD token refresh
        for attempt in 0..3 {
            if attempt > 0 {
                tracing::debug!("Retry attempt {} after VQD refresh", attempt);
            }

            // Ensure we have a VQD token
            if self.vqd.is_none() {
                self.fetch_vqd().await?;
            }

            let vqd = self.vqd.as_ref().unwrap().clone();

            let request = ChatRequest {
                model: model.unwrap_or("gpt-4o-mini").to_string(),
                metadata: Metadata {
                    tool_choice: ToolChoice {
                        news_search: false,
                        videos_search: false,
                        local_search: false,
                        weather_forecast: false,
                    },
                },
                messages: vec![ChatMessage {
                    role: "user".to_string(),
                    content: message.to_string(),
                }],
                can_use_tools: true,
                can_use_approx_location: true,
            };

            tracing::debug!(
                "Sending streaming chat request with model: {}",
                request.model
            );

            let headers = self.build_headers(&vqd)?;

            let response = self
                .client
                .post(CHAT_URL)
                .headers(headers)
                .json(&request)
                .send()
                .await
                .context("Failed to send chat request")?;

            let status = response.status();

            if status == 418 || status == 429 {
                tracing::warn!("Received status {}, refreshing VQD token", status);
                self.fetch_vqd().await?;
                if attempt < 2 {
                    continue; // Retry
                }
                return Err(anyhow!("VQD token expired or rate limited after retries"));
            }

            if !status.is_success() {
                return Err(anyhow!("Chat endpoint returned {}", status));
            }

            // Parse SSE stream
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        let text = String::from_utf8_lossy(&chunk);
                        buffer.push_str(&text);

                        // Process complete SSE events
                        while let Some(event_end) = buffer.find("\n\n") {
                            let event = buffer[..event_end].to_string();
                            buffer.drain(..event_end + 2);

                            // Parse SSE event
                            for line in event.lines() {
                                if let Some(data) = line.strip_prefix("data: ") {
                                    if data == "[DONE]" {
                                        return Ok(());
                                    }

                                    // Try to parse the message data as JSON
                                    if let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(data)
                                    {
                                        if let Some(message) = json.get("message") {
                                            if let Some(content) = message.as_str() {
                                                callback(content);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        tracing::warn!("Stream error: {}", err);
                        break;
                    }
                }
            }

            return Ok(());
        }

        Err(anyhow!(
            "Failed to complete streaming chat request after retries"
        ))
    }
}

impl Default for DuckDuckGoClient {
    fn default() -> Self {
        Self::new().expect("Failed to create DuckDuckGoClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = DuckDuckGoClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_vqd() {
        let mut client = DuckDuckGoClient::new().unwrap();
        let result = client.fetch_vqd().await;
        if let Err(ref err) = result {
            eprintln!("VQD fetch error: {}", err);
        }
        assert!(result.is_ok(), "Failed to fetch VQD: {:?}", result);
        assert!(client.vqd.is_some());
    }
}
