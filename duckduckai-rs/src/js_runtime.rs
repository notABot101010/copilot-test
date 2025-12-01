//! VQD Token Fetcher for DuckDuckGo VQD challenges
//!
//! This module provides a VQD token fetcher for DuckDuckGo's AI chat API
//! using headless Chrome to execute the challenge JavaScript and solve it.

use anyhow::{Context, Result, anyhow};
use aws_lc_rs::digest::{SHA256, digest};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use headless_chrome::{Browser, LaunchOptions};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::time::{Duration, Instant};

/// Structure representing the challenge result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResult {
    pub server_hashes: Vec<String>,
    #[serde(default)]
    pub client_hashes: Vec<String>,
    #[serde(default)]
    pub signals: serde_json::Value,
    #[serde(default)]
    pub meta: ChallengeMeta,
}

/// Metadata for the challenge result
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
    /// Create a new VQD challenge solver with headless Chrome
    pub fn new() -> Result<Self> {
        // Set up Chrome with options to avoid bot detection
        let args: Vec<&OsStr> = vec![
            // Avoid detection
            OsStr::new("--disable-blink-features=AutomationControlled"),
            OsStr::new("--disable-infobars"),
            OsStr::new("--disable-extensions"),
            OsStr::new("--disable-dev-shm-usage"),
            OsStr::new("--no-first-run"),
            OsStr::new("--no-default-browser-check"),
            // Realistic user agent
            OsStr::new(
                "--user-agent=Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
            ),
        ];

        let options = LaunchOptions::default_builder()
            .headless(true)
            .sandbox(false) // Required for running in Docker/CI environments
            .args(args)
            .build()
            .context("Failed to build launch options")?;

        let browser = Browser::new(options).context("Failed to launch headless Chrome")?;

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

        tracing::debug!(
            "Decoded challenge: {}",
            &challenge_code[..challenge_code.len().min(100)]
        );

        // Execute the challenge in a browser context that mimics DuckDuckGo
        let challenge_result = self
            .execute_challenge(&challenge_code)
            .context("Failed to execute challenge in browser")?;

        tracing::debug!("Challenge result from browser: {:?}", challenge_result);

        // Hash the client_hashes with SHA-256 and encode as base64
        let hashed_client_hashes: Vec<String> = challenge_result
            .client_hashes
            .iter()
            .map(|h| {
                let hash = digest(&SHA256, h.as_bytes());
                BASE64.encode(hash.as_ref())
            })
            .collect();

        let duration = start_time.elapsed().as_millis().to_string();

        // Build the final result
        let result = ChallengeResult {
            server_hashes: challenge_result.server_hashes,
            client_hashes: hashed_client_hashes,
            signals: challenge_result.signals,
            meta: ChallengeMeta {
                v: challenge_result.meta.v,
                challenge_id: challenge_result.meta.challenge_id,
                timestamp: challenge_result.meta.timestamp,
                debug: challenge_result.meta.debug,
                origin: "https://duckduckgo.com".to_string(),
                stack: "Error\nat l (https://duckduckgo.com/dist/wpm.main.js:1:424103)".to_string(),
                duration,
            },
        };

        // Encode the result as base64
        let result_json = serde_json::to_string(&result).context("Failed to serialize result")?;
        let result_b64 = BASE64.encode(result_json.as_bytes());

        tracing::debug!("Solved challenge, result length: {}", result_b64.len());

        Ok(result_b64)
    }

    /// Execute the challenge JavaScript in a browser with a minimal DuckDuckGo-like environment
    fn execute_challenge(&self, challenge_code: &str) -> Result<ChallengeResult> {
        let tab = self
            .browser
            .new_tab()
            .context("Failed to create new browser tab")?;

        // Navigate to about:blank first
        tab.navigate_to("about:blank")
            .context("Failed to navigate to about:blank")?;
        tab.wait_until_navigated()
            .context("Failed to wait for navigation")?;

        // Set up the environment to simulate DuckDuckGo's wpm.main.js behavior
        // The challenge code uses document.getElementById('jsa-frame') to find an iframe
        let setup_script = r#"
            // Set up result variables
            window.__challengeResult = null;
            window.__challengeError = null;
            window.__iframeReady = false;
            
            // Set up __jsaCallbacks like DuckDuckGo does
            window.__jsaCallbacks = {};
            
            // Create an iframe with the expected ID that the challenge code looks for
            const iframe = document.createElement('iframe');
            iframe.id = 'jsa-frame';
            iframe.sandbox = 'allow-scripts allow-same-origin';
            iframe.style.cssText = 'position:absolute;width:1px;height:1px;left:-9999px';
            iframe.srcdoc = '<!DOCTYPE html><html><head></head><body></body></html>';
            document.body.appendChild(iframe);
            
            // Set ready flag when iframe loads
            iframe.onload = () => { window.__iframeReady = true; };
            if (iframe.contentDocument && iframe.contentDocument.readyState === 'complete') {
                window.__iframeReady = true;
            }
        "#;

        tab.evaluate(setup_script, false)
            .context("Failed to set up challenge environment")?;

        // Wait for the iframe to be ready using polling
        let iframe_timeout = Duration::from_secs(5);
        let start = Instant::now();
        loop {
            if start.elapsed() > iframe_timeout {
                tracing::warn!("Timeout waiting for iframe to be ready, proceeding anyway");
                break;
            }

            let ready_check = tab.evaluate("window.__iframeReady === true", false);
            if let Ok(result) = ready_check {
                if let Some(value) = result.value {
                    if value == true {
                        tracing::debug!("Iframe is ready");
                        break;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        // Execute the challenge code
        // The challenge is an async IIFE that returns an object with server_hashes, client_hashes, etc.
        let execute_script = format!(
            r#"
            (async function() {{
                try {{
                    const challengeFn = {};
                    const result = await challengeFn;
                    window.__challengeResult = JSON.stringify(result);
                }} catch (e) {{
                    window.__challengeError = e.message + ' | ' + e.stack;
                }}
            }})();
            "#,
            challenge_code
        );

        tab.evaluate(&execute_script, false)
            .context("Failed to start challenge execution")?;

        // Wait for the challenge to complete (poll for result)
        let timeout = Duration::from_secs(15);
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                // Try to get any error that occurred
                let error_check = tab.evaluate("window.__challengeError", false);
                if let Ok(error_result) = error_check {
                    if let Some(error) = error_result.value {
                        if !error.is_null() {
                            let error_str = error.as_str().unwrap_or("Unknown error");
                            return Err(anyhow!("Challenge execution failed: {}", error_str));
                        }
                    }
                }
                return Err(anyhow!("Challenge execution timed out"));
            }

            // Check for error first
            let error_check = tab
                .evaluate("window.__challengeError", false)
                .context("Failed to check for error")?;
            if let Some(error) = error_check.value {
                if !error.is_null() {
                    let error_str = error.as_str().unwrap_or("Unknown error");
                    return Err(anyhow!("Challenge execution failed: {}", error_str));
                }
            }

            // Check for result
            let result_check = tab
                .evaluate("window.__challengeResult", false)
                .context("Failed to check for result")?;
            if let Some(result) = result_check.value {
                if !result.is_null() {
                    let result_str = result
                        .as_str()
                        .ok_or_else(|| anyhow!("Result is not a string"))?;
                    let challenge_data: ChallengeResult = serde_json::from_str(result_str)
                        .context("Failed to parse challenge result as JSON")?;
                    return Ok(challenge_data);
                }
            }

            // Wait a bit before polling again
            std::thread::sleep(Duration::from_millis(100));
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
}
