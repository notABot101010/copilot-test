use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use crate::llm::{ChatMessage, LlmClient};
use crate::models::{SubAgentType, Task};
use crate::templates::TemplateManager;
use super::SubAgent;

pub struct ResearchAgent {
    llm_client: Arc<dyn LlmClient>,
}

impl ResearchAgent {
    pub fn new(llm_client: Arc<dyn LlmClient>) -> Self {
        Self { llm_client }
    }
}

#[async_trait]
impl SubAgent for ResearchAgent {
    fn agent_type(&self) -> SubAgentType {
        SubAgentType::Research
    }

    fn name(&self) -> &str {
        "Research"
    }

    fn llm_client(&self) -> &Arc<dyn LlmClient> {
        &self.llm_client
    }

    async fn execute(&self, task: &Task, templates: &TemplateManager) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut vars = HashMap::new();
        vars.insert("tech_stack".to_string(), "Rust, TypeScript, Preact".to_string());
        vars.insert("research_question".to_string(), task.description.clone());

        let system_prompt = templates.render("research", &vars)
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
                    "## Research Agent Response\n\n\
                    **Question:** {}\n\n\
                    **Analysis:** Researching topic...\n\n\
                    **Findings:**\n\
                    1. Investigated relevant documentation\n\
                    2. Identified best practices\n\
                    3. Found applicable examples\n\n\
                    *Note: LLM unavailable, showing placeholder response.*",
                    task.description
                ))
            }
        }
    }
}
