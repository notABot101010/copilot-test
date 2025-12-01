use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use crate::llm::{ChatMessage, LlmClient};
use crate::models::{SubAgentType, Task};
use crate::templates::TemplateManager;
use super::SubAgent;

pub struct DocumentationAgent {
    llm_client: Arc<dyn LlmClient>,
}

impl DocumentationAgent {
    pub fn new(llm_client: Arc<dyn LlmClient>) -> Self {
        Self { llm_client }
    }
}

#[async_trait]
impl SubAgent for DocumentationAgent {
    fn agent_type(&self) -> SubAgentType {
        SubAgentType::Documentation
    }

    fn name(&self) -> &str {
        "Documentation"
    }

    fn llm_client(&self) -> &Arc<dyn LlmClient> {
        &self.llm_client
    }

    async fn execute(&self, task: &Task, templates: &TemplateManager) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut vars = HashMap::new();
        vars.insert("doc_style".to_string(), "Markdown".to_string());
        vars.insert("existing_docs".to_string(), "(docs would be analyzed)".to_string());
        vars.insert("task_description".to_string(), task.description.clone());

        let system_prompt = templates.render("documentation", &vars)
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
                    "## Documentation Agent Response\n\n\
                    **Task:** {}\n\n\
                    **Analysis:** Analyzing documentation needs...\n\n\
                    **Documentation Plan:**\n\
                    1. Identified documentation to update\n\
                    2. Following existing documentation style\n\
                    3. Including examples where helpful\n\n\
                    *Note: LLM unavailable, showing placeholder response.*",
                    task.description
                ))
            }
        }
    }
}
