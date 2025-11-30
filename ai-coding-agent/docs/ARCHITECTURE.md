# AI Coding Agent Architecture

## Overview

This document describes the architecture for an asynchronous AI coding agent inspired by GitHub Copilot. The agent orchestrates multiple specialized sub-agents, tools, memory systems, and sandboxes to assist developers in coding tasks.

## Core Components

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              User Interface                                   │
│                        (Preact/TypeScript + TailwindCSS)                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                            API Gateway (Axum)                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                          Session Manager                                      │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  Manages conversation sessions, message history, and context          │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────────┤
│                        Orchestrator (Main Agent)                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  Routes tasks to specialized sub-agents based on intent classification  │   │
│  │  Manages work queue, prioritization, and parallel execution            │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
├────────────────┬────────────────┬───────────────┬───────────────────────────┤
│ Code Editor    │ Test Runner    │ Documentation │  Research Agent           │
│ Sub-Agent      │ Sub-Agent      │ Sub-Agent     │                           │
├────────────────┴────────────────┴───────────────┴───────────────────────────┤
│                            Tool Registry                                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐  │
│  │ File Ops │ │ Git Ops  │ │ Shell    │ │ Search   │ │ External APIs    │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────────────┘  │
├─────────────────────────────────────────────────────────────────────────────┤
│                         Memory / MCP Servers                                  │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────────┐  │
│  │ Conversation     │  │ Code Index       │  │ External Knowledge       │  │
│  │ Memory           │  │ Memory           │  │ (GitHub, Docs, etc.)     │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────────┤
│                           Sandbox Manager                                     │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  Isolated execution environments for running code, tests, builds      │   │
│  │  Container-based isolation with resource limits                       │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Sub-Agent System

### Orchestrator (Main Agent)

The orchestrator is responsible for:
1. **Intent Classification**: Analyzing user requests to determine which sub-agents are needed
2. **Task Decomposition**: Breaking complex requests into manageable tasks
3. **Work Queue Management**: Prioritizing and scheduling tasks
4. **Result Aggregation**: Combining results from multiple sub-agents
5. **User Steering**: Responding to real-time user guidance

**Orchestrator System Prompt Template:**
```
You are the main orchestrator for an AI coding assistant. Your responsibilities:

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
- Dependencies: <task dependency graph if any>
```

### Code Editor Sub-Agent

Specialized in making code changes with minimal modifications.

**Code Editor System Prompt Template:**
```
You are a specialized code editing agent. Your role is to make precise, minimal changes to code.

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
4. Explanation of the change
```

### Test Runner Sub-Agent

Specialized in testing, debugging, and quality assurance.

**Test Runner System Prompt Template:**
```
You are a specialized testing agent. Your role is to ensure code quality through tests.

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
4. Any setup/teardown requirements
```

### Documentation Sub-Agent

Specialized in documentation and comments.

**Documentation System Prompt Template:**
```
You are a specialized documentation agent. Your role is to create clear, useful documentation.

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
3. Reason for documentation
```

### Research Sub-Agent

Specialized in researching solutions and patterns.

**Research System Prompt Template:**
```
You are a specialized research agent. Your role is to investigate and recommend solutions.

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
5. References
```

## Task Routing Algorithm

```
1. Receive user message
2. Extract intent using LLM classification
3. Create task queue based on intent:
   - For "implement feature": code_editor → test_runner → documentation
   - For "fix bug": research → code_editor → test_runner
   - For "add tests": test_runner
   - For "update docs": documentation
4. Execute tasks respecting dependencies
5. Aggregate results and present to user
6. Accept steering commands to modify approach
```

## Memory/MCP Server Integration

### Conversation Memory
- Stores full conversation history per session
- Provides context window management
- Supports semantic search over past messages

### Code Index Memory
- Indexes project files for fast retrieval
- Tracks file changes and versions
- Enables code-aware context selection

### External Knowledge (MCP Servers)
- GitHub: Repository access, PR management
- Documentation servers: Language docs, API refs
- Search servers: Web search for solutions

## Sandbox Architecture

Sandboxes provide isolated execution environments:

```
┌─────────────────────────────────────┐
│           Sandbox Manager            │
├─────────────────────────────────────┤
│ ┌─────────┐ ┌─────────┐ ┌─────────┐│
│ │Sandbox 1│ │Sandbox 2│ │Sandbox N││
│ │ (Build) │ │ (Test)  │ │ (Shell) ││
│ └─────────┘ └─────────┘ └─────────┘│
├─────────────────────────────────────┤
│         Container Runtime            │
│       (Resource Limits, Isolation)   │
└─────────────────────────────────────┘
```

Each sandbox:
- Runs in isolated container/process
- Has resource limits (CPU, memory, time)
- Has read access to relevant project files
- Can execute commands and capture output
- Reports results back to orchestrator

## Tool Registry

Tools are registered with the following interface:

```rust
trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Vec<ParameterSpec>;
    async fn execute(&self, params: Value) -> Result<ToolResult>;
}
```

Built-in tools:
- **file_read**: Read file contents
- **file_write**: Create or update files
- **file_delete**: Remove files
- **shell_execute**: Run shell commands
- **git_status**: Get git repository status
- **git_diff**: Show changes
- **git_commit**: Commit changes
- **search_code**: Search project files
- **search_web**: Search external resources

## User Steering

Users can steer the agent while it's working:

1. **Cancel**: Stop current task
2. **Pause**: Pause execution for review
3. **Modify**: Change approach mid-execution
4. **Prioritize**: Adjust task priorities
5. **Focus**: Direct agent to specific aspect

Steering is implemented via WebSocket for real-time communication.

## API Endpoints

- `POST /api/sessions` - Create new session
- `GET /api/sessions` - List all sessions
- `GET /api/sessions/:id` - Get session details
- `POST /api/sessions/:id/messages` - Send message
- `GET /api/sessions/:id/messages` - Get messages
- `POST /api/sessions/:id/steer` - Send steering command
- `GET /api/templates` - List prompt templates
- `PUT /api/templates/:id` - Update prompt template
- `WS /api/sessions/:id/stream` - Real-time updates

## Configuration

Prompt templates are stored in YAML format for easy customization:

```yaml
templates:
  orchestrator:
    system: |
      You are the main orchestrator...
    variables:
      - session_context
      - user_message
      - available_subagents
      - available_tools
  
  code_editor:
    system: |
      You are a specialized code editor...
    variables:
      - project_type
      - primary_language
      - relevant_files
      - task_description
```

## Future Enhancements

1. **Multi-agent collaboration**: Allow sub-agents to communicate
2. **Learning from feedback**: Improve based on user corrections
3. **Custom sub-agents**: User-defined specialized agents
4. **Plugin system**: Extensible tool and MCP server support
5. **Distributed execution**: Scale across multiple machines
