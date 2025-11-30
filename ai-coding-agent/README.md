# AI Coding Agent

An asynchronous AI coding agent MVP inspired by GitHub Copilot Coding Agent. This project demonstrates how to build an orchestrated multi-agent system with specialized sub-agents for different coding tasks.

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture documentation.

### Key Components

- **Orchestrator**: Main agent that analyzes user requests, classifies intent, and routes tasks to specialized sub-agents
- **Sub-Agents**: Specialized agents for different tasks
  - Code Editor: Makes precise, minimal code changes
  - Test Runner: Writes and runs tests
  - Documentation: Creates and updates documentation
  - Research: Investigates APIs and patterns
- **Tool Registry**: Extensible set of tools (file operations, shell, search)
- **Template System**: Customizable prompts for each sub-agent
- **WebSocket Streaming**: Real-time updates during task execution

## Project Structure

```
ai-coding-agent/
├── docs/
│   └── ARCHITECTURE.md    # Detailed architecture documentation
├── server/                # Rust backend (Axum)
│   └── src/
│       ├── main.rs        # Server entry point and routes
│       ├── db.rs          # SQLite database initialization
│       ├── models.rs      # Data models and types
│       ├── handlers/      # HTTP request handlers
│       ├── orchestrator.rs # Main agent orchestration logic
│       ├── subagents/     # Specialized sub-agent implementations
│       ├── templates.rs   # Prompt template management
│       └── tools.rs       # Tool definitions
└── webapp/                # Preact/TypeScript frontend
    └── src/
        ├── api.ts         # API client
        ├── types.ts       # TypeScript types
        ├── components/    # UI components
        └── pages/         # Page components
```

## Features

### Session Management
- Create new conversation sessions
- View list of all sessions
- Continue previous conversations

### Conversation Interface
- Send prompts to the agent
- View agent responses with task results
- Real-time streaming updates via WebSocket

### User Steering
- Pause/Resume agent execution
- Cancel running tasks
- Focus agent on specific aspects

### Prompt Templates
- View all sub-agent prompts
- Edit prompts with variable placeholders
- Save customizations

## Running the Project

### Backend (Rust)

```bash
cd ai-coding-agent/server
cargo run -- --port 8080
```

Options:
- `-H, --host`: Host address (default: 0.0.0.0)
- `-p, --port`: Port number (default: 8080)
- `--database`: SQLite database path (default: agent.db)

### Frontend (Preact)

```bash
cd ai-coding-agent/webapp
npm install
npm run dev
```

The webapp runs on `http://localhost:4000` and proxies API requests to the backend.

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/sessions` | Create new session |
| GET | `/api/sessions` | List all sessions |
| GET | `/api/sessions/:id` | Get session details |
| POST | `/api/sessions/:id/messages` | Send message |
| GET | `/api/sessions/:id/messages` | Get messages |
| POST | `/api/sessions/:id/steer` | Send steering command |
| WS | `/api/sessions/:id/stream` | WebSocket for real-time updates |
| GET | `/api/templates` | List prompt templates |
| PUT | `/api/templates/:id` | Update template |

## Sub-Agent Prompt Templates

Each sub-agent has a customizable system prompt with variable placeholders:

### Orchestrator
Variables: `session_context`, `user_message`, `available_subagents`, `available_tools`

### Code Editor
Variables: `project_type`, `primary_language`, `relevant_files`, `task_description`

### Test Runner
Variables: `test_framework`, `existing_test_patterns`, `test_results`, `task_description`

### Documentation
Variables: `doc_style`, `existing_docs`, `task_description`

### Research
Variables: `tech_stack`, `research_question`

## Technology Stack

### Backend
- Rust
- Axum (HTTP framework)
- SQLx (SQLite database)
- Tokio (async runtime)
- WebSockets for streaming

### Frontend
- Preact with TypeScript
- TailwindCSS for styling
- Mantine UI components
- Preact Signals for state management

## Future Enhancements

- LLM integration for actual code generation
- MCP server connections for external knowledge
- Container-based sandbox execution
- Multi-agent collaboration
- Learning from user feedback
- Custom sub-agent definitions
- Plugin system for tools

## License

MIT
