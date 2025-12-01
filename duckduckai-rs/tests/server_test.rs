use async_openai::{
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
    Client, config::OpenAIConfig,
};
use duckduckai::{run_server, DEFAULT_MODEL};
use std::time::Duration;
use tokio::time::sleep;

const TEST_API_KEY: &str = "test-api-key-12345";
const TEST_PORT: u16 = 3456;

async fn start_test_server() {
    tokio::spawn(async move {
        run_server("127.0.0.1", TEST_PORT, TEST_API_KEY.to_string())
            .await
            .expect("Server failed to start");
    });

    // Give the server time to start
    sleep(Duration::from_secs(2)).await;
}

fn create_test_client() -> Client<OpenAIConfig> {
    let config = OpenAIConfig::new()
        .with_api_key(TEST_API_KEY)
        .with_api_base(format!("http://127.0.0.1:{}/v1", TEST_PORT));

    Client::with_config(config)
}

#[tokio::test]
#[ignore] // Requires network access to DuckDuckGo
async fn test_server_non_streaming_chat_completion() {
    start_test_server().await;

    let client = create_test_client();

    let request = CreateChatCompletionRequestArgs::default()
        .model(DEFAULT_MODEL)
        .messages(vec![ChatCompletionRequestMessage::User(
            async_openai::types::ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                    "What is 2+2? Reply with just the number.".to_string()
                ),
                name: None,
            },
        )])
        .build();

    assert!(request.is_ok(), "Failed to build request");
    let request = request.unwrap();

    let response = client.chat().create(request).await;

    if let Err(ref err) = response {
        eprintln!("Chat completion error: {:#?}", err);
        panic!("Chat completion failed");
    }

    let response = response.unwrap();
    assert!(!response.choices.is_empty(), "Response should have choices");

    let content = &response.choices[0].message.content;
    assert!(content.is_some(), "Response should have content");

    let content = content.as_ref().unwrap();
    assert!(!content.is_empty(), "Response content should not be empty");
    println!("Response: {}", content);

    // Verify response contains "4"
    assert!(
        content.contains("4"),
        "Response should contain the answer 4"
    );
}

#[tokio::test]
#[ignore] // Requires network access to DuckDuckGo
async fn test_server_streaming_chat_completion() {
    start_test_server().await;

    let client = create_test_client();

    let request = CreateChatCompletionRequestArgs::default()
        .model(DEFAULT_MODEL)
        .messages(vec![ChatCompletionRequestMessage::User(
            async_openai::types::ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                    "Say hello in one word".to_string()
                ),
                name: None,
            },
        )])
        .stream(true)
        .build();

    assert!(request.is_ok(), "Failed to build streaming request");
    let request = request.unwrap();

    let stream = client.chat().create_stream(request).await;

    if let Err(ref err) = stream {
        eprintln!("Streaming chat error: {:#?}", err);
        panic!("Streaming chat failed");
    }

    let mut stream = stream.unwrap();
    let mut full_content = String::new();
    let mut chunk_count = 0;

    use futures::StreamExt;
    while let Some(result) = stream.next().await {
        match result {
            Ok(chunk) => {
                chunk_count += 1;
                if let Some(choice) = chunk.choices.first() {
                    if let Some(content) = &choice.delta.content {
                        full_content.push_str(content);
                        print!("{}", content);
                    }
                }
            }
            Err(err) => {
                eprintln!("Stream error: {:#?}", err);
                break;
            }
        }
    }
    println!();

    assert!(chunk_count > 0, "Should receive at least one chunk");
    assert!(!full_content.is_empty(), "Should receive some content");
    println!("Received {} chunks", chunk_count);
    println!("Full response: {}", full_content);
}

#[tokio::test]
#[ignore] // Requires network access to DuckDuckGo
async fn test_server_with_invalid_api_key() {
    start_test_server().await;

    let config = OpenAIConfig::new()
        .with_api_key("invalid-api-key")
        .with_api_base(format!("http://127.0.0.1:{}/v1", TEST_PORT));

    let client = Client::with_config(config);

    let request = CreateChatCompletionRequestArgs::default()
        .model(DEFAULT_MODEL)
        .messages(vec![ChatCompletionRequestMessage::User(
            async_openai::types::ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                    "Hello".to_string()
                ),
                name: None,
            },
        )])
        .build()
        .unwrap();

    let response = client.chat().create(request).await;

    assert!(
        response.is_err(),
        "Request with invalid API key should fail"
    );
}

#[tokio::test]
#[ignore] // Requires network access to DuckDuckGo
async fn test_server_with_different_models() {
    start_test_server().await;

    let client = create_test_client();

    let models = vec![DEFAULT_MODEL, "claude-3-haiku"];

    for model in models {
        println!("Testing with model: {}", model);

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(vec![ChatCompletionRequestMessage::User(
                async_openai::types::ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        "Say hi in one word".to_string()
                    ),
                    name: None,
                },
            )])
            .build()
            .unwrap();

        let response = client.chat().create(request).await;

        if let Err(ref err) = response {
            eprintln!("Error with model {}: {:#?}", model, err);
            continue;
        }

        let response = response.unwrap();
        assert!(!response.choices.is_empty(), "Response should have choices");

        let content = response.choices[0].message.content.as_ref().unwrap();
        println!("Response from {}: {}", model, content);
        assert!(!content.is_empty(), "Response should not be empty");
    }
}

#[tokio::test]
#[ignore] // Requires network access to DuckDuckGo
async fn test_server_multiple_sequential_requests() {
    start_test_server().await;

    let client = create_test_client();

    for i in 1..=3 {
        println!("Request {}", i);

        let request = CreateChatCompletionRequestArgs::default()
            .model(DEFAULT_MODEL)
            .messages(vec![ChatCompletionRequestMessage::User(
                async_openai::types::ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        format!("Count to {}. Just output the numbers.", i)
                    ),
                    name: None,
                },
            )])
            .build()
            .unwrap();

        let response = client.chat().create(request).await;

        assert!(response.is_ok(), "Request {} should succeed", i);

        let response = response.unwrap();
        let content = response.choices[0].message.content.as_ref().unwrap();
        println!("Response {}: {}", i, content);
        assert!(!content.is_empty(), "Response should not be empty");

        // Small delay between requests
        sleep(Duration::from_millis(500)).await;
    }
}
