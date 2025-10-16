#!/bin/bash

# Data Designer WASM Runner
# Builds and serves the WASM web version with one command

set -e

function check_docker() {
    echo "🐳 Checking Docker status..."

    # Check if Docker daemon is running
    if ! docker info >/dev/null 2>&1; then
        echo "⚠️  Docker daemon not running. Starting Docker Desktop..."

        # Start Docker Desktop on macOS
        if [[ "$OSTYPE" == "darwin"* ]]; then
            open /Applications/Docker.app
            echo "⏳ Waiting for Docker Desktop to start..."

            # Wait up to 60 seconds for Docker to be ready
            local count=0
            while ! docker info >/dev/null 2>&1 && [ $count -lt 60 ]; do
                sleep 2
                count=$((count + 2))
                echo -n "."
            done
            echo ""

            if docker info >/dev/null 2>&1; then
                echo "✅ Docker Desktop started successfully"
            else
                echo "❌ Failed to start Docker Desktop. Please start it manually."
                exit 1
            fi
        else
            echo "❌ Docker daemon not running. Please start Docker manually."
            exit 1
        fi
    else
        echo "✅ Docker daemon is running"
    fi
}

echo "🦀 Data Designer WASM Runner"
echo "=================================="

# Check Docker status first
check_docker

# Kill any existing processes on ports 8080 and 3030
echo "🔧 Cleaning up existing processes..."
if lsof -ti:8080 >/dev/null 2>&1; then
    echo "   Killing processes on port 8080..."
    lsof -ti:8080 | xargs kill -9 2>/dev/null || true
else
    echo "   Port 8080 is free"
fi

if lsof -ti:3030 >/dev/null 2>&1; then
    echo "   Killing processes on port 3030..."
    lsof -ti:3030 | xargs kill -9 2>/dev/null || true
else
    echo "   Port 3030 is free"
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