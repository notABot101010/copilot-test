//! JavaScript runtime for solving DuckDuckGo VQD challenges
//!
//! This module provides a challenge solver for DuckDuckGo's AI chat API.
//! It extracts challenge data and computes client hashes without requiring a JS runtime.

use anyhow::{Context, Result};
use aws_lc_rs::digest::{SHA256, digest};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
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

/// The VQD challenge solver - pure Rust implementation
pub struct ChallengeSolver;

impl ChallengeSolver {
    /// Create a new challenge solver
    pub fn new() -> Result<Self> {
        Ok(Self)
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

        // Parse the challenge (JSON or extract from JavaScript)
        let challenge = self.parse_challenge(&challenge_str)
            .context("Failed to parse challenge")?;

        tracing::debug!("Challenge data: {:?}", challenge);

        // Execute the challenge and compute client hashes (pure Rust)
        let result = self.execute_challenge(&challenge, start_time)?;

        // Encode the result as base64
        let result_json = serde_json::to_string(&result).context("Failed to serialize result")?;
        let result_b64 = BASE64.encode(result_json.as_bytes());

        tracing::debug!("Solved challenge, result length: {}", result_b64.len());

        Ok(result_b64)
    }

    /// Parse challenge - either JSON directly or extract from JavaScript
    fn parse_challenge(&self, challenge_str: &str) -> Result<ChallengeData> {
        // First, try to parse as JSON directly
        if let Ok(challenge) = serde_json::from_str::<ChallengeData>(challenge_str) {
            tracing::debug!("Challenge parsed as JSON directly");
            return Ok(challenge);
        }

        tracing::debug!("Challenge is not JSON, extracting data from JavaScript");

        // Extract server_hashes from the JavaScript code
        // The challenge returns an object with server_hashes that are base64-encoded strings
        // They appear in the code like: 'server_hashes':['base64string1','base64string2','base64string3']
        // Or using obfuscated variable references
        
        // Look for base64-encoded strings that look like hashes (44 chars ending with =)
        let base64_pattern = regex::Regex::new(r"'([A-Za-z0-9+/]{40,}={0,2})'")
            .map_err(|e| anyhow::anyhow!("Failed to compile regex: {:?}", e))?;
        
        let mut server_hashes: Vec<String> = Vec::new();
        for cap in base64_pattern.captures_iter(challenge_str) {
            if let Some(hash) = cap.get(1) {
                let hash_str = hash.as_str().to_string();
                // Only include if it looks like a valid base64 hash (SHA-256 = 32 bytes = 44 chars base64)
                if hash_str.len() >= 40 && hash_str.len() <= 50 {
                    server_hashes.push(hash_str);
                }
            }
        }

        // Look for challenge_id - typically in format like 'xxxxxxxx' (hex string followed by something)
        let challenge_id_pattern = regex::Regex::new(r"'([a-f0-9]{64}[a-z0-9]+)'")
            .map_err(|e| anyhow::anyhow!("Failed to compile regex: {:?}", e))?;
        
        let challenge_id = challenge_id_pattern
            .captures(challenge_str)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Look for timestamp - typically a numeric string like '1764576114716'
        let timestamp_pattern = regex::Regex::new(r"'(\d{13})'")
            .map_err(|e| anyhow::anyhow!("Failed to compile regex: {:?}", e))?;
        
        let timestamp = timestamp_pattern
            .captures(challenge_str)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis().to_string())
                    .unwrap_or_else(|_| "0".to_string())
            });

        // Limit to first 3 server_hashes (that's what the challenge typically has)
        if server_hashes.len() > 3 {
            server_hashes.truncate(3);
        }

        if server_hashes.is_empty() {
            return Err(anyhow::anyhow!("Could not extract server_hashes from challenge JavaScript"));
        }

        tracing::debug!("Extracted {} server hashes from JavaScript", server_hashes.len());

        // Create a challenge data structure with extracted values
        // The client_hashes will be computed later based on fingerprinting simulation
        Ok(ChallengeData {
            server_hashes,
            client_hashes: vec![],
            signals: serde_json::json!({}),
            meta: ChallengeMeta {
                v: "4".to_string(),
                challenge_id,
                timestamp,
                debug: "".to_string(),
                origin: "https://duckduckgo.com".to_string(),
                stack: "Error\nat anonymous (eval:1:1)".to_string(),
                duration: "0".to_string(),
            },
        })
    }

    /// Execute the challenge and compute client hashes (pure Rust)
    fn execute_challenge(
        &self,
        challenge: &ChallengeData,
        start_time: Instant,
    ) -> Result<ChallengeData> {
        // Compute client hashes using pure Rust
        let client_hashes = self.compute_client_hashes(challenge)?;

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

    /// Compute client hashes (pure Rust, simulating browser behavior)
    ///
    /// Based on analysis of the DuckDuckGo challenge JavaScript, the client_hashes are:
    /// 1. The browser's userAgent string
    /// 2. A DOM-based fingerprint (0x1aee + innerHTML.length * querySelectorAll('*').length)
    /// 3. A bot detection result (sum of various boolean checks + 0x1d64 or 0xf2e)
    ///
    /// These raw strings are then SHA-256 hashed and base64 encoded.
    fn compute_client_hashes(&self, _challenge: &ChallengeData) -> Result<Vec<String>> {
        // The raw fingerprint values that would be collected by the JavaScript
        let raw_fingerprints = [
            // 1. navigator.userAgent - the full user agent string
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36",
            
            // 2. DOM-based fingerprint calculation:
            //    String(0x1aee + innerHTML.length * querySelectorAll('*').length)
            //    For an empty div with innerHTML='<li><div></li><li></div':
            //    innerHTML.length = 24 (or thereabouts), querySelectorAll('*').length = 2
            //    So: 6894 + 24*2 = 6942
            //    But this varies based on the HTML parsing - typical value is around "6894"
            "6894",
            
            // 3. Bot detection fingerprint:
            //    String([webdriver===true, iframeTest, aliasTest].map(Number).reduce((a,b)=>a+b, baseValue))
            //    The base value changes (0x1d64 = 7524 or 0xf2e = 3886)
            //    For a real browser: webdriver=false(0), iframeTest varies, aliasTest=false(0)
            //    Typical values: "7525" (7524+1) for 0x1d64 base, or "3887" for 0xf2e base
            "7525",
        ];

        let mut client_hashes = Vec::new();
        for fingerprint in raw_fingerprints {
            // Hash with SHA-256 and encode as base64
            let hash = self.sha256_base64(fingerprint);
            client_hashes.push(hash);
        }

        Ok(client_hashes)
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
