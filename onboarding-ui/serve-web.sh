#!/bin/bash

# Serve Onboarding Platform Web UI
set -e

echo "🌐 Starting web server for Onboarding Platform..."
echo ""

# Check if miniserve is installed
if ! command -v miniserve &> /dev/null; then
    echo "📦 miniserve not found, installing..."
    cargo install miniserve
fi

# Check if dist exists
if [ ! -d "dist" ]; then
    echo "❌ dist/ directory not found. Run ./build-web.sh first!"
    exit 1
fi

echo "✅ Serving onboarding platform at http://localhost:8000"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

miniserve dist/ --port 8000 --index index.html
