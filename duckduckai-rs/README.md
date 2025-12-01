# DuckDuckGo AI Rust Client

A Rust client library and CLI for interacting with the DuckDuckGo AI chat API, featuring a built-in JavaScript runtime for solving VQD challenges.

## Features

- Simple Rust client library for DuckDuckGo AI
- Streaming and non-streaming chat support
- CLI interface for quick testing
- **Automatic VQD challenge solving using QuickJS JavaScript runtime**
- Error handling and token refresh logic

## How It Works

The DuckDuckGo AI API uses a challenge-response mechanism for anti-bot protection:

1. The client fetches the `/status` endpoint which returns a challenge in the `x-vqd-hash-1` header
2. The challenge is a base64-encoded JSON containing server hashes
3. The client computes `client_hashes` using SHA-256 and browser fingerprint simulation
4. The solved challenge is used as the `X-Vqd-Hash-1` header for chat requests

This implementation uses [rquickjs](https://github.com/DelSkayn/rquickjs) (QuickJS bindings) to execute JavaScript-like computations and simulate browser APIs.

## Installation

```bash
cargo build --release
```

## Usage

### CLI

**Single message:**
```bash
cargo run -- --message "What is Rust?"
```

**With streaming:**
```bash
cargo run -- --message "Tell me about AI" --stream
```

**Interactive mode:**
```bash
cargo run
```

**With debug logging:**
```bash
cargo run -- --message "Hello" --debug
```

### Library Usage

```rust
use duckduckai::DuckDuckGoClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = DuckDuckGoClient::new()?;

    // Non-streaming - automatically handles challenge solving
    let response = client.chat("What is 2+2?", None).await?;
    println!("{}", response);

    // Streaming
    client.chat_stream("Count to 5", None, |chunk| {
        print!("{}", chunk);
    }).await?;

    Ok(())
}
```

### Using the Challenge Solver Directly

```rust
use duckduckai::ChallengeSolver;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let solver = ChallengeSolver::new()?;
    
    // Solve a challenge from the status endpoint
    let challenge_b64 = "..."; // base64-encoded challenge from x-vqd-hash-1 header
    let solved = solver.solve(challenge_b64).await?;
    
    // Use `solved` as the X-Vqd-Hash-1 header value
    println!("Solved: {}", solved);
    
    Ok(())
}
```

## Architecture

### JavaScript Runtime (js_runtime module)

The `ChallengeSolver` provides a sandboxed QuickJS environment that:

1. **Parses the challenge**: Decodes the base64 challenge from the status endpoint
2. **Simulates browser APIs**: Provides `window`, `document`, `navigator`, `screen`, `atob`, `btoa`, `Date.now()`
3. **Computes client hashes**: Generates fingerprint-like strings and hashes them with SHA-256
4. **Produces the VQD token**: Packages the result with metadata and encodes as base64

### VQD Token Management

The `x-vqd-hash-1` header serves as the VQD (Verification Query Data) token. The client automatically:

- Fetches challenges from the `/status` endpoint
- Solves challenges using the JavaScript runtime
- Uses the solved VQD token for chat requests
- Refreshes VQD and retries (up to 3 attempts) on 418/429 errors

## API Reference

### `DuckDuckGoClient`

#### Methods

- `new() -> Result<Self>` - Create a new client instance
- `chat(&mut self, message: &str, model: Option<&str>) -> Result<String>` - Send a chat message and get the full response
- `chat_stream<F>(&mut self, message: &str, model: Option<&str>, callback: F) -> Result<()>` - Stream chat responses with a callback function

### `ChallengeSolver`

#### Methods

- `new() -> Result<Self>` - Create a new challenge solver with a QuickJS runtime
- `solve(&self, challenge_b64: &str) -> Result<String>` - Solve a base64-encoded challenge and return the VQD header value

#### Available Models

- `gpt-4o-mini` (default)
- `claude-3-haiku-20240307`
- `meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo`
- `mistralai/Mixtral-8x7B-Instruct-v0.1`

## Testing

```bash
# Run all non-ignored tests (unit tests + challenge solver tests)
cargo test

# Run ignored integration tests (requires network access to DuckDuckGo API)
cargo test -- --ignored

# Run specific test with output
cargo test test_challenge_solver_with_sample_data -- --nocapture
```

**Note:** Integration tests marked with `#[ignore]` require network access to the real DuckDuckGo API.

## VQD Token Reverse Engineering

For detailed information about how the `x-vqd-hash-1` header works, see:

- **[VQD_REVERSE_ENGINEERING.md](VQD_REVERSE_ENGINEERING.md)** - Complete technical analysis of the VQD token generation mechanism
- **[VQD_QUICK_REFERENCE.md](VQD_QUICK_REFERENCE.md)** - Quick reference guide
- **[VQD_CLARIFICATION.md](VQD_CLARIFICATION.md)** - Header naming clarification

### Key Findings

The VQD token is generated through a sophisticated anti-bot mechanism:

1. Server sends a base64-encoded challenge (not JavaScript code, but JSON data)
2. Client computes browser fingerprints to generate `client_hashes`
3. Each hash is SHA-256 encoded and base64-encoded
4. Result is packaged with metadata (origin, stack trace, duration)
5. Final JSON is base64-encoded to create the `x-vqd-hash-1` header value

## References

- [DuckDuckGo Chat CLI](https://github.com/benoitpetit/duckduckgo-chat-cli) - Original Go implementation
- [rquickjs](https://github.com/DelSkayn/rquickjs) - QuickJS JavaScript engine bindings for Rust
- [status.sh](status.sh) - Bash script to test the `/status` endpoint

## License

This is a demonstration project for educational purposes. Please respect DuckDuckGo's terms of service.
