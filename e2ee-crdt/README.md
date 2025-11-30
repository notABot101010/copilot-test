# End-to-End Encrypted CRDT Application

A prototype demonstrating end-to-end encrypted collaborative document editing using Automerge CRDT and WebCrypto API.

## Features

- **End-to-End Encryption**: All documents are encrypted with their own AES-256-GCM keys
- **CRDT-based Collaboration**: Real-time editing using Automerge for conflict-free synchronization
- **Hardcoded Identity**: Uses a hardcoded Ed25519 identity key (prototype only)
- **Document Management**: Create, list, and edit encrypted documents
- **Real-time Sync**: WebSocket-based broadcasting of encrypted operations

## Architecture

### Server (Rust)
- **Framework**: Axum with WebSocket support
- **Database**: SQLite for storing encrypted documents and operations
- **Endpoints**:
  - `GET /api/documents` - List all documents
  - `POST /api/documents` - Create a new document
  - `GET /api/documents/:id` - Get a specific document
  - `PUT /api/documents/:id` - Update a document
  - `GET /api/documents/:id/operations` - Get all operations for a document
  - `GET /ws` - WebSocket endpoint for real-time collaboration

### Client (Preact + TypeScript)
- **Framework**: Preact with Vite
- **CRDT**: Automerge for conflict-free editing
- **Encryption**: WebCrypto API (AES-256-GCM)
- **UI**: Mantine components with Tailwind CSS
- **State**: Preact Signals

## Security Model

1. **Document Encryption**: Each document has its own AES-256-GCM encryption key
2. **Encrypted Storage**: Documents are stored encrypted on the server with format: `encryptedData:iv:key`
3. **Encrypted Operations**: CRDT operations are encrypted before being sent over WebSocket
4. **Identity**: Hardcoded Ed25519 identity key shared by all users (prototype only)

**Note**: This is a prototype. In production, you should:
- Generate unique identity keys per user
- Implement proper key exchange mechanisms
- Store keys securely (not in the document data)
- Add authentication and authorization
- Use proper key management systems

## How Real-time Collaboration Works

### Document Creation
1. **User1** creates a document → saved to DB with encrypted data
2. **Server** broadcasts `document_created` message via WebSocket to all connected clients
3. **User2** (and all other clients) receive the message, decrypt the document data, and add it to their document list
4. Both users can now see the same document instantly

### Document Editing
1. **User1** types in the editor → local Automerge document updated
2. **Client** generates CRDT changes and encrypts them
3. **Client** sends encrypted changes to server via REST API
4. **Server** saves encrypted changes and broadcasts to all WebSocket clients
5. **All clients** (including User2) receive the broadcast, decrypt the changes, and apply them to their local Automerge document
6. **UI automatically updates** to reflect the merged changes (using Preact signals)

The app supports full real-time collaboration with:
- **Automatic conflict resolution** via Automerge CRDT
- **End-to-end encryption** of all document data and operations
- **Instant synchronization** across all connected clients

## Getting Started

### Prerequisites

- Rust (latest stable)
- Node.js (v20+)
- npm

### Running the Server

```bash
cd server
cargo run
```

Server will start on `http://localhost:3001`

### Running the Client

```bash
cd webapp
npm install
npm run dev
```

Client will start on `http://localhost:4000`

## Usage

1. Open `http://localhost:4000` in your browser
2. Create a new document by entering a title and clicking "Create"
3. Click on a document to open the editor
4. Edit the title or content - changes are automatically encrypted and synced
5. Open the same document in another browser tab/window to see real-time collaboration

### Testing Real-time Collaboration

1. **Reset the database** (if testing after code changes):
   ```bash
   cd e2ee-crdt
   ./reset.sh
   ```

2. **Start the server** (watch for logs):
   ```bash
   cd server
   cargo run
   ```

3. **Start the client**:
   ```bash
   cd webapp
   npm run dev
   ```

4. **Open two browser windows** side by side to `http://localhost:4000`

5. **Test document creation**:
   - Window 1: Create a new document
   - Window 2: Should see it appear immediately
   - Server logs: `Broadcasted operation for document <id> to N receivers`

6. **Test real-time editing**:
   - Both windows: Open the same document
   - Window 1: Type some text
   - Window 2: Should see changes appear in real-time
   - Check logs (see Debugging section below)

### Debugging Real-time Sync

**Browser Console** (press F12):
- `[WebSocket] Received operation for document: <id>` - Operation received
- `[WebSocket] Applied remote changes` - Changes merged into local doc
- `[Sync] Syncing document: <id>` - Local changes being sent

**Server Terminal**:
- `New WebSocket connection established` - Client connected
- `Updating document: <id>` - REST API received update
- `Broadcasted operation for document <id> to N receivers` - Message sent to N clients
- `Sending WebSocket message: Operation(<id>)` - Individual message sent

**Common Issues**:
- **0 receivers**: No WebSocket clients connected - refresh both browsers
- **Operation not received**: Document not loaded - make sure both windows have the document open
- **Changes only go one way**: Check both browser consoles - one might have an error

## Project Structure

```
e2ee-crdt/
├── server/           # Rust backend
│   ├── src/
│   │   └── main.rs  # Server implementation
│   └── Cargo.toml
└── webapp/          # Preact frontend
    ├── src/
    │   ├── components/
    │   │   ├── DocumentList.tsx
    │   │   └── DocumentEditor.tsx
    │   ├── utils/
    │   │   ├── api.ts          # API client
    │   │   ├── automerge.ts    # Automerge utilities
    │   │   ├── crypto.ts       # Encryption utilities
    │   │   └── websocket.ts    # WebSocket client
    │   ├── app.tsx
    │   ├── store.ts            # Preact Signals store
    │   └── main.tsx
    └── package.json
```

## Technologies

### Server
- Axum - Web framework
- SQLx - Database access
- Tokio - Async runtime
- Serde - Serialization

### Client
- Preact - UI framework
- Automerge - CRDT library
- WebCrypto - Encryption
- Mantine - UI components
- Tailwind CSS - Styling
- Preact Signals - State management

## Troubleshooting

### Decryption Errors

If you encounter decryption errors like "The operation failed for an operation-specific reason":

**Quick Fix:**
```bash
./reset.sh
```

Then clear your browser storage and restart both server and client.

**Manual Steps:**
1. **Clear the database**: Delete `server/crdt.db` and restart the server
2. **Clear browser storage**: Open browser DevTools → Application → Clear all storage
3. **Restart both server and client**

This usually happens when the encryption format changes during development.

### WebSocket Connection Issues

If documents aren't syncing in real-time:

1. Check the browser console for WebSocket connection errors
2. Verify the server is running on port 3001
3. Make sure no firewall is blocking WebSocket connections
4. Check that the Vite proxy is configured correctly

## Limitations & Future Improvements

- Hardcoded identity key (should be per-user)
- No authentication/authorization
- Keys stored with document data (should be separate key management)
- No key rotation
- No access control
- Basic UI (could add rich text editing, presence indicators, etc.)
- No offline support
- No conflict resolution UI

## License

Prototype for demonstration purposes.
