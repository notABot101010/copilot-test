use duckduckai::{DuckDuckGoClient, ChallengeSolver};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

#[tokio::test]
async fn test_client_initialization() {
    let client = DuckDuckGoClient::new();
    assert!(client.is_ok(), "Client should initialize successfully");
}

#[tokio::test]
async fn test_challenge_solver_initialization() {
    let solver = ChallengeSolver::new();
    assert!(solver.is_ok(), "Challenge solver should initialize successfully");
}

/// Test that we can solve a sample challenge with the JavaScript runtime
#[tokio::test]
async fn test_challenge_solver_with_sample_data() {
    let solver = ChallengeSolver::new().expect("Failed to create solver");
    
    // Create a sample challenge similar to what the real API returns
    let sample_challenge = serde_json::json!({
        "server_hashes": [
            "D6tw1CTSfzfXxZAzuq8FTWwYhNO+7ILj167buCgD0kE=",
            "zYTKWaCoardg8EbpKvhuj7J7mbjEB8aU0+kz1zEni5c=",
            "4q+SQC+gzYtSTye8aVUEl9tVOYjSDWeGrlnlszr1yfo="
        ],
        "client_hashes": [],
        "signals": {},
        "meta": {
            "v": "4",
            "challenge_id": "test_challenge_id",
            "timestamp": "1764563984360",
            "debug": "",
            "origin": "https://duckduckgo.com",
            "stack": "Error\nat l (test.js:1:1)",
            "duration": "9"
        }
    });
    
    let challenge_json = serde_json::to_string(&sample_challenge).unwrap();
    let challenge_b64 = BASE64.encode(challenge_json.as_bytes());
    
    let result = solver.solve(&challenge_b64).await;
    assert!(result.is_ok(), "Challenge solving failed: {:?}", result);
    
    let solved = result.unwrap();
    
    // Verify the result is valid base64
    let decoded = BASE64.decode(&solved);
    assert!(decoded.is_ok(), "Result should be valid base64");
    
    // Verify the decoded result is valid JSON
    let decoded_json: serde_json::Value = serde_json::from_slice(&decoded.unwrap()).unwrap();
    
    // Verify structure
    assert!(decoded_json.get("server_hashes").is_some(), "Should have server_hashes");
    assert!(decoded_json.get("client_hashes").is_some(), "Should have client_hashes");
    assert!(decoded_json.get("meta").is_some(), "Should have meta");
    
    // Verify we computed client hashes
    let client_hashes = decoded_json["client_hashes"].as_array().unwrap();
    assert_eq!(client_hashes.len(), 3, "Should have 3 client hashes");
    
    // Verify origin is set correctly
    assert_eq!(
        decoded_json["meta"]["origin"].as_str().unwrap(),
        "https://duckduckgo.com",
        "Origin should be duckduckgo.com"
    );
    
    println!("Solved challenge successfully!");
    println!("Client hashes: {:?}", client_hashes);
}

/// Integration test: Fetch X-Vqd-Hash-1 from status endpoint and solve the challenge
/// This test requires network access to the real DuckDuckGo API
#[tokio::test]
#[ignore] // Ignored by default - requires network access
async fn test_fetch_and_solve_vqd_challenge() {
    let client = reqwest::Client::new();
    
    // Step 1: Fetch challenge from status endpoint
    let response = client
        .get("https://duckduckgo.com/duckchat/v1/status")
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .header("Accept", "*/*")
        .header("x-vqd-accept", "1")
        .send()
        .await;
    
    if let Err(ref err) = response {
        eprintln!("Failed to fetch status: {:#?}", err);
        eprintln!("Note: This test requires network access to duckduckgo.com");
        return;
    }
    
    let response = response.unwrap();
    println!("Status: {}", response.status());
    
    // Step 2: Extract challenge from x-vqd-hash-1 header
    let challenge = response
        .headers()
        .get("x-vqd-hash-1")
        .map(|h| h.to_str().unwrap().to_string());
    
    if challenge.is_none() {
        eprintln!("x-vqd-hash-1 header not found in response");
        eprintln!("Headers: {:?}", response.headers());
        return;
    }
    
    let challenge = challenge.unwrap();
    println!("Received challenge (first 50 chars): {}...", &challenge[..challenge.len().min(50)]);
    
    // Step 3: Solve the challenge using the JavaScript runtime
    let solver = ChallengeSolver::new().expect("Failed to create solver");
    let solved = solver.solve(&challenge).await;
    
    assert!(solved.is_ok(), "Challenge solving failed: {:?}", solved);
    
    let vqd_header = solved.unwrap();
    println!("Solved VQD header (first 50 chars): {}...", &vqd_header[..vqd_header.len().min(50)]);
    
    // Verify the solved header is valid base64 JSON
    let decoded = BASE64.decode(&vqd_header);
    assert!(decoded.is_ok(), "Solved header should be valid base64");
    
    let decoded_json: serde_json::Value = serde_json::from_slice(&decoded.unwrap()).unwrap();
    assert!(decoded_json.get("client_hashes").is_some(), "Should have client_hashes");
    
    println!("Successfully fetched and solved VQD challenge!");
}

