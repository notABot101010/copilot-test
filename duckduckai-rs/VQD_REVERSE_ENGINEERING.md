# X-Vqd-Hash-1 Header - Reverse Engineering Analysis

## Overview

This document describes the reverse-engineered VQD (Verification Query Data) token generation mechanism used by DuckDuckGo AI Chat API, based on analysis of `wpm.main.js` from `https://duckduckgo.com/dist/wpm.main.758b58e5295173a9d89c.js`.

## Header Information

- **Header Name**: `X-Vqd-Hash-1` (constant defined in module 49523)
- **Purpose**: Anti-bot protection via client-side cryptographic challenge-response
- **Format**: Base64-encoded JSON object

## Token Structure

The VQD token is a base64-encoded JSON with the following structure:

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

## Token Generation Process

### Step 1: Server Challenge

The server sends a base64-encoded challenge string (typically via the `x-vqd-hash-1` response header from the `/status` endpoint).

### Step 2: Challenge Decoding

```javascript
const r = atob(e) // Decode base64 challenge
```

The decoded challenge contains JavaScript code that will compute the client hashes.

### Step 3: Sandboxed JavaScript Execution

The challenge JavaScript is executed in an isolated iframe:

```javascript
const s = o.createElement("script");
s.textContent = `
  try {
    window.parent.__jsaCallbacks[${n}](null, ${e});
  } catch (e) {
    window.parent.__jsaCallbacks[${n}](e, null);
  }
`;
o.body.appendChild(s);
```

This dynamically executed code:
- Runs in a sandboxed iframe environment
- Has access to browser APIs and fingerprinting data
- Computes `client_hashes` based on browser characteristics
- Returns an object with `client_hashes`, `server_hashes`, `signals`, and `meta`

### Step 4: Hash Computation

Each `client_hash` string returned from the iframe is processed:

```javascript
async function hashClientHash(e) {
  const t = (new TextEncoder).encode(e);           // UTF-8 encode
  const n = await crypto.subtle.digest("SHA-256", t); // SHA-256 hash
  const r = new Uint8Array(n);
  return btoa(r.reduce(((e,t) => e + String.fromCharCode(t)), "")); // Base64 encode
}
```

**Process:**
1. Encode each client_hash string as UTF-8
2. Compute SHA-256 hash using Web Crypto API
3. Convert hash bytes to base64 string

### Step 5: Metadata Addition

Additional metadata is appended:

```javascript
{
  origin: u(),                  // window.location.origin
  stack: d(new Error),          // Formatted error stack trace
  duration: String(Date.now() - n) // Time taken in milliseconds
}
```

**Helper Functions:**

```javascript
function u() {
  return (window.top || window).location.origin;
}

function d(e, t = 5) {
  if (!e || !e.stack) return "no-stack";
  const n = e.message || "Unknown";
  const r = e.stack.split("\n");
  const a = r[0].includes(n) ? 1 : 0;
  return `${r.slice(a, a + t).map(e => e.trim()).join("\n")}${
    r.length > a + t ? `\n... (${r.length - a - t} more frames omitted)` : ""
  }`;
}
```

### Step 6: Final Encoding

The complete object is base64-encoded:

```javascript
btoa(JSON.stringify({
  ...a,  // Original challenge response
  client_hashes: await Promise.all(a.client_hashes.map(hashClientHash)),
  meta: {
    ...a.meta || {},
    origin: u(),
    stack: d(new Error),
    duration: String(Date.now() - n)
  }
}))
```

## Error Handling

If token generation fails, a fallback mechanism appends error information:

```javascript
function c(e, t) {
  try {
    const n = atob(e);
    return t instanceof Error
      ? btoa(`${n}::${t.message}::${d(t)}::${u()}`)
      : btoa(`${n}::${String(t)}::${d(new Error)}::${u()}`);
  } catch(t) {
    return `${e}::unknown::${u()}`;
  }
}
```

## Anti-Reverse-Engineering Measures

1. **Dynamic Code Execution**: The actual client_hash computation happens in dynamically executed JavaScript code sent from the server
2. **Sandbox Isolation**: Code runs in an iframe, limiting inspection
3. **Obfuscation**: The wpm.main.js file is heavily minified
4. **Challenge-Response**: The server controls what gets computed by sending different challenge code
5. **Browser Fingerprinting**: The dynamically executed code likely collects browser characteristics that are difficult to replicate

## Why Simple Token Reuse Works

Our Rust client successfully uses hardcoded tokens because:

1. **Token Lifetime**: VQD tokens have a reasonable validity period
2. **Server Tolerance**: The API accepts slightly stale tokens
3. **Refresh Mechanism**: We fetch fresh tokens from `/status` endpoint when needed
4. **No Strict Validation**: The server doesn't strictly validate all token components for every request

## Limitations of Full Replication

Fully replicating this in Rust would require:

1. **JavaScript Engine**: Executing arbitrary JavaScript (challenge code)
2. **Browser Environment**: Simulating browser APIs and fingerprinting surface
3. **Dynamic Adaptation**: Handling changing challenge code from server
4. **Maintenance Burden**: Keeping up with server-side changes

## Current Implementation Strategy

Our Rust client uses a pragmatic approach:

1. **Hardcoded Initial Token**: Start with a working base64-encoded VQD token
2. **Token Extraction**: Parse `x-vqd-hash-1` from response headers
3. **Automatic Refresh**: Fetch new tokens from `/status` when expired (418/429 errors)
4. **Retry Logic**: Attempt up to 3 times with fresh tokens

This approach is:
- ✅ Simple and maintainable
- ✅ Works reliably in practice
- ✅ No JavaScript engine required
- ❌ Requires periodic header updates
- ❌ Dependent on DuckDuckGo's server behavior

## Token Update Procedure

When tokens expire:

1. Open browser DevTools at https://duckduckgo.com/aichat
2. Send a message in AI Chat
3. Find POST to `/duckchat/v1/chat`
4. Copy these headers:
   - `x-vqd-hash-1` → `INITIAL_VQD`
   - `x-fe-signals` → `X_FE_SIGNALS`
   - `x-fe-version` → `X_FE_VERSION`
5. Update constants in `src/lib.rs`
6. Rebuild with `cargo build`

## Module Information

**Source File**: `wpm.main.js` (webpack module 49523)

**Exports**:
- `TY` → `"X-Vqd-Hash-1"` (header name constant)
- `wR` → Main token generation function
- `P5` → Token encoding helper
- `nU` → (additional export, purpose unknown)

## Conclusion

The VQD token system is a sophisticated anti-bot mechanism using:
- Server-controlled challenge-response
- Client-side cryptographic operations (SHA-256)
- Browser fingerprinting via sandboxed JavaScript
- Multiple layers of obfuscation

For practical API clients, the token reuse strategy is the most viable approach given the complexity of full replication.

---

**Analysis Date**: 2025-12-01
**Source**: `wpm.main.js` (hash: 758b58e5295173a9d89c)
**Analyzed Module**: 49523 (VQD generation)
