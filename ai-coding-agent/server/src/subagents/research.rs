use async_trait::async_trait;
use std::collections::HashMap;
use crate::models::{SubAgentType, Task};
use crate::templates::TemplateManager;
use super::SubAgent;

pub struct ResearchAgent;

impl ResearchAgent {
    pub fn new() -> Self {
        Self
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

    async fn execute(&self, task: &Task, templates: &TemplateManager) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut vars = HashMap::new();
        vars.insert("tech_stack".to_string(), "Rust, TypeScript, Preact".to_string());
        vars.insert("research_question".to_string(), task.description.clone());

        let _prompt = templates.render("research", &vars)
            .unwrap_or_else(|| task.description.clone());

        Ok(format!(
            "## Research Agent Response\n\n\
            **Question:** {}\n\n\
            **Analysis:** Researching topic...\n\n\
            **Findings:**\n\
            1. Investigated relevant documentation\n\
            2. Identified best practices\n\
            3. Found applicable examples\n\n\
            *Note: In production, this would search documentation and provide real findings.*",
            task.description
        ))
    }
}
