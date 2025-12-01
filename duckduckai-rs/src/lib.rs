use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};

const STATUS_URL: &str = "https://duckduckgo.com/duckchat/v1/status";
const CHAT_URL: &str = "https://duckduckgo.com/duckchat/v1/chat";

// Static headers based on reverse engineering
// const X_VQD_HASH_1: &str = "eyJzZXJ2ZXJfaGFzaGVzIjpbIkdKTVJYUzNNeklVNnoxdnBrcFVBaWIxQkdWS2FHN1NnYTBkcVhMSndieUU9IiwidG5samExTVY5N1hCYVNBZ1Q4VjhnT3pEd0RtSnpJS2w1cnhpNVpnaUluVT0iLCJodFJOMnh5K3BjT0JVMXg1WGZ6bE42d0Q4SFQxZDErd2hreHNibjRFUnlFPSJdLCJjbGllbnRfaGFzaGVzIjpbInhISm9VY1JQU2xVZWtHdnBVZENadkRPZmVONk5EeCtrWFM0bm1xdysvdjg9IiwiRWtaQXZ5ZVNvTTNPTEFUaE15YldlL0FUdXNiT1ZHWVdzRWlJNThUbWhqRT0iLCIzTXM4VXVmSG4zQXo0ZW9HNnQrUy9aQy8xN0MxYzMzQXdsRWFEaFk0Y3ZVPSJdLCJzaWduYWxzIjp7fSwibWV0YSI6eyJ2IjoiNCIsImNoYWxsZW5nZV9pZCI6IjVmMGE3ZWJiYTI1ZWQxYjFhZWRlYmE5NjY5YmVmYjBjMTUwNDc0ODc5MTNmZWM4Yzk5MjhkMDljMDE5MDdhYjNweGp6ciIsInRpbWVzdGFtcCI6IjE3NjQ1NjI1MDU0MzEiLCJkZWJ1ZyI6Ilx1MDAxZkgiLCJvcmlnaW4iOiJodHRwczovL2R1Y2tkdWNrZ28uY29tIiwic3RhY2siOiJFcnJvclxuYXQgbCAoaHR0cHM6Ly9kdWNrZHVja2dvLmNvbS9kaXN0L3dwbS5tYWluLjc1OGI1OGU1Mjk1MTczYTlkODljLmpzOjE6NDI0MTAzKVxuYXQgYXN5bmMgaHR0cHM6Ly9kdWNrZHVja2dvLmNvbS9kaXN0L3dwbS5tYWluLjc1OGI1OGU1Mjk1MTczYTlkODljLmpzOjE6MzU2MTY0IiwiZHVyYXRpb24iOiIxNCJ9fQ==";
const X_FE_SIGNALS: &str = "eyJzdGFydCI6MTc1MjE1NTc3NzQ4MCwiZXZlbnRzIjpbeyJuYW1lIjoic3RhcnROZXdDaGF0IiwiZGVsdGEiOjc1fSx7Im5hbWUiOiJyZWNlbnRDaGF0c0xpc3RJbXByZXNzaW9uIiwiZGVsdGEiOjEyNH1dLCJlbmQiOjQzNDN9";
const X_FE_VERSION: &str = "serp_20250710_090702_ET-70eaca6aea2948b0bb60";
const USER_AGENT_STRING: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36";
// Initial VQD value (will be updated from responses)
// const INITIAL_VQD: &str = "eyJzZXJ2ZXJfaGFzaGVzIjpbIkdKTVJYUzNNeklVNnoxdnBrcFVBaWIxQkdWS2FHN1NnYTBkcVhMSndieUU9IiwidG5samExTVY5N1hCYVNBZ1Q4VjhnT3pEd0RtSnpJS2w1cnhpNVpnaUluVT0iLCJodFJOMnh5K3BjT0JVMXg1WGZ6bE42d0Q4SFQxZDErd2hreHNibjRFUnlFPSJdLCJjbGllbnRfaGFzaGVzIjpbInhISm9VY1JQU2xVZWtHdnBVZENadkRPZmVONk5EeCtrWFM0bm1xdysvdjg9IiwiRWtaQXZ5ZVNvTTNPTEFUaE15YldlL0FUdXNiT1ZHWVdzRWlJNThUbWhqRT0iLCIzTXM4VXVmSG4zQXo0ZW9HNnQrUy9aQy8xN0MxYzMzQXdsRWFEaFk0Y3ZVPSJdLCJzaWduYWxzIjp7fSwibWV0YSI6eyJ2IjoiNCIsImNoYWxsZW5nZV9pZCI6IjVmMGE3ZWJiYTI1ZWQxYjFhZWRlYmE5NjY5YmVmYjBjMTUwNDc0ODc5MTNmZWM4Yzk5MjhkMDljMDE5MDdhYjNweGp6ciIsInRpbWVzdGFtcCI6IjE3NjQ1NjI1MDU0MzEiLCJkZWJ1ZyI6Ilx1MDAxZkgiLCJvcmlnaW4iOiJodHRwczovL2R1Y2tkdWNrZ28uY29tIiwic3RhY2siOiJFcnJvclxuYXQgbCAoaHR0cHM6Ly9kdWNrZHVja2dvLmNvbS9kaXN0L3dwbS5tYWluLjc1OGI1OGU1Mjk1MTczYTlkODljLmpzOjE6NDI0MTAzKVxuYXQgYXN5bmMgaHR0cHM6Ly9kdWNrZHVja2dvLmNvbS9kaXN0L3dwbS5tYWluLjc1OGI1OGU1Mjk1MTczYTlkODljLmpzOjE6MzU2MTY0IiwiZHVyYXRpb24iOiIxNCJ9fQ==";

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

        // Add required cookies
        // jar.add_cookie_str("5=1", &STATUS_URL.parse()?);
        // jar.add_cookie_str("dcm=3", &STATUS_URL.parse()?);
        // jar.add_cookie_str("dcs=1", &STATUS_URL.parse()?);

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
        // headers.insert("x-vqd-4", HeaderValue::from_str(vqd)?);
        // headers.insert("x-vqd-hash-1", HeaderValue::from_static(X_VQD_HASH_1));
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
            // .header("x-vqd-accept","1")
            // .header("x-vqd-hash-1", X_VQD_HASH_1)
            .send()
            .await
            .context("Failed to fetch status")?;

        if !response.status().is_success() {
            return Err(anyhow!("Status endpoint returned {}", response.status()));
        }

        // println!("{:?}", response.headers());

        // Try to extract VQD token from response headers
        // let vqd = response
        //     .headers()
        //     .get("x-vqd-hash-1")
        //     .ok_or(anyhow!("x-vqd-hash-1 header is missing"))?
        //     .to_str()
        //     .context("Failed to parse x-vqd-hash-1 header")?
        //     .to_string();
        let vqd = "eyJzZXJ2ZXJfaGFzaGVzIjpbIkQ2dHcxQ1RTZnpmWHhaQXp1cThGVFd3WWhOTys3SUxqMTY3YnVDZ0Ewa0U9IiwiellUS1dhQ29hcURnOEVicEt2aHVqN0o3bWJqRUI4YVUwK2t6MXpFbmk1Yz0iLCI0cStTUUMrZ3pZdFNUeWU4YVZVRWw5dFZPWWpTRFdlR3JsbmxzenIxeWZvPSJdLCJjbGllbnRfaGFzaGVzIjpbInhISm9VY1JQU2xVZWtHdnBVZENadkRPZmVONk5EeCtrWFM0bm1xdysvdjg9IiwiQVBQS0xzaGtBWDV3TWlnWG5iU1BJVm9rSmdXUHhNbHdxdGd3dkc3WU0rUT0iLCJDYlZ2SWVQRU53ck1GYW5uYnRRR1QxRFFZSVcyTVBleTV6YllxUXMybVNNPSJdLCJzaWduYWxzIjp7fSwibWV0YSI6eyJ2IjoiNCIsImNoYWxsZW5nZV9pZCI6ImY0NTExNGZlZTA3MWZkODY5OTAwNDYwYzg3Y2IwM2VmZTBiNTc1OTYzMDc3NzQxNzFiOWNiNTgzNWViOTBkNmZweGp6ciIsInRpbWVzdGFtcCI6IjE3NjQ1NjM5ODQzNjAiLCJkZWJ1ZyI6Ilx1MDAxOVx1MDAxZiIsIm9yaWdpbiI6Imh0dHBzOi8vZHVja2R1Y2tnby5jb20iLCJzdGFjayI6IkVycm9yXG5hdCBsIChodHRwczovL2R1Y2tkdWNrZ28uY29tL2Rpc3Qvd3BtLm1haW4uNzU4YjU4ZTUyOTUxNzNhOWQ4OWMuanM6MTo0MjQxMDMpXG5hdCBhc3luYyBodHRwczovL2R1Y2tkdWNrZ28uY29tL2Rpc3Qvd3BtLm1haW4uNzU4YjU4ZTUyOTUxNzNhOWQ4OWMuanM6MTozNTYxNjQiLCJkdXJhdGlvbiI6IjkifX0=".to_string();

        tracing::debug!("Using VQD token: {}", vqd);
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

            // // Update VQD token from response headers if present
            // if let Some(new_vqd) = response.headers().get("x-vqd-hash-1") {
            //     if let Ok(new_vqd_str) = new_vqd.to_str() {
            //         tracing::debug!("Updating VQD token from response: {}", new_vqd_str);
            //         self.vqd = Some(new_vqd_str.to_string());
            //     }
            // }

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

            // // Update VQD token from response headers if present
            // if let Some(new_vqd) = response.headers().get("x-vqd-hash-1") {
            //     if let Ok(new_vqd_str) = new_vqd.to_str() {
            //         tracing::debug!("Updating VQD token from response: {}", new_vqd_str);
            //         self.vqd = Some(new_vqd_str.to_string());
            //     }
            // }

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
