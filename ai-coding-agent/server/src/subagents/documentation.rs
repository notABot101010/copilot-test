use async_trait::async_trait;
use std::collections::HashMap;
use crate::models::{SubAgentType, Task};
use crate::templates::TemplateManager;
use super::SubAgent;

pub struct DocumentationAgent;

impl DocumentationAgent {
    pub fn new() -> Self {
        Self
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

    async fn execute(&self, task: &Task, templates: &TemplateManager) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut vars = HashMap::new();
        vars.insert("doc_style".to_string(), "Markdown".to_string());
        vars.insert("existing_docs".to_string(), "(docs would be analyzed)".to_string());
        vars.insert("task_description".to_string(), task.description.clone());

        let _prompt = templates.render("documentation", &vars)
            .unwrap_or_else(|| task.description.clone());

        Ok(format!(
            "## Documentation Agent Response\n\n\
            **Task:** {}\n\n\
            **Analysis:** Analyzing documentation needs...\n\n\
            **Documentation Plan:**\n\
            1. Identified documentation to update\n\
            2. Following existing documentation style\n\
            3. Including examples where helpful\n\n\
            *Note: In production, this would generate actual documentation.*",
            task.description
        ))
    }
}
