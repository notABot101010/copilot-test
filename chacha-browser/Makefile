.PHONY: build dev clean install

# Build WASM library using wasm-pack and output to pkg/
build:
	wasm-pack build --target web --out-dir pkg

# Install webapp dependencies and start Vite dev server
dev: build
	cd webapp && npm install && npm run dev

# Install webapp dependencies only
install:
	cd webapp && npm install

# Clean build artifacts
clean:
	rm -rf pkg target webapp/node_modules
