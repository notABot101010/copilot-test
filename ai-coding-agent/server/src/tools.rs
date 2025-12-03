// Note: #![allow(dead_code)] is used here because these tools are scaffolded
// for future LLM integration but not yet connected to the orchestrator.
#![allow(dead_code)]

use crate::models::{ParameterSpec, ToolResult, ToolSpec};
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    fn spec(&self) -> ToolSpec;
    async fn execute(
        &self,
        params: Value,
    ) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct FileReadTool;
pub struct FileWriteTool;
pub struct ShellExecuteTool;
pub struct SearchCodeTool;

impl FileReadTool {
    pub fn new() -> Self {
        Self
    }
}

impl FileWriteTool {
    pub fn new() -> Self {
        Self
    }
}

impl ShellExecuteTool {
    pub fn new() -> Self {
        Self
    }
}

impl SearchCodeTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FileReadTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "file_read".to_string(),
            description: "Read contents of a file".to_string(),
            parameters: vec![ParameterSpec {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file to read".to_string(),
                required: true,
            }],
        }
    }

    async fn execute(
        &self,
        params: Value,
    ) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing path parameter")?;

        match tokio::fs::read_to_string(path).await {
            Ok(content) => Ok(ToolResult {
                success: true,
                output: content,
                error: None,
            }),
            Err(err) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(err.to_string()),
            }),
        }
    }
}

#[async_trait]
impl Tool for FileWriteTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "file_write".to_string(),
            description: "Write content to a file".to_string(),
            parameters: vec![
                ParameterSpec {
                    name: "path".to_string(),
                    param_type: "string".to_string(),
                    description: "Path to the file to write".to_string(),
                    required: true,
                },
                ParameterSpec {
                    name: "content".to_string(),
                    param_type: "string".to_string(),
                    description: "Content to write to the file".to_string(),
                    required: true,
                },
            ],
        }
    }

    async fn execute(
        &self,
        params: Value,
    ) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing path parameter")?;
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing content parameter")?;

        match tokio::fs::write(path, content).await {
            Ok(_) => Ok(ToolResult {
                success: true,
                output: format!("Successfully wrote to {}", path),
                error: None,
            }),
            Err(err) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(err.to_string()),
            }),
        }
    }
}

#[async_trait]
impl Tool for ShellExecuteTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "shell_execute".to_string(),
            description: "Execute a shell command".to_string(),
            parameters: vec![ParameterSpec {
                name: "command".to_string(),
                param_type: "string".to_string(),
                description: "Command to execute".to_string(),
                required: true,
            }],
        }
    }

    async fn execute(
        &self,
        params: Value,
    ) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let command = params
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or("Missing command parameter")?;

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(ToolResult {
            success: output.status.success(),
            output: stdout,
            error: if stderr.is_empty() {
                None
            } else {
                Some(stderr)
            },
        })
    }
}

#[async_trait]
impl Tool for SearchCodeTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "search_code".to_string(),
            description: "Search for code patterns in files".to_string(),
            parameters: vec![
                ParameterSpec {
                    name: "pattern".to_string(),
                    param_type: "string".to_string(),
                    description: "Pattern to search for".to_string(),
                    required: true,
                },
                ParameterSpec {
                    name: "directory".to_string(),
                    param_type: "string".to_string(),
                    description: "Directory to search in".to_string(),
                    required: false,
                },
            ],
        }
    }

    async fn execute(
        &self,
        params: Value,
    ) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let pattern = params
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or("Missing pattern parameter")?;
        let directory = params
            .get("directory")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let output = tokio::process::Command::new("grep")
            .args(["-rn", pattern, directory])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(ToolResult {
            success: true,
            output: stdout,
            error: None,
        })
    }
}

pub fn get_all_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(FileReadTool::new()),
        Box::new(FileWriteTool::new()),
        Box::new(ShellExecuteTool::new()),
        Box::new(SearchCodeTool::new()),
    ]
}
