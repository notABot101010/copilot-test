//! LLM Client module for interacting with OpenAI-compatible APIs.
//! 
//! This module provides a configurable LLM client that can connect to
//! any OpenAI-compatible API endpoint (OpenAI, Azure, local models, etc.)

use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, CreateChatCompletionRequest,
        CreateChatCompletionResponse,
    },
    Client,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Error type for LLM operations
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("No response content")]
    NoContent,
}

/// Configuration for the LLM client
#[derive(Clone, Debug)]
pub struct LlmConfig {
    /// Base URL for the API (defaults to OpenAI's API)
    pub api_base: String,
    /// API key for authentication
    pub api_key: String,
    /// Model to use for completions
    pub model: String,
    /// Maximum tokens for responses
    pub max_tokens: Option<u32>,
    /// Temperature for response randomness (0.0 - 2.0)
    pub temperature: Option<f32>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_base: std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            model: std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4".to_string()),
            max_tokens: Some(4096),
            temperature: Some(0.7),
        }
    }
}

impl LlmConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = api_base.into();
        self
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

/// A message in a conversation
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

/// Role of a message sender
#[derive(Clone, Debug, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

/// Trait for LLM client implementations
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Complete a chat conversation
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String, LlmError>;

    /// Complete a single prompt with a system message
    async fn complete(&self, system_prompt: &str, user_message: &str) -> Result<String, LlmError> {
        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_message),
        ];
        self.chat(messages).await
    }
}

/// OpenAI-compatible LLM client using async-openai
pub struct OpenAiLlmClient {
    client: Client<OpenAIConfig>,
    config: LlmConfig,
}

impl OpenAiLlmClient {
    pub fn new(config: LlmConfig) -> Self {
        let openai_config = OpenAIConfig::new()
            .with_api_base(&config.api_base)
            .with_api_key(&config.api_key);

        let client = Client::with_config(openai_config);

        Self { client, config }
    }

    fn convert_messages(&self, messages: &[ChatMessage]) -> Vec<ChatCompletionRequestMessage> {
        messages
            .iter()
            .map(|msg| match msg.role {
                MessageRole::System => {
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                        content: msg.content.clone().into(),
                        name: None,
                    })
                }
                MessageRole::User => {
                    ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                        content: msg.content.clone().into(),
                        name: None,
                    })
                }
                MessageRole::Assistant => {
                    #[allow(deprecated)]
                    ChatCompletionRequestMessage::Assistant(
                        async_openai::types::chat::ChatCompletionRequestAssistantMessage {
                            content: Some(msg.content.clone().into()),
                            refusal: None,
                            name: None,
                            audio: None,
                            tool_calls: None,
                            function_call: None,
                        },
                    )
                }
            })
            .collect()
    }
}

#[async_trait]
impl LlmClient for OpenAiLlmClient {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String, LlmError> {
        let request = CreateChatCompletionRequest {
            model: self.config.model.clone(),
            messages: self.convert_messages(&messages),
            max_completion_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            ..Default::default()
        };

        let response: CreateChatCompletionResponse = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|err| LlmError::ApiError(err.to_string()))?;

        response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or(LlmError::NoContent)
    }
}

/// Mock LLM client for testing
pub struct MockLlmClient {
    responses: Arc<tokio::sync::RwLock<Vec<String>>>,
    call_count: Arc<tokio::sync::RwLock<usize>>,
}

impl MockLlmClient {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            call_count: Arc::new(tokio::sync::RwLock::new(0)),
        }
    }

    pub fn with_responses(responses: Vec<String>) -> Self {
        Self {
            responses: Arc::new(tokio::sync::RwLock::new(responses)),
            call_count: Arc::new(tokio::sync::RwLock::new(0)),
        }
    }

    pub async fn add_response(&self, response: impl Into<String>) {
        self.responses.write().await.push(response.into());
    }

    pub async fn call_count(&self) -> usize {
        *self.call_count.read().await
    }
}

impl Default for MockLlmClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn chat(&self, _messages: Vec<ChatMessage>) -> Result<String, LlmError> {
        let mut count = self.call_count.write().await;
        let responses = self.responses.read().await;
        
        let response = responses
            .get(*count)
            .cloned()
            .unwrap_or_else(|| "Mock response".to_string());
        
        *count += 1;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_client_basic() {
        let client = MockLlmClient::new();
        let result = client.chat(vec![ChatMessage::user("Hello")]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Mock response");
    }

    #[tokio::test]
    async fn test_mock_client_with_responses() {
        let client = MockLlmClient::with_responses(vec![
            "First response".to_string(),
            "Second response".to_string(),
        ]);

        let result1 = client.chat(vec![ChatMessage::user("Hello")]).await;
        assert_eq!(result1.unwrap(), "First response");

        let result2 = client.chat(vec![ChatMessage::user("World")]).await;
        assert_eq!(result2.unwrap(), "Second response");

        // Third call should return default since no more responses
        let result3 = client.chat(vec![ChatMessage::user("Test")]).await;
        assert_eq!(result3.unwrap(), "Mock response");
    }

    #[tokio::test]
    async fn test_mock_client_call_count() {
        let client = MockLlmClient::new();
        
        assert_eq!(client.call_count().await, 0);
        
        let _ = client.chat(vec![ChatMessage::user("Hello")]).await;
        assert_eq!(client.call_count().await, 1);
        
        let _ = client.chat(vec![ChatMessage::user("World")]).await;
        assert_eq!(client.call_count().await, 2);
    }

    #[tokio::test]
    async fn test_mock_client_complete() {
        let client = MockLlmClient::with_responses(vec!["Completed task".to_string()]);
        
        let result = client.complete("You are helpful", "Help me").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Completed task");
    }

    #[test]
    fn test_chat_message_constructors() {
        let system = ChatMessage::system("System prompt");
        assert_eq!(system.role, MessageRole::System);
        assert_eq!(system.content, "System prompt");

        let user = ChatMessage::user("User message");
        assert_eq!(user.role, MessageRole::User);
        assert_eq!(user.content, "User message");

        let assistant = ChatMessage::assistant("Assistant response");
        assert_eq!(assistant.role, MessageRole::Assistant);
        assert_eq!(assistant.content, "Assistant response");
    }

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert!(!config.api_base.is_empty());
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.max_tokens, Some(4096));
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_llm_config_builder() {
        let config = LlmConfig::new()
            .with_api_base("http://localhost:8080/v1")
            .with_api_key("test-key")
            .with_model("gpt-3.5-turbo")
            .with_max_tokens(2048)
            .with_temperature(0.5);

        assert_eq!(config.api_base, "http://localhost:8080/v1");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.model, "gpt-3.5-turbo");
        assert_eq!(config.max_tokens, Some(2048));
        assert_eq!(config.temperature, Some(0.5));
    }
}
