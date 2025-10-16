#!/bin/bash

# Data Designer WASM Runner
# Builds and serves the WASM web version with one command

set -e

echo "🦀 Data Designer WASM Runner"
echo "=================================="

# Kill any existing processes on port 8080
echo "🔧 Cleaning up existing processes..."
if lsof -ti:8080 >/dev/null 2>&1; then
    echo "   Killing processes on port 8080..."
    lsof -ti:8080 | xargs kill -9 2>/dev/null || true
else
    echo "   Port 8080 is free"
fi

# Navigate to web-ui directory
cd web-ui

# Build the WASM package
echo "📦 Building WASM package..."
./build-web.sh

# Start the server in background
echo "🚀 Starting web server..."
./serve-web.sh &
SERVER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 3

# Check if server is running
if ! curl -s http://localhost:8080 >/dev/null 2>&1; then
    echo "❌ Server failed to start"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

echo "✅ Data Designer Web Edition is ready!"
echo ""
echo "🌐 URL: http://localhost:8080"
echo "📁 Serving from: web-ui/dist/"
echo "🔧 Server PID: $SERVER_PID"
echo ""
echo "Press Ctrl+C to stop the server"

# Keep script running and handle Ctrl+C
trap "echo ''; echo '🛑 Stopping server...'; kill $SERVER_PID 2>/dev/null || true; echo '✅ Server stopped'; exit 0" INT

# Wait for server process
wait $SERVER_PID