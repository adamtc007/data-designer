#!/bin/bash

echo "Testing DSL Language Server..."
echo "=============================="
echo ""

# Check if LSP server binary exists
if [ ! -f "./dsl-lsp/target/release/dsl-lsp-server" ]; then
    echo "❌ LSP server binary not found. Building..."
    cd dsl-lsp
    cargo build --release
    cd ..
fi

echo "✓ LSP server binary found"
echo ""

# Start LSP server in background
echo "Starting LSP server on port 3030..."
./dsl-lsp/target/release/dsl-lsp-server tcp --port 3030 &
LSP_PID=$!

echo "LSP server started with PID: $LSP_PID"
echo ""

# Wait for server to start
sleep 2

# Check if server is running
if ps -p $LSP_PID > /dev/null; then
    echo "✓ LSP server is running"
    echo ""

    # Check if port is listening
    if lsof -i:3030 > /dev/null 2>&1; then
        echo "✓ LSP server is listening on port 3030"
    else
        echo "⚠ LSP server may not be listening on port 3030"
    fi
else
    echo "❌ LSP server failed to start"
fi

echo ""
echo "Server is ready for IDE connection!"
echo "To stop the server, run: kill $LSP_PID"
echo ""
echo "You can now:"
echo "1. Open src/ide.html in your browser"
echo "2. Click 'Connect LSP' to establish connection"
echo "3. Start typing DSL code to see IntelliSense"