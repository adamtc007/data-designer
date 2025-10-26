#!/bin/bash

# Data Designer WASM Runner
# Builds and serves the WASM web version with one command

set -e

# Ensure cargo bin is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

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

# Parse command line arguments
START_LSP=false
LSP_PORT=9257

for arg in "$@"; do
    case $arg in
        --with-lsp|--lsp)
            START_LSP=true
            shift
            ;;
        --lsp-port=*)
            LSP_PORT="${arg#*=}"
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --with-lsp, --lsp     Start LSP server alongside WASM app"
            echo "  --lsp-port=PORT       LSP server port (default: 9257)"
            echo "  --help, -h            Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                    # Start WASM app only"
            echo "  $0 --with-lsp         # Start WASM app + LSP server"
            echo "  $0 --lsp --lsp-port=9999  # Custom LSP port"
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Check Docker status first
check_docker

# Kill any existing processes on ports 8080, 8081, 50051, and LSP port
echo "🔧 Cleaning up existing processes..."
if lsof -ti:8080 >/dev/null 2>&1; then
    echo "   Killing processes on port 8080..."
    lsof -ti:8080 | xargs kill -9 2>/dev/null || true
else
    echo "   Port 8080 is free"
fi

if lsof -ti:8081 >/dev/null 2>&1; then
    echo "   Killing processes on port 8081..."
    lsof -ti:8081 | xargs kill -9 2>/dev/null || true
else
    echo "   Port 8081 is free"
fi

if lsof -ti:50051 >/dev/null 2>&1; then
    echo "   Killing processes on port 50051..."
    lsof -ti:50051 | xargs kill -9 2>/dev/null || true
else
    echo "   Port 50051 is free"
fi

if [ "$START_LSP" = true ]; then
    if lsof -ti:$LSP_PORT >/dev/null 2>&1; then
        echo "   Killing processes on LSP port $LSP_PORT..."
        lsof -ti:$LSP_PORT | xargs kill -9 2>/dev/null || true
    else
        echo "   LSP port $LSP_PORT is free"
    fi
fi

# Set database URL for gRPC server (required for sqlx compile-time query verification)
# Use Unix socket instead of TCP to avoid password authentication
export DATABASE_URL="postgresql:///data_designer?user=adamtc007"

# Check if gRPC server is already running
echo "🔍 Checking gRPC server status..."
if curl -s http://localhost:8080/api/health >/dev/null 2>&1; then
    echo "✅ gRPC server already running on port 50051 + HTTP API on port 8080"
    GRPC_SERVER_PID=""
else
    echo "🚀 Starting gRPC server..."
    cd grpc-server
    DATABASE_URL="$DATABASE_URL" cargo run &
    GRPC_SERVER_PID=$!
    cd ..

    # Wait for gRPC server to start (longer timeout for compilation)
    echo "⏳ Waiting for gRPC server to start..."
    echo "   (This may take 1-2 minutes on first run while Cargo compiles...)"
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

    # Check if gRPC server HTTP API is running
    if ! curl -s http://localhost:8080/api/health >/dev/null 2>&1; then
        echo ""
        echo "❌ gRPC server failed to start after ${MAX_WAIT}s"
        echo "💡 Server may still be compiling. Check: ps aux | grep 'cargo run'"
        kill $GRPC_SERVER_PID 2>/dev/null || true
        exit 1
    fi
fi

# Start LSP server if requested
LSP_SERVER_PID=""
if [ "$START_LSP" = true ]; then
    echo "🎨 Starting CBU DSL Language Server..."

    # Build LSP server first
    echo "📦 Building LSP server..."
    if cargo build --release --bin cbu-dsl-lsp-server; then
        echo "✅ LSP server built successfully"
    else
        echo "❌ Failed to build LSP server"
        if [ -n "$GRPC_SERVER_PID" ]; then
            kill $GRPC_SERVER_PID 2>/dev/null || true
        fi
        exit 1
    fi

    # Start LSP server with socat for TCP access
    if command -v socat &> /dev/null; then
        echo "🚀 Starting LSP server on port $LSP_PORT..."
        socat TCP-LISTEN:$LSP_PORT,reuseaddr,fork EXEC:"./target/release/cbu-dsl-lsp-server" >/dev/null 2>&1 &
        LSP_SERVER_PID=$!

        # Wait for LSP server to start
        sleep 2
        if kill -0 $LSP_SERVER_PID 2>/dev/null; then
            echo "✅ CBU DSL Language Server ready on port $LSP_PORT"
            echo "   Features: Syntax highlighting, code completion, diagnostics"
            echo "   Protocol: LSP over TCP"
        else
            echo "❌ LSP server failed to start"
            LSP_SERVER_PID=""
        fi
    else
        echo "⚠️  socat not found - LSP server will run in stdio mode only"
        echo "   Install socat for TCP access: brew install socat"
    fi
fi

# Navigate to web-ui directory
cd web-ui

# Build the WASM package
echo "📦 Building WASM package..."
./build-web.sh

# Start the web server in background
echo "🚀 Starting web server..."
./serve-web.sh &
SERVER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 3

# Check if server is running
if ! curl -s http://localhost:8081 >/dev/null 2>&1; then
    echo "❌ Web server failed to start"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

echo "✅ Data Designer Web Edition is ready!"
echo ""
echo "🌐 URL: http://localhost:8081"
echo "📁 Serving from: web-ui/dist/"
echo "🔧 Web Server PID: $SERVER_PID (port 8081)"
if [ -n "$GRPC_SERVER_PID" ]; then
    echo "🔧 gRPC Server PID: $GRPC_SERVER_PID (port 50051 + HTTP API on 8080)"
else
    echo "🔧 gRPC Server: Already running (port 50051 + HTTP API on 8080)"
fi
if [ -n "$LSP_SERVER_PID" ]; then
    echo "🎨 LSP Server PID: $LSP_SERVER_PID (port $LSP_PORT)"
    echo "   Navigate to CBU DSL IDE for enhanced editing features!"
fi
echo ""
echo "🚀 All services ready! Press Ctrl+C to stop servers"

# Keep script running and handle Ctrl+C
PIDS_TO_KILL="$SERVER_PID"
[ -n "$GRPC_SERVER_PID" ] && PIDS_TO_KILL="$PIDS_TO_KILL $GRPC_SERVER_PID"
[ -n "$LSP_SERVER_PID" ] && PIDS_TO_KILL="$PIDS_TO_KILL $LSP_SERVER_PID"

trap "echo ''; echo '🛑 Stopping servers...'; kill $PIDS_TO_KILL 2>/dev/null || true; echo '✅ All servers stopped'; exit 0" INT

# Wait for all server processes
if [ -n "$GRPC_SERVER_PID" ] && [ -n "$LSP_SERVER_PID" ]; then
    wait $SERVER_PID $GRPC_SERVER_PID $LSP_SERVER_PID
elif [ -n "$GRPC_SERVER_PID" ]; then
    wait $SERVER_PID $GRPC_SERVER_PID
elif [ -n "$LSP_SERVER_PID" ]; then
    wait $SERVER_PID $LSP_SERVER_PID
else
    wait $SERVER_PID
fi