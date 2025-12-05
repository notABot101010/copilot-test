.PHONY: build bench serve clean

# Build the ChaCha WebAssembly bindings with SIMD128 support
build:
	cd ../chacha-browser && RUSTFLAGS="-C target-feature=+simd128" wasm-pack build --target web --out-dir ../webcrypto-bench/pkg

# Run the benchmark with Node.js
bench: build
	node benchmark.js

# Start the server to run the benchmark in the browser
serve: build
	node server.js

# Clean build artifacts
clean:
	rm -rf pkg