/// Full integration test: Fetch challenge, solve it, and use it to chat
/// This test requires network access to the real DuckDuckGo API
#[tokio::test]
#[ignore] // Ignored by default - requires network access
async fn test_full_chat_flow_with_challenge_solving() {
    let mut client = DuckDuckGoClient::new().expect("Failed to create client");
    
    // This will internally:
    // 1. Fetch the challenge from status endpoint
    // 2. Solve the challenge using JavaScript runtime
    // 3. Use the solved X-Vqd-Hash-1 header for chat
    let result = client.chat("What is 2+2? Reply with just the number.", None).await;
    
    if let Err(ref err) = result {
        eprintln!("Chat error: {:#?}", err);
        eprintln!("Note: This test requires network access and the challenge solver to work correctly");
        return;
    }
    
    let response = result.unwrap();
    assert!(!response.is_empty(), "Response should not be empty");
    println!("Chat response: {}", response);
    
    // Verify response contains "4" somewhere
    assert!(response.contains("4"), "Response should contain the answer 4");
}

#[tokio::test]
#[ignore] // Ignored by default - requires network access
async fn test_simple_chat() {
    let mut client = DuckDuckGoClient::new().expect("Failed to create client");

    let result = client.chat("What is 2+2?", None).await;

    if let Err(ref err) = result {
        eprintln!("Chat error: {:#?}", err);
        eprintln!("Note: This test requires network access to the DuckDuckGo API");
        return;
    }

    let response = result.unwrap();
    assert!(!response.is_empty(), "Response should not be empty");
    println!("Response: {}", response);
}

#[tokio::test]
#[ignore] // Ignored by default - requires network access
async fn test_chat_with_specific_model() {
    let mut client = DuckDuckGoClient::new().expect("Failed to create client");

    let result = client.chat("Say hello", Some("gpt-4o-mini")).await;

    if let Err(ref err) = result {
        eprintln!("Chat error: {:#?}", err);
        eprintln!("Note: This test requires network access to the DuckDuckGo API");
        return;
    }

    let response = result.unwrap();
    assert!(!response.is_empty(), "Response should not be empty");
    println!("Response: {}", response);
}

#[tokio::test]
#[ignore] // Ignored by default - requires network access
async fn test_streaming_chat() {
    let mut client = DuckDuckGoClient::new().expect("Failed to create client");

    let mut chunks = Vec::new();

    let result = client
        .chat_stream("Count to 3", None, |chunk| {
            chunks.push(chunk.to_string());
        })
        .await;

    if let Err(ref err) = result {
        eprintln!("Streaming chat error: {:#?}", err);
        eprintln!("Note: This test requires network access to the DuckDuckGo API");
        return;
    }

    assert!(!chunks.is_empty(), "Should receive at least one chunk");
    let full_response = chunks.join("");
    println!("Response: {}", full_response);
}

#[tokio::test]
#[ignore] // Ignored by default - requires network access
async fn test_multiple_requests() {
    let mut client = DuckDuckGoClient::new().expect("Failed to create client");

    // First request
    let result1 = client.chat("What is the capital of France?", None).await;

    if let Err(ref err) = result1 {
        eprintln!("Chat error: {:#?}", err);
        eprintln!("Note: This test requires network access to the DuckDuckGo API");
        return;
    }

    let response1 = result1.unwrap();
    println!("Response 1: {}", response1);

    // Second request with same client (testing VQD token reuse)
    let result2 = client.chat("What is 10 + 5?", None).await;

    if let Err(ref err) = result2 {
        eprintln!("Second chat error: {:#?}", err);
        return;
    }

    let response2 = result2.unwrap();
    println!("Response 2: {}", response2);
}

