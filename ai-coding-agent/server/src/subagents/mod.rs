use crate::llm::LlmClient;
use crate::models::{SubAgentType, Task};
use crate::templates::TemplateManager;
use async_trait::async_trait;
use std::sync::Arc;

mod code_editor;
mod documentation;
mod research;
mod test_runner;

pub use code_editor::CodeEditorAgent;
pub use documentation::DocumentationAgent;
pub use research::ResearchAgent;
pub use test_runner::TestRunnerAgent;

#[async_trait]
pub trait SubAgent: Send + Sync {
    fn agent_type(&self) -> SubAgentType;
    fn name(&self) -> &str;
    fn llm_client(&self) -> &Arc<dyn LlmClient>;
    async fn execute(
        &self,
        task: &Task,
        templates: &TemplateManager,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
