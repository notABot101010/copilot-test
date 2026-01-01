#!/bin/bash

# Qt Chat Server - Manual Testing Script
# This script demonstrates the server API functionality

BASE_URL="http://localhost:3000"

echo "=== Qt Chat Server Testing ==="
echo ""

# Check if server is running
if ! curl -s "$BASE_URL/rooms" > /dev/null 2>&1; then
    echo "Error: Server is not running. Start it with:"
    echo "  cd qt-chat-server && cargo run --release"
    exit 1
fi

echo "✓ Server is running"
echo ""

# Create a room
echo "1. Creating a room..."
ROOM_RESPONSE=$(curl -s -X POST "$BASE_URL/rooms" \
    -H "Content-Type: application/json" \
    -d '{"name": "Testing Room", "creator": "TestUser1"}')
ROOM_ID=$(echo "$ROOM_RESPONSE" | grep -o '"room_id":"[^"]*"' | cut -d'"' -f4)
echo "   Room created with ID: $ROOM_ID"
echo ""

# Join the room as second user
echo "2. Joining room as second user..."
curl -s -X POST "$BASE_URL/rooms/$ROOM_ID/join" \
    -H "Content-Type: application/json" \
    -d '{"username": "TestUser2"}' > /dev/null
echo "   ✓ TestUser2 joined the room"
echo ""

# Send a message from User1
echo "3. Sending message from TestUser1..."
curl -s -X POST "$BASE_URL/rooms/$ROOM_ID/messages" \
    -H "Content-Type: application/json" \
    -d '{"sender": "TestUser1", "content": "Hello from User1!"}' > /dev/null
echo "   ✓ Message sent"
echo ""

# Send a message from User2
echo "4. Sending message from TestUser2..."
curl -s -X POST "$BASE_URL/rooms/$ROOM_ID/messages" \
    -H "Content-Type: application/json" \
    -d '{"sender": "TestUser2", "content": "Hi User1! Great to chat with you."}' > /dev/null
echo "   ✓ Message sent"
echo ""

# Get all messages
echo "5. Retrieving all messages..."
MESSAGES=$(curl -s "$BASE_URL/rooms/$ROOM_ID/messages")
echo "$MESSAGES" | python3 -m json.tool 2>/dev/null || echo "$MESSAGES"
echo ""

# List all rooms
echo "6. Listing all rooms..."
ROOMS=$(curl -s "$BASE_URL/rooms")
echo "$ROOMS" | python3 -m json.tool 2>/dev/null || echo "$ROOMS"
echo ""

echo "=== Test Complete ==="
echo ""
echo "To test with the web client, open qt-chat-demo/index.html in your browser"
