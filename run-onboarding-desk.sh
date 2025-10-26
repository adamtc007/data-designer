#!/bin/bash

# Launch Onboarding Platform - Desktop (Native)
set -e

echo "🚀 Starting Onboarding Workflow Platform (Desktop)"
echo "=================================================="
echo ""

# Set database URL for backend
export DATABASE_URL="postgresql:///data_designer?user=adamtc007"

# Check if backend is running
if ! curl -s http://localhost:8080/api/health > /dev/null 2>&1; then
    echo "⚠️  Backend server not running!"
    echo "   Starting backend server in background..."
    cargo run --bin grpc-server > /tmp/onboarding-backend.log 2>&1 &
    BACKEND_PID=$!
    echo "   Backend PID: $BACKEND_PID"
    echo "   Waiting for server to start..."
    sleep 3
fi

echo "✅ Backend server is running"
echo ""
echo "🖥️  Launching Onboarding Desktop App..."
echo ""

cargo run --bin onboarding-desktop --features tokio

echo ""
echo "👋 Onboarding Platform closed"
