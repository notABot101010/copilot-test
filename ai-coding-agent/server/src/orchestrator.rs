use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use sqlx::sqlite::SqlitePool;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::models::{
    SteerCommand, StreamEvent, StreamEventType, SubAgentType, Task, TaskPriority, TaskStatus,
};
use crate::subagents::SubAgent;
use crate::templates::TemplateManager;

pub struct Orchestrator {
    sessions: RwLock<HashMap<String, SessionState>>,
    subagents: Vec<Box<dyn SubAgent>>,
}

struct SessionState {
    tasks: Vec<Task>,
    status: SessionStatus,
    sender: broadcast::Sender<StreamEvent>,
}

#[derive(Clone, PartialEq)]
enum SessionStatus {
    Idle,
    Running,
    Paused,
    Cancelled,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            subagents: vec![
                Box::new(crate::subagents::CodeEditorAgent::new()),
                Box::new(crate::subagents::TestRunnerAgent::new()),
                Box::new(crate::subagents::DocumentationAgent::new()),
                Box::new(crate::subagents::ResearchAgent::new()),
            ],
        }
    }

    pub async fn process_message(
        &self,
        db: &SqlitePool,
        templates: &Arc<RwLock<TemplateManager>>,
        session_id: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get or create session state
        let sender = {
            let mut sessions = self.sessions.write().await;
            let state = sessions.entry(session_id.to_string()).or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                SessionState {
                    tasks: Vec::new(),
                    status: SessionStatus::Idle,
                    sender: tx,
                }
            });
            state.status = SessionStatus::Running;
            state.sender.clone()
        };

        // Emit thinking event
        let _ = sender.send(StreamEvent {
            event_type: StreamEventType::AgentThinking,
            data: serde_json::json!({ "message": "Analyzing request..." }),
        });

        // Classify intent and create tasks
        let tasks = self.classify_and_plan(content).await;
        
        // Store tasks in session
        {
            let mut sessions = self.sessions.write().await;
            if let Some(state) = sessions.get_mut(session_id) {
                state.tasks = tasks.clone();
            }
        }

        // Execute tasks
        for task in tasks {
            // Check if cancelled or paused
            {
                let sessions = self.sessions.read().await;
                if let Some(state) = sessions.get(session_id) {
                    match state.status {
                        SessionStatus::Cancelled => {
                            let _ = sender.send(StreamEvent {
                                event_type: StreamEventType::TaskFailed,
                                data: serde_json::json!({ "reason": "cancelled" }),
                            });
                            break;
                        }
                        SessionStatus::Paused => {
                            // Wait for resume (simplified - in real impl would use condition variable)
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                            continue;
                        }
                        _ => {}
                    }
                }
            }

            // Emit task started event
            let _ = sender.send(StreamEvent {
                event_type: StreamEventType::TaskStarted,
                data: serde_json::json!({
                    "task_id": task.id,
                    "subagent": task.subagent.to_string(),
                    "description": task.description,
                }),
            });

            // Find appropriate subagent
            if let Some(agent) = self.subagents.iter().find(|a| a.agent_type() == task.subagent) {
                let templates_guard = templates.read().await;
                let result = agent.execute(&task, &templates_guard).await;
                
                // Store result in database
                let result_content = match &result {
                    Ok(r) => r.clone(),
                    Err(e) => format!("Error: {}", e),
                };

                // Create assistant message with result
                let msg_id = Uuid::new_v4().to_string();
                let now = Utc::now();
                let _ = sqlx::query(
                    r#"
                    INSERT INTO messages (id, session_id, role, content, created_at, metadata)
                    VALUES (?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(&msg_id)
                .bind(session_id)
                .bind("assistant")
                .bind(&result_content)
                .bind(&now.to_rfc3339())
                .bind(serde_json::to_string(&serde_json::json!({
                    "task_id": task.id,
                    "subagent": task.subagent.to_string(),
                })).ok())
                .execute(db)
                .await;

                // Emit completion event
                let event_type = if result.is_ok() {
                    StreamEventType::TaskCompleted
                } else {
                    StreamEventType::TaskFailed
                };

                let _ = sender.send(StreamEvent {
                    event_type,
                    data: serde_json::json!({
                        "task_id": task.id,
                        "result": result_content,
                    }),
                });
            }
        }

        // Final response
        let _ = sender.send(StreamEvent {
            event_type: StreamEventType::AgentResponse,
            data: serde_json::json!({ "message": "Tasks completed" }),
        });

        // Mark session as idle
        {
            let mut sessions = self.sessions.write().await;
            if let Some(state) = sessions.get_mut(session_id) {
                state.status = SessionStatus::Idle;
            }
        }

        Ok(())
    }

    async fn classify_and_plan(&self, content: &str) -> Vec<Task> {
        // Simple intent classification (in production, use LLM)
        let mut tasks = Vec::new();
        let content_lower = content.to_lowercase();

        // Determine which subagents are needed based on keywords
        if content_lower.contains("implement") || content_lower.contains("code") || 
           content_lower.contains("add") || content_lower.contains("create") ||
           content_lower.contains("fix") || content_lower.contains("change") {
            tasks.push(Task {
                id: Uuid::new_v4().to_string(),
                session_id: String::new(),
                subagent: SubAgentType::CodeEditor,
                description: format!("Code changes for: {}", content),
                priority: TaskPriority::High,
                status: TaskStatus::Pending,
                result: None,
            });
        }

        if content_lower.contains("test") || content_lower.contains("verify") {
            tasks.push(Task {
                id: Uuid::new_v4().to_string(),
                session_id: String::new(),
                subagent: SubAgentType::TestRunner,
                description: format!("Tests for: {}", content),
                priority: TaskPriority::Medium,
                status: TaskStatus::Pending,
                result: None,
            });
        }

        if content_lower.contains("doc") || content_lower.contains("readme") ||
           content_lower.contains("comment") {
            tasks.push(Task {
                id: Uuid::new_v4().to_string(),
                session_id: String::new(),
                subagent: SubAgentType::Documentation,
                description: format!("Documentation for: {}", content),
                priority: TaskPriority::Low,
                status: TaskStatus::Pending,
                result: None,
            });
        }

        if content_lower.contains("research") || content_lower.contains("investigate") ||
           content_lower.contains("find") || content_lower.contains("how") {
            tasks.push(Task {
                id: Uuid::new_v4().to_string(),
                session_id: String::new(),
                subagent: SubAgentType::Research,
                description: format!("Research for: {}", content),
                priority: TaskPriority::High,
                status: TaskStatus::Pending,
                result: None,
            });
        }

        // Default to code editor if no specific intent detected
        if tasks.is_empty() {
            tasks.push(Task {
                id: Uuid::new_v4().to_string(),
                session_id: String::new(),
                subagent: SubAgentType::CodeEditor,
                description: content.to_string(),
                priority: TaskPriority::Medium,
                status: TaskStatus::Pending,
                result: None,
            });
        }

        tasks
    }

    pub async fn steer(&self, session_id: &str, command: SteerCommand) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(state) = sessions.get_mut(session_id) {
            match command {
                SteerCommand::Cancel => {
                    state.status = SessionStatus::Cancelled;
                }
                SteerCommand::Pause => {
                    state.status = SessionStatus::Paused;
                }
                SteerCommand::Resume => {
                    state.status = SessionStatus::Running;
                }
                _ => {
                    // Handle other commands
                }
            }
            
            // Notify subscribers
            let _ = state.sender.send(StreamEvent {
                event_type: StreamEventType::AgentThinking,
                data: serde_json::json!({ "steering": format!("{:?}", command) }),
            });
        }

        Ok(())
    }

    pub fn subscribe(&self, session_id: &str) -> broadcast::Receiver<StreamEvent> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let mut sessions = self.sessions.write().await;
            let state = sessions.entry(session_id.to_string()).or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                SessionState {
                    tasks: Vec::new(),
                    status: SessionStatus::Idle,
                    sender: tx,
                }
            });
            state.sender.subscribe()
        })
    }

    pub fn unsubscribe(&self, _session_id: &str) {
        // Cleanup if needed
    }
}
