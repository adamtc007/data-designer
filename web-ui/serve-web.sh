#!/bin/bash

# Serve script for Data Designer Web using miniserve
set -e

echo "🌐 Starting Data Designer Web Edition..."

# Check if miniserve is installed
if ! command -v miniserve &> /dev/null; then
    echo "❌ miniserve is not installed. Installing..."
    if command -v cargo &> /dev/null; then
        cargo install miniserve
    else
        echo "Please install miniserve: cargo install miniserve"
        exit 1
    fi
fi

# Check if dist directory exists
if [ ! -d "dist" ]; then
    echo "❌ dist/ directory not found. Building first..."
    ./build-web.sh
fi

echo "🚀 Starting web server..."
echo "📁 Serving from: dist/"
echo "🌐 URL: http://localhost:8081"
echo ""

# Start miniserve with appropriate settings
miniserve dist/ \
    --port 8081 \
    --index index.html \
    --header "Cross-Origin-Embedder-Policy: require-corp" \
    --header "Cross-Origin-Opener-Policy: same-origin" \
    --header "Cache-Control: no-cache"