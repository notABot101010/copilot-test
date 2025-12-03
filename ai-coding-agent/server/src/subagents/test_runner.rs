use super::SubAgent;
use crate::llm::{ChatMessage, LlmClient};
use crate::models::{SubAgentType, Task};
use crate::templates::TemplateManager;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TestRunnerAgent {
    llm_client: Arc<dyn LlmClient>,
}

impl TestRunnerAgent {
    pub fn new(llm_client: Arc<dyn LlmClient>) -> Self {
        Self { llm_client }
    }
}

#[async_trait]
impl SubAgent for TestRunnerAgent {
    fn agent_type(&self) -> SubAgentType {
        SubAgentType::TestRunner
    }

    fn name(&self) -> &str {
        "Test Runner"
    }

    fn llm_client(&self) -> &Arc<dyn LlmClient> {
        &self.llm_client
    }

    async fn execute(
        &self,
        task: &Task,
        templates: &TemplateManager,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut vars = HashMap::new();
        vars.insert(
            "test_framework".to_string(),
            "cargo test / vitest".to_string(),
        );
        vars.insert(
            "existing_test_patterns".to_string(),
            "(patterns would be analyzed)".to_string(),
        );
        vars.insert("test_results".to_string(), "(would run tests)".to_string());
        vars.insert("task_description".to_string(), task.description.clone());

        let system_prompt = templates
            .render("test_runner", &vars)
            .unwrap_or_else(|| task.description.clone());

        let messages = vec![
            ChatMessage::system(&system_prompt),
            ChatMessage::user(&task.description),
        ];

        match self.llm_client.chat(messages).await {
            Ok(response) => Ok(response),
            Err(err) => {
                tracing::warn!("LLM call failed, returning fallback response: {}", err);
                Ok(format!(
                    "## Test Runner Agent Response\n\n\
                    **Task:** {}\n\n\
                    **Analysis:** Analyzing test requirements...\n\n\
                    **Test Plan:**\n\
                    1. Identified test cases needed\n\
                    2. Following existing test patterns\n\
                    3. Coverage for edge cases included\n\n\
                    *Note: LLM unavailable, showing placeholder response.*",
                    task.description
                ))
            }
        }
    }
}
