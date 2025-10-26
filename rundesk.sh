#!/bin/bash

# rundesk.sh - Launch Data Designer Desktop Edition with all services
#
# Native egui application with full debugging capabilities
# Automatically starts gRPC server if not running

# Navigate to project root (where this script is located)
cd "$(dirname "$0")"

echo "🦀 Data Designer Desktop Edition Launcher"
echo "=========================================="
echo ""

# Set smart logging environment
# Default to warn for all crates, but enable info for our application code only
# This prevents terminal strobing from verbose tokio/reqwest/eframe logs
export RUST_LOG=warn,data_designer=info,grpc_server=info,web_ui=info
export DATABASE_URL="postgresql:///data_designer?user=adamtc007"

echo "📊 Database: $DATABASE_URL"
echo "🌐 gRPC Server: localhost:50051 (gRPC) / localhost:8080 (HTTP)"
echo "🖥️  Native debugging enabled"
echo ""

# Check if gRPC server is already running
echo "🔍 Checking gRPC server status..."
if curl -s http://localhost:8080/api/health >/dev/null 2>&1; then
    echo "✅ gRPC server already running on port 8080"
    GRPC_SERVER_PID=""
else
    echo "🚀 Starting gRPC server..."
    echo "   (This may take 1-2 minutes on first run while Cargo compiles...)"
    cd grpc-server
    cargo run &
    GRPC_SERVER_PID=$!
    cd ..

    # Wait for gRPC server to start (longer timeout for compilation)
    echo "⏳ Waiting for gRPC server to start..."
    WAIT_COUNT=0
    MAX_WAIT=120  # 2 minutes for first-time compilation

    while [ $WAIT_COUNT -lt $MAX_WAIT ]; do
        if curl -s http://localhost:8080/api/health >/dev/null 2>&1; then
            echo ""
            echo "✅ gRPC server ready on port 8080 (took ${WAIT_COUNT}s)"
            break
        fi
        sleep 1
        WAIT_COUNT=$((WAIT_COUNT + 1))
        # Show progress every 10 seconds
        if [ $((WAIT_COUNT % 10)) -eq 0 ]; then
            echo "   Still waiting... (${WAIT_COUNT}s / ${MAX_WAIT}s)"
        fi
    done

    # Check if server actually started
    if ! curl -s http://localhost:8080/api/health >/dev/null 2>&1; then
        echo ""
        echo "❌ gRPC server failed to start after ${MAX_WAIT}s"
        echo "💡 Troubleshooting:"
        echo "   1. Check if database is running: psql -U adamtc007 -d data_designer -c 'SELECT 1'"
        echo "   2. Check for port conflicts: netstat -tlnp | grep 8080"
        echo "   3. Run server manually to see errors:"
        echo "      cd grpc-server && DATABASE_URL=\"$DATABASE_URL\" cargo run"
        echo ""
        echo "   Server may still be compiling. Check background processes:"
        echo "      ps aux | grep 'cargo run'"
        if [ -n "$GRPC_SERVER_PID" ]; then
            echo ""
            echo "   Stopping failed server startup (PID: $GRPC_SERVER_PID)..."
            kill $GRPC_SERVER_PID 2>/dev/null || true
        fi
        exit 1
    fi
fi

echo ""
echo "🚀 Starting Desktop Edition with wgpu renderer..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Change to web-ui directory and run desktop binary
cd web-ui

# Trap Ctrl+C to clean up
cleanup() {
    echo ""
    echo "🛑 Shutting down..."
    if [ -n "$GRPC_SERVER_PID" ]; then
        echo "   Stopping gRPC server (PID: $GRPC_SERVER_PID)..."
        kill $GRPC_SERVER_PID 2>/dev/null || true
    fi
    echo "✅ Cleanup complete"
    exit 0
}

trap cleanup INT TERM

# Run desktop application
cargo run --bin data-designer-desktop

# Cleanup after desktop app exits
if [ -n "$GRPC_SERVER_PID" ]; then
    echo ""
    echo "🛑 Desktop app closed. Stopping gRPC server..."
    kill $GRPC_SERVER_PID 2>/dev/null || true
fi

echo "✅ Session complete"