/// Test that the challenge solver can handle JavaScript code challenges
/// This is the new format that DuckDuckGo uses
#[tokio::test]
async fn test_javascript_code_challenge() {
    let solver = ChallengeSolver::new().expect("Failed to create solver");
    
    // This simulates a JavaScript challenge that DuckDuckGo might return
    let js_challenge = r#"(function(){
        return {
            "server_hashes": [
                "ServerHash1==",
                "ServerHash2==",
                "ServerHash3=="
            ],
            "client_hashes": [],
            "signals": {},
            "meta": {
                "v": "4",
                "challenge_id": "integration_test_js",
                "timestamp": "1764563984360",
                "debug": "",
                "origin": "https://duckduckgo.com",
                "stack": "Error\nat test:1:1",
                "duration": "5"
            }
        };
    })()"#;
    
    let challenge_b64 = BASE64.encode(js_challenge.as_bytes());
    
    let result = solver.solve(&challenge_b64).await;
    assert!(result.is_ok(), "JavaScript challenge should be solved successfully: {:?}", result);
    
    let solved = result.unwrap();
    
    // Verify the result is valid base64
    let decoded = BASE64.decode(&solved);
    assert!(decoded.is_ok(), "Solved result should be valid base64");
    
    // Verify the structure
    let decoded_json: serde_json::Value = serde_json::from_slice(&decoded.unwrap()).unwrap();
    assert!(decoded_json.get("server_hashes").is_some(), "Should have server_hashes");
    assert!(decoded_json.get("client_hashes").is_some(), "Should have client_hashes");
    assert!(decoded_json.get("meta").is_some(), "Should have meta");
    
    // Verify client hashes were computed
    let client_hashes = decoded_json["client_hashes"].as_array().unwrap();
    assert_eq!(client_hashes.len(), 3, "Should have 3 client hashes");
    
    // Verify meta fields
    assert_eq!(decoded_json["meta"]["challenge_id"].as_str().unwrap(), "integration_test_js");
    assert_eq!(decoded_json["meta"]["origin"].as_str().unwrap(), "https://duckduckgo.com");
    
    println!("JavaScript challenge solved successfully!");
}

/// Test JavaScript challenge with browser API usage
#[tokio::test]
async fn test_javascript_challenge_with_browser_apis() {
    let solver = ChallengeSolver::new().expect("Failed to create solver");
    
    // This challenge uses browser APIs that should be available in our QuickJS environment
    // Note: We use more careful access patterns since the globals may have some limitations
    let js_challenge = r#"(function(){
        // Build the challenge response
        var result = {
            "server_hashes": ["api_test_1", "api_test_2"],
            "client_hashes": [],
            "signals": {},
            "meta": {
                "v": "4",
                "challenge_id": "browser_api_test",
                "timestamp": String(Date.now()),
                "debug": "",
                "origin": "",
                "stack": "",
                "duration": "1"
            }
        };
        
        // Try to use location if available
        if (typeof location !== 'undefined' && location.origin) {
            result.meta.origin = location.origin;
            result.signals.hasLocation = true;
        }
        
        // Try to use navigator if available
        if (typeof navigator !== 'undefined') {
            result.signals.hasNavigator = true;
        }
        
        return result;
    })()"#;
    
    let challenge_b64 = BASE64.encode(js_challenge.as_bytes());
    
    let result = solver.solve(&challenge_b64).await;
    assert!(result.is_ok(), "Browser API challenge should be solved: {:?}", result);
    
    let solved = result.unwrap();
    let decoded = BASE64.decode(&solved).unwrap();
    let decoded_json: serde_json::Value = serde_json::from_slice(&decoded).unwrap();
    
    // Verify the challenge was solved
    assert!(decoded_json.get("server_hashes").is_some(), "Should have server_hashes");
    assert!(decoded_json.get("client_hashes").is_some(), "Should have client_hashes");
    
    // Verify origin was retrieved correctly
    assert_eq!(decoded_json["meta"]["origin"].as_str().unwrap(), "https://duckduckgo.com");
    
    println!("Browser API challenge solved successfully!");
    println!("Meta: {}", serde_json::to_string_pretty(&decoded_json["meta"]).unwrap());
}
