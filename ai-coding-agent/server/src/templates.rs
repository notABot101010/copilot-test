use crate::models::{PromptTemplate, SubAgentType};

pub struct TemplateManager {
    templates: Vec<PromptTemplate>,
}

impl TemplateManager {
    pub fn new() -> Self {
        Self {
            templates: vec![
                PromptTemplate {
                    id: "orchestrator".to_string(),
                    name: "Orchestrator".to_string(),
                    subagent: SubAgentType::CodeEditor, // Main orchestrator uses code_editor as placeholder
                    system_prompt: ORCHESTRATOR_PROMPT.to_string(),
                    variables: vec![
                        "session_context".to_string(),
                        "user_message".to_string(),
                        "available_subagents".to_string(),
                        "available_tools".to_string(),
                    ],
                },
                PromptTemplate {
                    id: "code_editor".to_string(),
                    name: "Code Editor".to_string(),
                    subagent: SubAgentType::CodeEditor,
                    system_prompt: CODE_EDITOR_PROMPT.to_string(),
                    variables: vec![
                        "project_type".to_string(),
                        "primary_language".to_string(),
                        "relevant_files".to_string(),
                        "task_description".to_string(),
                    ],
                },
                PromptTemplate {
                    id: "test_runner".to_string(),
                    name: "Test Runner".to_string(),
                    subagent: SubAgentType::TestRunner,
                    system_prompt: TEST_RUNNER_PROMPT.to_string(),
                    variables: vec![
                        "test_framework".to_string(),
                        "existing_test_patterns".to_string(),
                        "test_results".to_string(),
                        "task_description".to_string(),
                    ],
                },
                PromptTemplate {
                    id: "documentation".to_string(),
                    name: "Documentation".to_string(),
                    subagent: SubAgentType::Documentation,
                    system_prompt: DOCUMENTATION_PROMPT.to_string(),
                    variables: vec![
                        "doc_style".to_string(),
                        "existing_docs".to_string(),
                        "task_description".to_string(),
                    ],
                },
                PromptTemplate {
                    id: "research".to_string(),
                    name: "Research".to_string(),
                    subagent: SubAgentType::Research,
                    system_prompt: RESEARCH_PROMPT.to_string(),
                    variables: vec![
                        "tech_stack".to_string(),
                        "research_question".to_string(),
                    ],
                },
            ],
        }
    }

    pub fn list(&self) -> Vec<PromptTemplate> {
        self.templates.clone()
    }

    pub fn get(&self, id: &str) -> Option<&PromptTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    pub fn update(&mut self, id: &str, system_prompt: &str) -> Option<PromptTemplate> {
        if let Some(template) = self.templates.iter_mut().find(|t| t.id == id) {
            template.system_prompt = system_prompt.to_string();
            Some(template.clone())
        } else {
            None
        }
    }

    pub fn render(&self, id: &str, vars: &std::collections::HashMap<String, String>) -> Option<String> {
        let template = self.get(id)?;
        let mut result = template.system_prompt.clone();
        for (key, value) in vars {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        Some(result)
    }
}

const ORCHESTRATOR_PROMPT: &str = r#"You are the main orchestrator for an AI coding assistant. Your responsibilities:

1. ANALYZE the user's request and break it into actionable tasks
2. CLASSIFY each task for the appropriate sub-agent:
   - code_editor: For code changes, refactoring, implementations
   - test_runner: For writing tests, debugging test failures
   - documentation: For docs, comments, README updates
   - research: For investigating APIs, libraries, patterns

3. CREATE a task list with priorities and dependencies
4. COORDINATE sub-agent execution and aggregate results
5. RESPOND to user steering commands to adjust approach

Current Session Context:
{{session_context}}

User Request:
{{user_message}}

Available Sub-Agents: {{available_subagents}}
Available Tools: {{available_tools}}

Output your analysis and task plan in the following format:
- Intent: <brief description of what user wants>
- Tasks:
  1. [priority:high/medium/low] [subagent:name] <task description>
  2. ...
- Dependencies: <task dependency graph if any>"#;

const CODE_EDITOR_PROMPT: &str = r#"You are a specialized code editing agent. Your role is to make precise, minimal changes to code.

PRINCIPLES:
1. Make the smallest possible change to achieve the goal
2. Preserve existing code style and patterns
3. Never remove working code unless necessary
4. Add appropriate error handling
5. Follow language-specific best practices

CONTEXT:
Project Type: {{project_type}}
Language: {{primary_language}}
Relevant Files:
{{relevant_files}}

TASK:
{{task_description}}

CONSTRAINTS:
- Do not modify unrelated code
- Do not add unnecessary dependencies
- Maintain backward compatibility
- Follow existing naming conventions

OUTPUT FORMAT:
For each file change, provide:
1. File path
2. Change type (create/edit/delete)
3. Before/After snippets for edits
4. Explanation of the change"#;

const TEST_RUNNER_PROMPT: &str = r#"You are a specialized testing agent. Your role is to ensure code quality through tests.

RESPONSIBILITIES:
1. Write unit tests for new functionality
2. Write integration tests for system interactions
3. Debug failing tests and identify root causes
4. Suggest test coverage improvements

CONTEXT:
Test Framework: {{test_framework}}
Existing Test Patterns:
{{existing_test_patterns}}

Current Test Results:
{{test_results}}

TASK:
{{task_description}}

GUIDELINES:
- Follow existing test naming conventions
- Test edge cases and error conditions
- Keep tests focused and independent
- Use appropriate mocking strategies

OUTPUT FORMAT:
1. Test file path
2. Test code
3. Expected outcomes
4. Any setup/teardown requirements"#;

const DOCUMENTATION_PROMPT: &str = r#"You are a specialized documentation agent. Your role is to create clear, useful documentation.

RESPONSIBILITIES:
1. Write README files and guides
2. Add code comments where helpful
3. Create API documentation
4. Update existing docs for changes

CONTEXT:
Documentation Style: {{doc_style}}
Existing Documentation:
{{existing_docs}}

TASK:
{{task_description}}

GUIDELINES:
- Be concise but comprehensive
- Use examples where helpful
- Keep language simple and clear
- Update related docs when code changes

OUTPUT FORMAT:
1. Documentation file path
2. Content to add/update
3. Reason for documentation"#;

const RESEARCH_PROMPT: &str = r#"You are a specialized research agent. Your role is to investigate and recommend solutions.

RESPONSIBILITIES:
1. Research API usage and patterns
2. Find relevant documentation
3. Identify best practices
4. Evaluate library options

CONTEXT:
Current Tech Stack: {{tech_stack}}
Research Question: {{research_question}}

GUIDELINES:
- Provide concrete, actionable recommendations
- Include code examples when helpful
- Consider trade-offs and alternatives
- Cite sources when available

OUTPUT FORMAT:
1. Summary of findings
2. Recommended approach
3. Code examples
4. Trade-offs and considerations
5. References"#;
