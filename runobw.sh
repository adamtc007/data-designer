#!/bin/bash

# Launch Onboarding Platform - Web (WASM)
set -e

echo "🚀 Starting Onboarding Workflow Platform (Web)"
echo "==============================================="
echo ""

cd onboarding-ui

# Build WASM
echo "📦 Building WASM package..."
./build-web.sh

# Set database URL for backend
export DATABASE_URL="postgresql:///data_designer?user=adamtc007"

# Check if backend is running
if ! curl -s http://localhost:8080/api/health > /dev/null 2>&1; then
    echo ""
    echo "⚠️  Backend server not running!"
    echo "   Starting backend server in background..."
    cd ..
    cargo run --bin grpc-server > /tmp/onboarding-backend.log 2>&1 &
    BACKEND_PID=$!
    echo "   Backend PID: $BACKEND_PID"
    cd onboarding-ui
    echo "   Waiting for server to start..."
    sleep 3
fi

echo ""
echo "✅ Backend server is running"
echo ""
echo "🌐 Starting web server..."

# Start web server
./serve-web.sh
