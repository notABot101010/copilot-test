use async_trait::async_trait;
use std::sync::Arc;
use crate::llm::LlmClient;
use crate::models::{SubAgentType, Task};
use crate::templates::TemplateManager;

mod code_editor;
mod test_runner;
mod documentation;
mod research;

pub use code_editor::CodeEditorAgent;
pub use test_runner::TestRunnerAgent;
pub use documentation::DocumentationAgent;
pub use research::ResearchAgent;

#[async_trait]
pub trait SubAgent: Send + Sync {
    fn agent_type(&self) -> SubAgentType;
    fn name(&self) -> &str;
    fn llm_client(&self) -> &Arc<dyn LlmClient>;
    async fn execute(&self, task: &Task, templates: &TemplateManager) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
