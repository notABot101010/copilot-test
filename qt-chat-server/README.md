# Qt Chat Application

A real-time chat application built with Rust, featuring an HTTP-based server and multiple client implementations.

## Project Structure

- **qt-chat-server**: Rust-based HTTP server using Axum for real-time chat functionality
- **qt-chat**: Qt GUI client (work in progress - uses qmetaobject bindings)
- **qt-chat-demo**: HTML/JavaScript web demo client for testing

## Features

### Server (qt-chat-server)
- Built with Axum web framework
- RESTful HTTP API
- In-memory room and message management
- Real-time message polling
- CORS enabled for web clients

### API Endpoints

- `POST /rooms` - Create a new chat room
- `GET /rooms` - List all available rooms
- `POST /rooms/{id}/join` - Join a specific room
- `POST /rooms/{id}/messages` - Send a message to a room
- `GET /rooms/{id}/messages` - Get all messages in a room

## Running the Application

### Starting the Server

```bash
cd qt-chat-server
cargo run --release
```

The server will start on `http://localhost:3000`

### Using the Web Demo Client

1. Start the server (see above)
2. Open `qt-chat-demo/index.html` in a web browser
3. Enter a username
4. Create or join a room
5. Start chatting!

To test multiple clients, open the page in multiple browser tabs or windows with different usernames.

## End-to-End Tests

The project includes comprehensive end-to-end tests that verify:
- Room creation
- Room joining
- Message sending
- Message receiving between two clients

Run the tests with:

```bash
cd qt-chat-server
cargo test --test e2e
```

## Development

### Building the Server

```bash
cd qt-chat-server
cargo build
```

### Building the Qt Client (Requires Qt 5)

The Qt client requires Qt 5 development libraries:

```bash
sudo apt-get install qtbase5-dev qtdeclarative5-dev qml-module-qtquick2 qml-module-qtquick-controls2
cd qt-chat
cargo build
```

Note: The Qt client is currently a work in progress.

## Architecture

The application uses a simple client-server architecture:

1. **Server**: Manages rooms and messages in memory
2. **Clients**: Poll the server for new messages every 2 seconds
3. **HTTP**: All communication happens over HTTP with JSON payloads

### Communication Flow

1. Client creates a room with a name and creator username
2. Other clients can join the room by providing their username
3. Clients send messages to the room
4. Clients poll for new messages at regular intervals

## Screenshots

### Initial State
![Initial State](https://github.com/user-attachments/assets/c5da795e-efe4-42c9-869f-e76fdb4dbbb8)

### Single User Chatting
![Single User](https://github.com/user-attachments/assets/176b3d8b-9980-4b84-b6fb-2c7270fb4880)

### Two Users Communicating
![Two Users](https://github.com/user-attachments/assets/572d0a4a-b8b0-4ba4-b2c8-a85054e62553)

## Dependencies

### Server
- axum 0.8 - Web framework
- tokio 1.x - Async runtime
- tower-http 0.6 - CORS middleware
- serde & serde_json - JSON serialization
- uuid - Unique identifiers
- chrono - Timestamps

### Qt Client (WIP)
- qmetaobject 0.2 - Qt bindings for Rust
- reqwest 0.12 - HTTP client

## Future Improvements

- Persistent storage (database integration)
- User authentication
- Private messaging
- File sharing
- Better error handling
- WebSocket support for real-time updates (instead of polling)
- Complete Qt native client
