# VQD Token Implementation - Clarification

## ❌ Common Misconception

Some documentation (including the original reverse engineering docs) refers to `x-vqd-4` as the VQD token header. **This is incorrect.**

## ✅ Correct Implementation

The VQD (Verification Query Data) token uses the **`x-vqd-hash-1`** header exclusively.

### How It Works

1. **Initial Token**: Use a hardcoded base64-encoded JSON value in `INITIAL_VQD` constant
2. **Sending Requests**: Include this token in the `x-vqd-hash-1` request header
3. **Receiving Updates**: The API may return updated tokens in the `x-vqd-hash-1` response header
4. **Token Refresh**: When receiving 418/429 errors, fetch a fresh token from `/status` endpoint

### Header Usage

```rust
// In requests (build_headers function):
headers.insert("x-vqd-hash-1", HeaderValue::from_str(vqd)?);

// From responses (fetch_vqd and chat functions):
if let Some(header_value) = response.headers().get("x-vqd-hash-1") {
    // Update the stored VQD token
    self.vqd = Some(header_value.to_str()?.to_string());
}
```

### Token Structure

The `x-vqd-hash-1` value is a base64-encoded JSON containing:
- `server_hashes`: Array of cryptographic hashes from the server
- `client_hashes`: Array of cryptographic hashes from the client
- `signals`: Additional telemetry data (usually empty object)
- `meta`: Metadata including version, challenge_id, timestamp, debug info, origin, stack trace, and duration

Example (decoded):
```json
{
  "server_hashes": ["...base64...", "...base64...", "...base64..."],
  "client_hashes": ["...base64...", "...base64...", "...base64..."],
  "signals": {},
  "meta": {
    "v": "4",
    "challenge_id": "...",
    "timestamp": "1764562505431",
    "debug": "...",
    "origin": "https://duckduckgo.com",
    "stack": "Error\nat l (https://...)...",
    "duration": "14"
  }
}
```

## Implementation in This Client

### Constants (src/lib.rs)
```rust
const INITIAL_VQD: &str = "eyJzZXJ2ZXJfaGFzaGVzIjpbIkdKTVJYUzNNeklVNnoxdnBrcFVB...";
const X_FE_SIGNALS: &str = "eyJzdGFydCI6MTc2NDU2MjUwMzAyMywiZXZlbnRzIjpb...";
const X_FE_VERSION: &str = "serp_20251128_165151_ET-b75a8c1a947c8014070f";
```

### Token Lifecycle

1. **Initialization**: Client starts with `INITIAL_VQD` value
2. **First Request**: Uses `INITIAL_VQD` as `x-vqd-hash-1` header
3. **Response Processing**: Checks for `x-vqd-hash-1` in response headers, updates if present
4. **Error Handling**: On 418/429 errors:
   - Calls `/status` endpoint
   - Extracts `x-vqd-hash-1` from response headers (if available)
   - Falls back to `INITIAL_VQD` if not found
   - Retries the request (up to 3 times)

## Why the Confusion?

The original reverse engineering documentation may have referenced `x-vqd-4` based on:
- Historical API versions
- Misinterpretation of the header name
- Analysis of obfuscated client code

Through testing and inspection of actual browser traffic, we've confirmed that **only `x-vqd-hash-1` is used** in the current DuckDuckGo AI API implementation.

## Testing

To verify this implementation:

```bash
# Run with debug logging to see VQD token flow
cargo run -- --message "Hello" --debug

# You'll see:
# - "Using VQD token: eyJzZXJ2ZXJfaGFzaGVzIjpb..."
# - "Sending chat request with model: gpt-5-mini"
# - (Optionally) "Updating VQD token from response: ..."
```

## Updating Headers

When headers expire, extract fresh values from browser DevTools:

1. Open https://duckduckgo.com/aichat
2. Send a message
3. Find POST to `/duckchat/v1/chat`
4. Copy `x-vqd-hash-1` header value → `INITIAL_VQD`
5. Copy `x-fe-signals` header value → `X_FE_SIGNALS`
6. Copy `x-fe-version` header value → `X_FE_VERSION`
7. Rebuild: `cargo build`

---

**Last Updated**: 2025-12-01
**Verified Against**: DuckDuckGo AI Chat API (https://duckduckgo.com/aichat)
