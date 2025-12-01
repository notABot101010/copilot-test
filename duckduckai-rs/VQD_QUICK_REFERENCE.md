# VQD Token Quick Reference

## Token Generation Flow

```
Server Challenge (base64)
         ↓
    Decode base64
         ↓
Execute JavaScript in iframe ← [Browser fingerprinting]
         ↓
Get client_hashes array
         ↓
For each client_hash:
  - UTF-8 encode
  - SHA-256 hash
  - base64 encode
         ↓
Add metadata:
  - origin: window.location.origin
  - stack: Error stack trace (5 frames)
  - duration: computation time (ms)
         ↓
Combine with server data
         ↓
JSON.stringify()
         ↓
base64 encode
         ↓
X-Vqd-Hash-1 header value
```

## Key Code Snippets

### Header Constant
```javascript
const o = "X-Vqd-Hash-1";
```

### Hash Function
```javascript
async function hashClientHash(clientHash) {
  const utf8 = new TextEncoder().encode(clientHash);
  const sha256 = await crypto.subtle.digest("SHA-256", utf8);
  const bytes = new Uint8Array(sha256);
  return btoa(bytes.reduce((str, byte) => str + String.fromCharCode(byte), ""));
}
```

### Stack Trace Extraction
```javascript
function getStackTrace(error, maxFrames = 5) {
  if (!error || !error.stack) return "no-stack";
  const message = error.message || "Unknown";
  const lines = error.stack.split("\n");
  const startIdx = lines[0].includes(message) ? 1 : 0;
  const frames = lines.slice(startIdx, startIdx + maxFrames)
                     .map(l => l.trim())
                     .join("\n");
  const remaining = lines.length - startIdx - maxFrames;
  return frames + (remaining > 0 ? `\n... (${remaining} more frames omitted)` : "");
}
```

### Origin Getter
```javascript
function getOrigin() {
  return (window.top || window).location.origin;
}
```

## Why Our Rust Client Works

### What We Do
1. Use hardcoded `INITIAL_VQD` constant (valid token from browser)
2. Extract updated tokens from `x-vqd-hash-1` response headers
3. Fetch fresh tokens from `/status` endpoint on 418/429 errors
4. Retry up to 3 times with token refresh

### Why It Works
- Tokens have reasonable validity period
- Server accepts slightly stale tokens
- `/status` endpoint provides fresh tokens
- No strict per-request validation

### Trade-offs
✅ Simple implementation
✅ No JavaScript engine needed
✅ Reliable in practice
❌ Requires manual header updates every few weeks/months
❌ Dependent on server behavior not changing

## Token Expiry Indicators

Watch for these HTTP status codes:
- `418 I'm a teapot` - VQD token expired or invalid
- `429 Too Many Requests` - Rate limited or bad token

## Header Relationship

```
Initial Request:
  x-vqd-hash-1: <INITIAL_VQD>
  x-fe-signals: <signals data>
  x-fe-version: <version string>

Response:
  x-vqd-hash-1: <UPDATED_VQD>  ← Extract and use for next request

Next Request:
  x-vqd-hash-1: <UPDATED_VQD>
  x-fe-signals: <same signals>
  x-fe-version: <same version>
```

## Update Checklist

When headers need refreshing:

- [ ] Open https://duckduckgo.com/aichat in browser
- [ ] Open DevTools → Network tab
- [ ] Send a message in AI Chat
- [ ] Find POST `/duckchat/v1/chat`
- [ ] Copy `x-vqd-hash-1` header → update `INITIAL_VQD` in src/lib.rs
- [ ] Copy `x-fe-signals` header → update `X_FE_SIGNALS` in src/lib.rs
- [ ] Copy `x-fe-version` header → update `X_FE_VERSION` in src/lib.rs
- [ ] Run `cargo check` to verify syntax
- [ ] Run `cargo test` to verify functionality
- [ ] Commit changes with descriptive message

## Token Anatomy

```json
{
  "server_hashes": [
    "aG93ZHkgcGFydG5lcg==",
    "aGVsbG8gdGhlcmU=",
    "Z2VuZXJhbCBrZW5vYmk="
  ],
  "client_hashes": [
    "SHA256-hashed-browser-fingerprint-1",
    "SHA256-hashed-browser-fingerprint-2",
    "SHA256-hashed-browser-fingerprint-3"
  ],
  "signals": {},
  "meta": {
    "v": "4",                              // Version
    "challenge_id": "abc123...",           // Challenge identifier
    "timestamp": "1764562505431",          // Unix timestamp (ms)
    "debug": "...",                        // Debug info
    "origin": "https://duckduckgo.com",    // Request origin
    "stack": "Error\nat l (...)",          // Stack trace
    "duration": "14"                       // Computation time (ms)
  }
}
```

After base64 encoding, this becomes the `x-vqd-hash-1` header value.

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Header constants | ✅ Implemented | Hardcoded in `src/lib.rs` |
| Token extraction | ✅ Implemented | From response headers |
| Token refresh | ✅ Implemented | Via `/status` endpoint |
| Retry logic | ✅ Implemented | Up to 3 attempts |
| Dynamic generation | ❌ Not implemented | Requires JS engine |
| Browser fingerprinting | ❌ Not needed | Using server-provided tokens |

---

**Last Updated**: 2025-12-01
