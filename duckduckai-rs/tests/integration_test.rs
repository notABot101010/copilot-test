use duckduckai::DuckDuckGoClient;

#[tokio::test]
async fn test_client_initialization() {
    let client = DuckDuckGoClient::new();
    assert!(client.is_ok(), "Client should initialize successfully");
}

#[tokio::test]
#[ignore] // Ignored by default - requires valid anti-bot headers
async fn test_simple_chat() {
    let mut client = DuckDuckGoClient::new().expect("Failed to create client");

    let result = client.chat("What is 2+2?", None).await;

    if let Err(ref err) = result {
        eprintln!("Chat error: {:#?}", err);
        eprintln!("Note: This test requires fresh anti-bot headers from real browser traffic");
        return;
    }

    let response = result.unwrap();
    assert!(!response.is_empty(), "Response should not be empty");
    println!("Response: {}", response);
}

#[tokio::test]
#[ignore] // Ignored by default - requires valid anti-bot headers
async fn test_chat_with_specific_model() {
    let mut client = DuckDuckGoClient::new().expect("Failed to create client");

    let result = client.chat("Say hello", Some("gpt-4o-mini")).await;

    if let Err(ref err) = result {
        eprintln!("Chat error: {:#?}", err);
        eprintln!("Note: This test requires fresh anti-bot headers from real browser traffic");
        return;
    }

    let response = result.unwrap();
    assert!(!response.is_empty(), "Response should not be empty");
    println!("Response: {}", response);
}

#[tokio::test]
#[ignore] // Ignored by default - requires valid anti-bot headers
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
        eprintln!("Note: This test requires fresh anti-bot headers from real browser traffic");
        return;
    }

    assert!(!chunks.is_empty(), "Should receive at least one chunk");
    let full_response = chunks.join("");
    println!("Response: {}", full_response);
}

#[tokio::test]
#[ignore] // Ignored by default - requires valid anti-bot headers
async fn test_multiple_requests() {
    let mut client = DuckDuckGoClient::new().expect("Failed to create client");

    // First request
    let result1 = client.chat("What is the capital of France?", None).await;

    if let Err(ref err) = result1 {
        eprintln!("Chat error: {:#?}", err);
        eprintln!("Note: This test requires fresh anti-bot headers from real browser traffic");
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
