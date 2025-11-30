#!/bin/bash

# Reset script for e2ee-crdt application
# This removes the database to start fresh

echo "Resetting e2ee-crdt application..."

# Remove the database file if it exists
if [ -f "server/crdt.db" ]; then
    rm server/crdt.db
    echo "✓ Removed server/crdt.db"
else
    echo "✓ No database file found"
fi

# Remove the WAL and SHM files if they exist
if [ -f "server/crdt.db-shm" ]; then
    rm server/crdt.db-shm
    echo "✓ Removed server/crdt.db-shm"
fi

if [ -f "server/crdt.db-wal" ]; then
    rm server/crdt.db-wal
    echo "✓ Removed server/crdt.db-wal"
fi

echo ""
echo "Database reset complete!"
echo ""
echo "Next steps:"
echo "1. Clear your browser storage (DevTools → Application → Clear Storage)"
echo "2. Restart the server: cd server && cargo run"
echo "3. Restart the client: cd webapp && npm run dev"
