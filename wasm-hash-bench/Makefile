.PHONY: build bench serve clean test install-wasm-pack

# Install wasm-pack if not already installed
install-wasm-pack:
	@which wasm-pack > /dev/null || cargo install wasm-pack

# Build the hash benchmark WebAssembly bindings
build: install-wasm-pack
	RUSTFLAGS="-C target-feature=+simd128" wasm-pack build --release --target web --out-dir pkg

# Run the benchmark with Node.js
bench: build
	node benchmark.js

# Start the server to run the benchmark in the browser
serve: build
	node server.js

# Run Rust tests
test:
	cargo test

# Clean build artifacts
clean:
	rm -rf pkg target
