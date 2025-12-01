# DuckDuckGo AI Rust Client

A Rust client library and CLI for interacting with the DuckDuckGo AI chat API, based on reverse engineering documentation from [duckduckgo-chat-cli](https://github.com/benoitpetit/duckduckgo-chat-cli).

## Features

- Simple Rust client library for DuckDuckGo AI
- Streaming and non-streaming chat support
- CLI interface for quick testing
- Automatic VQD token management
- Error handling and token refresh logic

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

    // Non-streaming
    let response = client.chat("What is 2+2?", None).await?;
    println!("{}", response);

    // Streaming
    client.chat_stream("Count to 5", None, |chunk| {
        print!("{}", chunk);
    }).await?;

    Ok(())
}
```

## Important Limitations

### Anti-Bot Headers

The DuckDuckGo AI API implements strong anti-bot protection that requires specific headers:

- `x-vqd-hash-1`: Base64-encoded JSON with cryptographic hashes (serves as the VQD token)
- `x-fe-signals`: Frontend telemetry signals
- `x-fe-version`: Client version identifier

**These headers are derived from real browser traffic and expire frequently.**

**Important:** Some documentation incorrectly refers to `x-vqd-4` - this header does not exist. The correct header for VQD tokens is `x-vqd-hash-1`.

The current implementation includes hardcoded header values that may not work if DuckDuckGo has updated their anti-bot measures. To obtain fresh headers:

1. Open https://duckduckgo.com/aichat in Chrome with Developer Tools (F12)
2. Go to Network tab and send a message in the chat
3. Find the POST request to `/duckchat/v1/chat`
4. In Request Headers, copy the values of:
   - `x-vqd-hash-1` (this is the VQD token - used for both sending and receiving)
   - `x-fe-signals`
   - `x-fe-version`
5. Update these constants in `src/lib.rs`:

```rust
const INITIAL_VQD: &str = "YOUR_EXTRACTED_x-vqd-hash-1_VALUE";
const X_FE_SIGNALS: &str = "YOUR_EXTRACTED_VALUE";
const X_FE_VERSION: &str = "YOUR_EXTRACTED_VALUE";
```

6. Rebuild the project: `cargo build`

### VQD Token Management

The `x-vqd-hash-1` header serves as the VQD (Verification Query Data) token. The client automatically:
- Uses the initial VQD token from `INITIAL_VQD` constant
- Fetches VQD token from the `/status` endpoint headers (if available)
- Updates VQD token from chat response headers (when provided by the API)
- Refreshes VQD and retries (up to 3 attempts) on 418/429 errors

**Note:** The VQD token is sent as `x-vqd-hash-1` header and may be updated from response headers with the same name.

## API Reference

### `DuckDuckGoClient`

#### Methods

- `new() -> Result<Self>` - Create a new client instance
- `chat(&mut self, message: &str, model: Option<&str>) -> Result<String>` - Send a chat message and get the full response
- `chat_stream<F>(&mut self, message: &str, model: Option<&str>, callback: F) -> Result<()>` - Stream chat responses with a callback function

#### Available Models

- `gpt-5-mini` (default)
- `gpt-4o-mini`
- `claude-3-haiku-20240307`
- `meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo`
- `mistralai/Mixtral-8x7B-Instruct-v0.1`

## Testing

```bash
# Run all non-ignored tests (unit tests + client initialization)
cargo test

# Run ignored integration tests (requires valid API headers)
cargo test -- --ignored

# Run specific test
cargo test test_simple_chat -- --ignored --nocapture
```

**Note:** Integration tests are marked as `#[ignore]` by default because they require fresh anti-bot headers. Update the headers as described above to run them successfully.

## Architecture

The client implements:

1. **Cookie Management**: Maintains required cookies (`5`, `dcm`, `dcs`)
2. **Header Construction**: Builds requests with all required anti-bot headers
3. **VQD Token Lifecycle**:
   - Uses initial hardcoded VQD token (`x-vqd-hash-1`)
   - Optionally fetches from `/status` endpoint response headers
   - Updates from chat response headers (when provided)
   - Automatic retry with VQD refresh on token expiration (418/429 errors)
4. **SSE Parsing**: Manual Server-Sent Events stream parsing for chat responses

## VQD Token Reverse Engineering

For detailed information about how the `x-vqd-hash-1` header works, see:

- **[VQD_REVERSE_ENGINEERING.md](VQD_REVERSE_ENGINEERING.md)** - Complete technical analysis of the VQD token generation mechanism extracted from `wpm.main.js`
- **[VQD_QUICK_REFERENCE.md](VQD_QUICK_REFERENCE.md)** - Quick reference guide with code snippets and update procedures
- **[VQD_CLARIFICATION.md](VQD_CLARIFICATION.md)** - Clarification about the `x-vqd-hash-1` vs `x-vqd-4` misconception

### Key Findings

The VQD token is generated through a sophisticated anti-bot mechanism:

1. Server sends a base64-encoded JavaScript challenge
2. Client executes the challenge in a sandboxed iframe
3. Challenge code computes browser fingerprints (client_hashes)
4. Each hash is SHA-256 encoded and base64-encoded
5. Result is packaged with metadata (origin, stack trace, duration)
6. Final JSON is base64-encoded to create the `x-vqd-hash-1` header value

**Why our simple approach works**: The DuckDuckGo API accepts VQD tokens with reasonable validity periods, and we can fetch fresh tokens from the `/status` endpoint when needed. Full replication would require a JavaScript engine and browser environment simulation, which is impractical for a Rust client.

## References

- [DuckDuckGo Chat CLI](https://github.com/benoitpetit/duckduckgo-chat-cli) - Original Go implementation
- [Reverse Engineering Documentation](https://github.com/benoitpetit/duckduckgo-chat-cli/blob/master/reverse/README.md) - API details
- [status.sh](status.sh) - Bash script to test the `/status` endpoint and extract VQD tokens

## License

This is a demonstration project for educational purposes. Please respect DuckDuckGo's terms of service.
