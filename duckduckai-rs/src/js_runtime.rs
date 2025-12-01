//! JavaScript runtime for solving DuckDuckGo VQD challenges
//!
//! This module provides a sandboxed JavaScript execution environment using QuickJS
//! to solve the anti-bot challenges from DuckDuckGo's AI chat API.

use anyhow::{Context, Result};
use aws_lc_rs::digest::{SHA256, digest};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use rquickjs::{AsyncContext, AsyncRuntime, Function, Object};
use serde::{Deserialize, Serialize};
use std::time::Instant;

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

/// The VQD challenge solver using QuickJS runtime
pub struct ChallengeSolver {
    runtime: AsyncRuntime,
}

impl ChallengeSolver {
    /// Create a new challenge solver
    pub fn new() -> Result<Self> {
        let runtime = AsyncRuntime::new().context("Failed to create QuickJS runtime")?;
        Ok(Self { runtime })
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

        // Decode the base64 challenge
        let challenge_bytes = BASE64
            .decode(challenge_b64)
            .context("Failed to decode challenge base64")?;
        let challenge_str =
            String::from_utf8(challenge_bytes).context("Challenge is not valid UTF-8")?;

        tracing::debug!("Decoded challenge: {}", &challenge_str[..challenge_str.len().min(DEBUG_TRUNCATE_LEN)]);

        // The challenge can be either:
        // 1. JSON data directly (old format)
        // 2. JavaScript code that returns the challenge object (new format)
        let challenge = self.parse_or_execute_challenge(&challenge_str).await
            .context("Failed to parse or execute challenge")?;

        tracing::debug!("Challenge data: {:?}", challenge);

        // Execute the challenge in the JavaScript runtime
        let result = self.execute_challenge(&challenge, start_time).await?;

        // Encode the result as base64
        let result_json = serde_json::to_string(&result).context("Failed to serialize result")?;
        let result_b64 = BASE64.encode(result_json.as_bytes());

        tracing::debug!("Solved challenge, result length: {}", result_b64.len());

        Ok(result_b64)
    }

    /// Try to parse challenge as JSON, or execute it as JavaScript if parsing fails
    async fn parse_or_execute_challenge(&self, challenge_str: &str) -> Result<ChallengeData> {
        // First, try to parse as JSON directly
        if let Ok(challenge) = serde_json::from_str::<ChallengeData>(challenge_str) {
            tracing::debug!("Challenge parsed as JSON directly");
            return Ok(challenge);
        }

        tracing::debug!("Challenge is not JSON, executing as JavaScript");

        // If not JSON, execute as JavaScript code
        let context = AsyncContext::full(&self.runtime)
            .await
            .context("Failed to create JS context")?;

        let challenge_code = challenge_str.to_string();
        let challenge = context
            .with(|ctx| {
                // Set up browser-like environment first
                self.setup_browser_globals(&ctx)?;

                // Execute the challenge code - it should return an object
                let result: rquickjs::Value = ctx.eval(challenge_code.as_bytes())
                    .map_err(|e| anyhow::anyhow!("JavaScript execution error: {:?}", e))?;

                // Convert the JS result to JSON string using JSON.stringify
                let json_obj: Object = ctx.globals().get("JSON")
                    .map_err(|e| anyhow::anyhow!("Failed to get JSON object: {:?}", e))?;
                let stringify_fn: Function = json_obj.get("stringify")
                    .map_err(|e| anyhow::anyhow!("Failed to get JSON.stringify: {:?}", e))?;
                
                let json_str: String = stringify_fn.call((result,))
                    .map_err(|e| anyhow::anyhow!("Failed to stringify JS result: {:?}", e))?;

                tracing::debug!("JavaScript result JSON: {}", &json_str[..json_str.len().min(DEBUG_TRUNCATE_LEN)]);

                serde_json::from_str(&json_str)
                    .context("Failed to parse JavaScript result as ChallengeData")
            })
            .await?;

        Ok(challenge)
    }

