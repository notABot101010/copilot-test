//! JavaScript runtime for solving DuckDuckGo VQD challenges
//!
//! This module provides a challenge solver for DuckDuckGo's AI chat API
//! using headless Chrome to execute the challenge JavaScript in a real browser context.

use anyhow::{Context, Result};
use aws_lc_rs::digest::{SHA256, digest};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use headless_chrome::{Browser, LaunchOptions};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Maximum length for debug output truncation
const DEBUG_TRUNCATE_LEN: usize = 200;

/// Structure representing the challenge data from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeData {
    pub server_hashes: Vec<String>,
    #[serde(default)]
    pub client_hashes: Vec<String>,
    #[serde(default)]
    pub signals: serde_json::Value,
    #[serde(default)]
    pub meta: ChallengeMeta,
}

/// Metadata for the challenge
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChallengeMeta {
    #[serde(default)]
    pub v: String,
    #[serde(default)]
    pub challenge_id: String,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub debug: String,
    #[serde(default)]
    pub origin: String,
    #[serde(default)]
    pub stack: String,
    #[serde(default)]
    pub duration: String,
}

/// The VQD challenge solver using headless Chrome
pub struct ChallengeSolver {
    browser: Browser,
}

impl ChallengeSolver {
    /// Create a new challenge solver with headless Chrome
    pub fn new() -> Result<Self> {
        let options = LaunchOptions::default_builder()
            .headless(true)
            .sandbox(false)  // Required for running in Docker/CI environments
            .build()
            .context("Failed to build launch options")?;
        
        let browser = Browser::new(options)
            .context("Failed to launch headless Chrome")?;
        
        Ok(Self { browser })
    }

    /// Solve a VQD challenge from the base64-encoded challenge string
    ///
    /// # Arguments
    /// * `challenge_b64` - The base64-encoded challenge from x-vqd-hash-1 header
    ///
    /// # Returns
    /// The solved X-Vqd-Hash-1 header value to use for chat requests
    pub async fn solve(&self, challenge_b64: &str) -> Result<String> {
        let start_time = Instant::now();

        // Decode the base64 challenge to get the JavaScript code
        let challenge_bytes = BASE64
            .decode(challenge_b64)
            .context("Failed to decode challenge base64")?;
        let challenge_code =
            String::from_utf8(challenge_bytes).context("Challenge is not valid UTF-8")?;

        tracing::debug!("Decoded challenge: {}", &challenge_code[..challenge_code.len().min(DEBUG_TRUNCATE_LEN)]);

        // Execute the challenge in a real browser context
        let challenge_result = self.execute_in_browser(&challenge_code)
            .context("Failed to execute challenge in browser")?;

        tracing::debug!("Challenge result from browser: {:?}", challenge_result);

        // Hash the client_hashes with SHA-256 and encode as base64
        let hashed_client_hashes: Vec<String> = challenge_result.client_hashes
            .iter()
            .map(|h| {
                let hash = digest(&SHA256, h.as_bytes());
                BASE64.encode(hash.as_ref())
            })
            .collect();

        let duration = start_time.elapsed().as_millis().to_string();

        // Build the final result
        let result = ChallengeData {
            server_hashes: challenge_result.server_hashes,
            client_hashes: hashed_client_hashes,
            signals: challenge_result.signals,
            meta: ChallengeMeta {
                v: challenge_result.meta.v,
                challenge_id: challenge_result.meta.challenge_id,
                timestamp: challenge_result.meta.timestamp,
                debug: challenge_result.meta.debug,
                origin: "https://duckduckgo.com".to_string(),
                stack: "Error\nat l (https://duckduckgo.com/dist/wpm.main.758b58e5295173a9d89c.js:1:424103)".to_string(),
                duration,
            },
        };

        // Encode the result as base64
        let result_json = serde_json::to_string(&result).context("Failed to serialize result")?;
        let result_b64 = BASE64.encode(result_json.as_bytes());

        tracing::debug!("Solved challenge, result length: {}", result_b64.len());

        Ok(result_b64)
    }

    /// Execute the challenge JavaScript in headless Chrome
    fn execute_in_browser(&self, challenge_code: &str) -> Result<ChallengeData> {
        let tab = self.browser.new_tab()
            .context("Failed to create new browser tab")?;

        // Navigate to a blank page first
        tab.navigate_to("about:blank")
            .context("Failed to navigate to blank page")?;
        tab.wait_until_navigated()
            .context("Failed to wait for navigation")?;

        // Create an HTML page with an iframe (like DuckDuckGo does)
        let html = format!(r#"
            <!DOCTYPE html>
            <html>
            <head><title>Challenge Solver</title></head>
            <body>
                <iframe id="jsa-frame" sandbox="allow-scripts allow-same-origin" srcdoc="DuckDuckGo Fraud &amp; Abuse"></iframe>
                <script>
                    window.__challengeResult = null;
                    window.__challengeError = null;
                    
                    (async function() {{
                        try {{
                            const result = await {challenge_code};
                            window.__challengeResult = JSON.stringify(result);
                        }} catch (e) {{
                            window.__challengeError = e.message || String(e);
                        }}
                    }})();
                </script>
            </body>
            </html>
        "#);

        // Set the page content
        let set_content_js = format!(
            r#"document.documentElement.innerHTML = `{}`;"#,
            html.replace('`', r"\`").replace("${", r"\${")
        );
        tab.evaluate(&set_content_js, false)
            .context("Failed to set page content")?;

        // Wait for the challenge to complete (poll for result)
        let timeout = Duration::from_secs(10);
        let start = Instant::now();
        
        loop {
            if start.elapsed() > timeout {
                return Err(anyhow::anyhow!("Challenge execution timed out"));
            }

            // Check for error first
            let error_check = tab.evaluate("window.__challengeError", false)
                .context("Failed to check for error")?;
            if let Some(error) = error_check.value {
                if !error.is_null() {
                    let error_str = error.as_str().unwrap_or("Unknown error");
                    return Err(anyhow::anyhow!("Challenge execution failed: {}", error_str));
                }
            }

            // Check for result
            let result_check = tab.evaluate("window.__challengeResult", false)
                .context("Failed to check for result")?;
            if let Some(result) = result_check.value {
                if !result.is_null() {
                    let result_str = result.as_str()
                        .ok_or_else(|| anyhow::anyhow!("Result is not a string"))?;
                    let challenge_data: ChallengeData = serde_json::from_str(result_str)
                        .context("Failed to parse challenge result as JSON")?;
                    return Ok(challenge_data);
                }
            }

            // Wait a bit before polling again
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Default for ChallengeSolver {
    fn default() -> Self {
        Self::new().expect("Failed to create ChallengeSolver")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_challenge_solver_creation() {
        let solver = ChallengeSolver::new();
        assert!(solver.is_ok());
    }

    #[tokio::test]
    async fn test_sha256_base64() {
        // Test that SHA256 + base64 encoding works correctly
        let input = "test";
        let hash = digest(&SHA256, input.as_bytes());
        let encoded = BASE64.encode(hash.as_ref());
        // SHA256 of "test" = n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg=
        assert_eq!(encoded, "n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg=");
    }
}
