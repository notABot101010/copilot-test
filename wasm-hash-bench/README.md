# WASM Hash Benchmark

A benchmark comparing BLAKE3, SHA-256, and SHA-512 hash functions compiled to WebAssembly.

## Features

- Benchmarks three hash algorithms:
  - **BLAKE3**: Modern, fast cryptographic hash function
  - **SHA-256**: 256-bit SHA-2 hash function
  - **SHA-512**: 512-bit SHA-2 hash function
- Runs in both **Node.js** and **browser** environments
- Tests multiple data sizes (64 bytes to 10 MiB)
- Measures throughput (MB/s), operations per second, and average time

## Quick Start

### Prerequisites

- Rust toolchain (for building WASM)
- Node.js (for running benchmarks)

### Build and Run

```bash
# Build the WASM module
make build

# Run benchmark in Node.js
make bench

# Serve benchmark for browser testing
make serve
```

Then open http://localhost:3000/ in your browser.

## Makefile Targets

- `make build` - Compile Rust code to WebAssembly
- `make bench` - Run benchmark in Node.js
- `make serve` - Start HTTP server for browser testing
- `make test` - Run Rust unit tests
- `make clean` - Remove build artifacts

## Benchmark Results

The benchmark tests hash performance across different data sizes:
- Small data (64-256 bytes)
- Medium data (1-16 KiB)
- Large data (64 KiB - 10 MiB)

Results show:
- **BLAKE3** typically performs best on medium to large data sizes
- **SHA-512** can be competitive on small data sizes
- **SHA-256** generally falls in between

Performance varies by environment (Node.js vs browser) and hardware.

## Project Structure

```
wasm-hash-bench/
├── src/
│   └── lib.rs          # Rust WASM bindings for hash functions
├── Cargo.toml          # Rust dependencies
├── Makefile            # Build automation
├── benchmark.js        # Benchmark implementation (Node.js & browser)
├── index.html          # Browser UI
├── server.js           # Simple HTTP server
└── package.json        # Node.js configuration
```

## Implementation Details

- Uses `blake3` crate (v1.8.2) for BLAKE3 hashing
- Uses `sha2` crate (v0.10) for SHA-256 and SHA-512
- Built with `wasm-pack` for optimal WASM output
- Compatible with ES modules in both Node.js and browsers
