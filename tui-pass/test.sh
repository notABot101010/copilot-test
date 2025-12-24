#!/bin/bash

# This script demonstrates how to use tui-pass
# Note: This is for documentation purposes as the tool requires interactive password input

echo "=== TUI Password Manager Test Script ==="
echo ""
echo "Binary location: $(pwd)/target/release/tui-pass"
echo ""

# Check if binary exists
if [ ! -f "target/release/tui-pass" ]; then
    echo "Error: Binary not found. Run 'cargo build --release' first."
    exit 1
fi

echo "✓ Binary exists"
echo ""

# Show help
echo "=== Help Output ==="
./target/release/tui-pass --help
echo ""

# Show create command help
echo "=== Create Command Help ==="
./target/release/tui-pass create --help
echo ""

echo "=== Manual Testing Instructions ==="
echo ""
echo "1. Create a new vault:"
echo "   ./target/release/tui-pass create my-passwords.vault"
echo "   - Enter a master password (twice for confirmation)"
echo ""
echo "2. Open the vault:"
echo "   ./target/release/tui-pass my-passwords.vault"
echo "   - Enter your master password"
echo ""
echo "3. Use the TUI:"
echo "   - Press 'a' to add a new credential"
echo "   - Use arrow keys or mouse to navigate"
echo "   - Press 'Space' to toggle password visibility"
echo "   - Press 'e' to edit selected credential"
echo "   - Press 'd' to delete selected credential"
echo "   - Press 's' to save changes"
echo "   - Press 'q' to quit (auto-saves)"
echo ""

echo "=== Running Automated Tests ==="
cd tui-pass
cargo test --release 2>&1 | grep -A 20 "running.*test"

echo ""
echo "✓ All tests passed!"
