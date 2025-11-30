# End-to-End Encrypted Chat Application

An MVP implementation of a secure, end-to-end encrypted chat application with advanced cryptographic features.

## Features

### Security & Encryption
- **Master Key Derivation**: Client-side PBKDF2 key derivation from user passwords
- **Identity Keys**: Ed25519 long-term identity keys, encrypted with master key
- **Key Exchange**: X25519-based Diffie-Hellman key exchange for each message
- **Message Encryption**: AES-256-GCM encryption for all messages
- **Double Ratchet Protocol**: Provides forward secrecy and post-compromise security
- **Sealed Sender**: Messages can be sent anonymously without revealing sender identity to server
- **Signature Verification**: All ephemeral keys are signed with Ed25519 identity keys

### Features
- User registration and login
- Real-time messaging with long-polling
- Responsive UI for desktop and mobile
- Conversation history
- Multiple simultaneous conversations

## Architecture

### Backend (Rust + Axum)
- REST API for user authentication and message forwarding
- SQLite database for users and encrypted messages
- Long-polling for real-time message delivery
- Server never has access to message plaintext

### Frontend (Preact + Vite)
- WebCrypto API for cryptographic operations
- @noble/curves for Ed25519 and X25519 operations
- Mantine UI components
- Tailwind CSS for styling
- Preact Router for navigation
- Preact Signals for state management

## Prerequisites

- Node.js (v18 or higher)
- Rust (latest stable)
- npm or yarn

## Installation

### Backend Setup

```bash
cd e2ee-chat/backend

# Build the backend (debug mode)
cargo build

# Run the server
cargo run
```

The backend server will start on `http://localhost:3000`.

### Frontend Setup

```bash
cd e2ee-chat/frontend

# Install dependencies
npm install

# Start development server
npm run dev
```

The frontend will be available at `http://localhost:4000`.

## Usage

1. **Register an Account**
   - Navigate to http://localhost:4000/register
   - Choose a username and password (min 8 characters)
   - Your identity keys will be generated and encrypted locally

2. **Login**
   - Enter your credentials
   - Your identity keys will be decrypted locally

3. **Start a Conversation**
   - Enter a username in the "New conversation" field
   - Send encrypted messages
   - Messages are end-to-end encrypted using the double ratchet protocol

## Security Details

### Key Hierarchy

```
Password (user input)
    ↓ PBKDF2 (100,000 iterations)
Master Key (256-bit)
    ↓ AES-256-GCM
Encrypted Identity Key (Ed25519)
    ↓ Signs
Ephemeral DH Keys (X25519)
    ↓ Key Exchange
Message Keys (AES-256-GCM)
```

### Double Ratchet Protocol

The application implements a simplified version of the Signal Protocol's double ratchet:

1. **Symmetric Ratchet**: Advances chain keys with each message
2. **DH Ratchet**: Generates new ephemeral keys periodically
3. **Message Keys**: Derived from chain keys, never reused
4. **Out-of-Order Messages**: Supports message skipping and delayed delivery

### Sealed Sender

Messages can be sent with or without sender identity:
- **Authenticated Mode**: Sender identity and signature included (encrypted)
- **Anonymous Mode**: Only ephemeral public key visible to server

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login with credentials

### Messages
- `POST /api/messages/send` - Send encrypted message
- `GET /api/messages/poll?username=<user>` - Long-poll for new messages
- `GET /api/messages/<username>?current_user=<user>` - Get conversation history

### User Keys
- `GET /api/users/<username>/keys` - Get user's public keys

### Ratchet State (Optional)
- `POST /api/ratchet` - Save ratchet state to server
- `GET /api/ratchet/<peer>?username=<user>` - Load ratchet state

## Development

### Building for Production

Backend:
```bash
cd backend
cargo build --release
./target/release/e2ee-chat-server
```

Frontend:
```bash
cd frontend
npm run build
npm run preview
```

### Environment Variables

Backend:
- `DATABASE_URL` - SQLite database path (default: `sqlite:chat.db`)

Frontend:
- `API_BASE_URL` - Backend API URL (default: `http://localhost:3000`)

## Testing

### Backend Tests
```bash
cd backend
cargo test
```

### Frontend Tests
```bash
cd frontend
npm test
```

### E2E Tests
```bash
cd frontend
npm run test:e2e
```

## Security Considerations

### What's Secure
- Messages are end-to-end encrypted
- Server cannot read message content
- Forward secrecy protects past messages
- Post-compromise security protects future messages
- Sealed sender hides sender from server

### MVP Limitations
- No formal security audit
- Simplified key exchange (production should use prekeys)
- Local storage used for keys (should use secure enclave in production)
- No key verification/trust establishment UI
- No group chat support
- Simplified ratchet implementation

### Production Recommendations
- Use hardware security modules for key storage
- Implement safety numbers for key verification
- Add support for multiple devices per user
- Implement key rotation policies
- Add comprehensive audit logging
- Perform professional security audit
- Implement rate limiting and abuse prevention

## License

MIT

## Contributing

This is an MVP demonstration project. For production use, please conduct a thorough security review and implement additional security measures.