    /// Execute the challenge and compute client hashes
    async fn execute_challenge(
        &self,
        challenge: &ChallengeData,
        start_time: Instant,
    ) -> Result<ChallengeData> {
        let context = AsyncContext::full(&self.runtime)
            .await
            .context("Failed to create JS context")?;

        // Compute client hashes by hashing the server hashes with browser-like data
        let client_hashes = self.compute_client_hashes(&context, challenge).await?;

        let duration = start_time.elapsed().as_millis().to_string();

        // Build the result with updated metadata
        let result = ChallengeData {
            server_hashes: challenge.server_hashes.clone(),
            client_hashes,
            signals: challenge.signals.clone(),
            meta: ChallengeMeta {
                v: challenge.meta.v.clone(),
                challenge_id: challenge.meta.challenge_id.clone(),
                timestamp: challenge.meta.timestamp.clone(),
                debug: challenge.meta.debug.clone(),
                origin: "https://duckduckgo.com".to_string(),
                stack: "Error\nat anonymous (eval:1:1)".to_string(),
                duration,
            },
        };

        Ok(result)
    }

    /// Compute client hashes using QuickJS for any JavaScript evaluation
    async fn compute_client_hashes(
        &self,
        context: &AsyncContext,
        challenge: &ChallengeData,
    ) -> Result<Vec<String>> {
        // For each server hash, we compute a corresponding client hash
        // The client hash is derived from browser fingerprinting data
        // Here we simulate the browser environment

        let mut client_hashes = Vec::new();

        context
            .with(|ctx| {
                // Set up global browser-like environment
                self.setup_browser_globals(&ctx)?;

                // For each server hash, compute a client hash
                for (i, _server_hash) in challenge.server_hashes.iter().enumerate() {
                    // Generate a fingerprint-like string based on index
                    // In a real browser, this would be computed from actual browser data
                    let fingerprint = self.generate_fingerprint(&ctx, i)?;

                    // Hash the fingerprint with SHA-256 and encode as base64
                    let hash = self.sha256_base64(&fingerprint);
                    client_hashes.push(hash);
                }

                Ok::<_, anyhow::Error>(())
            })
            .await?;

        Ok(client_hashes)
    }

    /// Set up browser-like global objects in the JavaScript context
    fn setup_browser_globals(&self, ctx: &rquickjs::Ctx) -> Result<()> {
        let globals = ctx.globals();

        // Create window object
        let window = Object::new(ctx.clone())?;

        // Set location
        let location = Object::new(ctx.clone())?;
        location.set("origin", "https://duckduckgo.com")?;
        location.set("href", "https://duckduckgo.com/aichat")?;
        location.set("protocol", "https:")?;
        location.set("host", "duckduckgo.com")?;
        location.set("hostname", "duckduckgo.com")?;
        location.set("pathname", "/aichat")?;
        window.set("location", location.clone())?;

        // Set navigator
        let navigator = Object::new(ctx.clone())?;
        navigator.set(
            "userAgent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        )?;
        navigator.set("language", "en-US")?;
        navigator.set("languages", vec!["en-US", "en"])?;
        navigator.set("platform", "MacIntel")?;
        navigator.set("hardwareConcurrency", 8)?;
        navigator.set("deviceMemory", 8)?;
        navigator.set("maxTouchPoints", 0)?;
        navigator.set("cookieEnabled", true)?;
        navigator.set("onLine", true)?;
        window.set("navigator", navigator)?;

        // Set document
        let document = Object::new(ctx.clone())?;
        document.set("documentElement", Object::new(ctx.clone())?)?;
        document.set("domain", "duckduckgo.com")?;
        document.set("referrer", "")?;
        window.set("document", document)?;

        // Set screen
        let screen = Object::new(ctx.clone())?;
        screen.set("width", 1920)?;
        screen.set("height", 1080)?;
        screen.set("availWidth", 1920)?;
        screen.set("availHeight", 1055)?;
        screen.set("colorDepth", 24)?;
        screen.set("pixelDepth", 24)?;
        window.set("screen", screen)?;

        // Set timing-related
        window.set("innerWidth", 1920)?;
        window.set("innerHeight", 1080)?;
        window.set("outerWidth", 1920)?;
        window.set("outerHeight", 1080)?;
        window.set("devicePixelRatio", 1.0)?;

        // Add atob and btoa functions
        let atob_fn =
            Function::new(ctx.clone(), |s: String| -> rquickjs::Result<String> {
                BASE64
                    .decode(&s)
                    .map_err(|_| rquickjs::Error::new_from_js("base64", "string"))
                    .and_then(|bytes| {
                        String::from_utf8(bytes).map_err(|_| {
                            rquickjs::Error::new_from_js("bytes", "string")
                        })
                    })
            })?;
        globals.set("atob", atob_fn)?;

        let btoa_fn = Function::new(ctx.clone(), |s: String| -> String { BASE64.encode(s.as_bytes()) })?;
        globals.set("btoa", btoa_fn)?;

        // Date.now
        let date_obj = Object::new(ctx.clone())?;
        let now_fn = Function::new(ctx.clone(), || -> i64 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0)
        })?;
        date_obj.set("now", now_fn)?;
        globals.set("Date", date_obj)?;

