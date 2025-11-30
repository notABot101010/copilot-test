# Offline-First Office Suite

An offline-first office suite built with Automerge CRDT for real-time collaboration. Features a markdown document editor powered by TipTap and a PowerPoint-like presentation editor.

## Features

- **Document Editor**: Real-time markdown editor with rich text formatting
- **Presentation Editor**: Slide-based presentation tool similar to PowerPoint
- **Offline-First**: Works offline with automatic synchronization when online
- **Real-time Collaboration**: Multiple users can edit the same document simultaneously
- **CRDT-Based**: Uses Automerge CRDT for conflict-free synchronization

## Tech Stack

### Server (Rust)
- **Axum**: HTTP server and WebSocket support
- **SQLx**: SQLite database for document persistence
- **Automerge**: CRDT library for conflict-free document synchronization
- **Tokio**: Async runtime

### Client (Preact)
- **Preact**: Lightweight React alternative
- **Vite**: Fast build tool
- **TipTap**: Rich text editor for documents
- **Automerge**: CRDT synchronization
- **Mantine**: UI component library
- **Tailwind CSS**: Utility-first CSS framework
- **Preact Signals**: State management

## Getting Started

### Prerequisites
- Rust (latest stable)
- Node.js (v18+)
- npm

### Running the Server

```bash
cd server
cargo run
```

The server will start on `http://localhost:8080`

### Running the Client

```bash
cd webapp
npm install
npm run dev
```

The webapp will start on `http://localhost:4000`

## Architecture

### Server
The Rust server handles:
- Document storage in SQLite database
- WebSocket connections for real-time sync
- Automerge CRDT change management
- Broadcasting changes to all connected clients

### Client
The Preact webapp provides:
- Document and presentation editors
- Real-time synchronization via WebSocket
- Offline-first architecture with local Automerge documents
- Automatic reconnection and sync

### CRDT Synchronization
Both server and client maintain Automerge documents. When a user makes changes:
1. Client applies changes locally to Automerge document
2. Changes are sent via WebSocket to server
3. Server merges changes and persists to SQLite
4. Server broadcasts changes to all other connected clients
5. Other clients apply changes to their local Automerge documents

This ensures conflict-free merging even when multiple users edit simultaneously.

## API Endpoints

- `GET /api/documents` - List all documents
- `POST /api/documents` - Create a new document
- `GET /api/documents/:id` - Get document data
- `DELETE /api/documents/:id` - Delete a document
- `POST /api/documents/:id/sync` - Sync document changes
- `WS /api/documents/:id/ws` - WebSocket for real-time sync

## Development

### Server Development
```bash
cd server
cargo check  # Check for errors
cargo run    # Run in debug mode
```

### Client Development
```bash
cd webapp
npm run dev   # Development server with HMR
npm run build # Production build
```

## License

MIT
