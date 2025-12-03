use super::SubAgent;
use crate::llm::{ChatMessage, LlmClient};
use crate::models::{SubAgentType, Task};
use crate::templates::TemplateManager;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct CodeEditorAgent {
    llm_client: Arc<dyn LlmClient>,
}

impl CodeEditorAgent {
    pub fn new(llm_client: Arc<dyn LlmClient>) -> Self {
        Self { llm_client }
    }
}

#[async_trait]
impl SubAgent for CodeEditorAgent {
    fn agent_type(&self) -> SubAgentType {
        SubAgentType::CodeEditor
    }

    fn name(&self) -> &str {
        "Code Editor"
    }

    fn llm_client(&self) -> &Arc<dyn LlmClient> {
        &self.llm_client
    }

    async fn execute(
        &self,
        task: &Task,
        templates: &TemplateManager,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Build context for template
        let mut vars = HashMap::new();
        vars.insert("project_type".to_string(), "Rust/TypeScript".to_string());
        vars.insert("primary_language".to_string(), "Rust".to_string());
        vars.insert(
            "relevant_files".to_string(),
            "(files would be listed here)".to_string(),
        );
        vars.insert("task_description".to_string(), task.description.clone());

        // Render prompt template
        let system_prompt = templates
            .render("code_editor", &vars)
            .unwrap_or_else(|| task.description.clone());

        // Use LLM to generate response
        let messages = vec![
            ChatMessage::system(&system_prompt),
            ChatMessage::user(&task.description),
        ];

        match self.llm_client.chat(messages).await {
            Ok(response) => Ok(response),
            Err(err) => {
                tracing::warn!("LLM call failed, returning fallback response: {}", err);
                Ok(format!(
                    "## Code Editor Agent Response\n\n\
                    **Task:** {}\n\n\
                    **Analysis:** Analyzing code changes needed...\n\n\
                    **Proposed Changes:**\n\
                    1. Identified target files for modification\n\
                    2. Prepared minimal changes to implement request\n\
                    3. Following existing code patterns and style\n\n\
                    *Note: LLM unavailable, showing placeholder response.*",
                    task.description
                ))
            }
        }
    }
}