        // Set window and self
        globals.set("window", window.clone())?;
        globals.set("self", window.clone())?;
        globals.set("top", window.clone())?;
        globals.set("parent", window)?;
        globals.set("location", location)?;

        // Add console for debugging
        let console = Object::new(ctx.clone())?;
        let log_fn = Function::new(ctx.clone(), |msg: String| {
            tracing::debug!("JS console.log: {}", msg);
        })?;
        console.set("log", log_fn)?;
        globals.set("console", console)?;

        Ok(())
    }

    /// Generate a browser fingerprint string for the given index
    fn generate_fingerprint(&self, _ctx: &rquickjs::Ctx, index: usize) -> Result<String> {
        // Generate consistent fingerprint data based on index
        // This simulates what the browser would compute from various APIs
        // These are hardcoded constants - not user-controlled input

        const FINGERPRINTS: [&str; 3] = [
            // Canvas fingerprint-like data
            "canvas:1920x1080:24:MacIntel:en-US",
            // WebGL fingerprint-like data
            "webgl:ANGLE (Apple, ANGLE Metal Renderer: Apple M1, Unspecified Version)",
            // Audio fingerprint-like data
            "audio:124.04347657808103",
        ];

        let fingerprint = FINGERPRINTS
            .get(index)
            .unwrap_or(&"default-fingerprint")
            .to_string();

        // Add timestamp to fingerprint (pure Rust, no JS eval needed)
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        Ok(format!("{}:{}", fingerprint, ts))
    }

    /// Compute SHA-256 hash and encode as base64
    fn sha256_base64(&self, input: &str) -> String {
        let hash = digest(&SHA256, input.as_bytes());
        BASE64.encode(hash.as_ref())
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
        let solver = ChallengeSolver::new().unwrap();
        let hash = solver.sha256_base64("test");
        // SHA256 of "test" = 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08
        // base64 of that = n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg=
        assert_eq!(hash, "n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg=");
    }

    #[tokio::test]
    async fn test_solve_sample_challenge() {
        let solver = ChallengeSolver::new().unwrap();

        // Create a sample challenge
        let challenge = ChallengeData {
            server_hashes: vec![
                "D6tw1CTSfzfXxZAzuq8FTWwYhNO+7ILj167buCgD0kE=".to_string(),
                "zYTKWaCoardg8EbpKvhuj7J7mbjEB8aU0+kz1zEni5c=".to_string(),
                "4q+SQC+gzYtSTye8aVUEl9tVOYjSDWeGrlnlszr1yfo=".to_string(),
            ],
            client_hashes: vec![],
            signals: serde_json::json!({}),
            meta: ChallengeMeta {
                v: "4".to_string(),
                challenge_id: "test_challenge_id".to_string(),
                timestamp: "1764563984360".to_string(),
                debug: "\u{0019}\u{001f}".to_string(),
                origin: "https://duckduckgo.com".to_string(),
                stack: "Error\nat l (https://duckduckgo.com/dist/wpm.main.758b58e5295173a9d89c.js:1:424103)".to_string(),
                duration: "9".to_string(),
            },
        };

        let challenge_json = serde_json::to_string(&challenge).unwrap();
        let challenge_b64 = BASE64.encode(challenge_json.as_bytes());

        let result = solver.solve(&challenge_b64).await;
        assert!(result.is_ok(), "Challenge solving failed: {:?}", result);

        let solved = result.unwrap();
        // Verify the result is valid base64
        let decoded = BASE64.decode(&solved);
        assert!(decoded.is_ok(), "Result is not valid base64");

        // Verify the decoded result is valid JSON with expected structure
        let decoded_json: ChallengeData = serde_json::from_slice(&decoded.unwrap()).unwrap();
        assert_eq!(decoded_json.server_hashes.len(), 3);
        assert_eq!(decoded_json.client_hashes.len(), 3);
        assert_eq!(decoded_json.meta.origin, "https://duckduckgo.com");
    }

    #[tokio::test]
    async fn test_solve_javascript_challenge() {
        let solver = ChallengeSolver::new().unwrap();

        // Create a JavaScript challenge that returns the challenge object
        let js_challenge = r#"(function(){
            return {
                "server_hashes": ["hash1", "hash2", "hash3"],
                "client_hashes": [],
                "signals": {},
                "meta": {
                    "v": "4",
                    "challenge_id": "js_test_challenge",
                    "timestamp": "1764563984360",
                    "debug": "",
                    "origin": "https://duckduckgo.com",
                    "stack": "Error\nat test",
                    "duration": "5"
                }
            };
        })()"#;

        let challenge_b64 = BASE64.encode(js_challenge.as_bytes());

        let result = solver.solve(&challenge_b64).await;
        assert!(result.is_ok(), "JavaScript challenge solving failed: {:?}", result);

        let solved = result.unwrap();
        // Verify the result is valid base64
        let decoded = BASE64.decode(&solved);
        assert!(decoded.is_ok(), "Result is not valid base64");

        // Verify the decoded result has expected structure
        let decoded_json: ChallengeData = serde_json::from_slice(&decoded.unwrap()).unwrap();
        assert_eq!(decoded_json.server_hashes.len(), 3);
        assert_eq!(decoded_json.client_hashes.len(), 3);
        assert_eq!(decoded_json.meta.challenge_id, "js_test_challenge");
        assert_eq!(decoded_json.meta.origin, "https://duckduckgo.com");
    }

    #[tokio::test]
    async fn test_solve_javascript_challenge_with_computation() {
        let solver = ChallengeSolver::new().unwrap();

        // Create a more complex JavaScript challenge that computes values
        let js_challenge = r#"(function(){
            var serverHashes = [];
            for (var i = 0; i < 3; i++) {
                serverHashes.push("computed_hash_" + i);
            }
            return {
                "server_hashes": serverHashes,
                "client_hashes": [],
                "signals": { "computed": true },
                "meta": {
                    "v": "4",
                    "challenge_id": "computed_challenge",
                    "timestamp": String(Date.now()),
                    "debug": "",
                    "origin": location.origin,
                    "stack": "",
                    "duration": "10"
                }
            };
        })()"#;

        let challenge_b64 = BASE64.encode(js_challenge.as_bytes());

        let result = solver.solve(&challenge_b64).await;
        assert!(result.is_ok(), "JavaScript computation challenge failed: {:?}", result);

        let solved = result.unwrap();
        let decoded = BASE64.decode(&solved).unwrap();
        let decoded_json: ChallengeData = serde_json::from_slice(&decoded).unwrap();
        
        // Verify computed values
        assert_eq!(decoded_json.server_hashes.len(), 3);
        assert_eq!(decoded_json.server_hashes[0], "computed_hash_0");
        assert_eq!(decoded_json.server_hashes[1], "computed_hash_1");
        assert_eq!(decoded_json.server_hashes[2], "computed_hash_2");
        assert_eq!(decoded_json.meta.origin, "https://duckduckgo.com");
    }
}
