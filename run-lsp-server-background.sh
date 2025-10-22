#!/bin/bash

# CBU DSL Language Server Background Runner
# Starts the LSP server as a TCP server for easier testing

set -e

echo "🚀 CBU DSL Language Server (Background Mode)"
echo "============================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Default port
PORT=${1:-9257}
PID_FILE="/tmp/cbu-dsl-lsp-server.pid"
LOG_FILE="/tmp/cbu-dsl-lsp-server.log"

# Change to project directory
cd "$(dirname "$0")"

# Function to stop existing server
stop_server() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            print_status "Stopping existing LSP server (PID: $pid)..."
            kill "$pid"
            rm -f "$PID_FILE"
            print_success "Server stopped"
        else
            rm -f "$PID_FILE"
        fi
    fi
}

# Function to start server
start_server() {
    print_status "Building CBU DSL Language Server..."

    if cargo build --release --bin cbu-dsl-lsp-server; then
        print_success "LSP server built successfully"
    else
        print_error "Failed to build LSP server"
        exit 1
    fi

    LSP_BINARY="./target/release/cbu-dsl-lsp-server"
    if [ ! -f "$LSP_BINARY" ]; then
        print_error "LSP server binary not found at $LSP_BINARY"
        exit 1
    fi

    print_status "Starting CBU DSL Language Server on port $PORT..."

    # Create a wrapper script for TCP mode using socat
    if command -v socat &> /dev/null; then
        nohup socat TCP-LISTEN:$PORT,reuseaddr,fork EXEC:"$LSP_BINARY" > "$LOG_FILE" 2>&1 &
        local pid=$!
        echo "$pid" > "$PID_FILE"

        # Wait a moment to see if it started successfully
        sleep 1
        if kill -0 "$pid" 2>/dev/null; then
            print_success "LSP server started successfully"
            print_status "PID: $pid"
            print_status "Port: $PORT"
            print_status "Log file: $LOG_FILE"
            print_status "PID file: $PID_FILE"
        else
            print_error "Failed to start LSP server"
            rm -f "$PID_FILE"
            exit 1
        fi
    else
        print_warning "socat not found, starting in stdio mode..."
        nohup "$LSP_BINARY" > "$LOG_FILE" 2>&1 &
        local pid=$!
        echo "$pid" > "$PID_FILE"
        print_success "LSP server started in stdio mode (PID: $pid)"
    fi
}

# Function to show status
show_status() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            print_success "LSP server is running (PID: $pid)"
            if command -v socat &> /dev/null; then
                print_status "Listening on port: $PORT"
            fi
            print_status "Log file: $LOG_FILE"

            # Show recent log entries
            if [ -f "$LOG_FILE" ]; then
                echo ""
                print_status "Recent log entries:"
                tail -5 "$LOG_FILE" 2>/dev/null || echo "No log entries yet"
            fi
        else
            print_error "LSP server is not running (stale PID file)"
            rm -f "$PID_FILE"
        fi
    else
        print_warning "LSP server is not running"
    fi
}

# Function to test the server
test_server() {
    if command -v nc &> /dev/null; then
        print_status "Testing LSP server connection..."

        # Send a simple LSP initialize request
        local test_request='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}'
        local content_length=${#test_request}

        {
            echo -e "Content-Length: $content_length\r\n\r\n$test_request"
        } | nc localhost "$PORT" | head -10

        print_status "Test completed"
    else
        print_warning "netcat (nc) not available for testing"
    fi
}

# Parse command line arguments
case "${1:-start}" in
    "start")
        stop_server
        start_server
        ;;
    "stop")
        stop_server
        ;;
    "restart")
        stop_server
        start_server
        ;;
    "status")
        show_status
        ;;
    "test")
        test_server
        ;;
    "logs")
        if [ -f "$LOG_FILE" ]; then
            tail -f "$LOG_FILE"
        else
            print_error "Log file not found: $LOG_FILE"
        fi
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|test|logs} [port]"
        echo ""
        echo "Commands:"
        echo "  start   - Start the LSP server (default)"
        echo "  stop    - Stop the running server"
        echo "  restart - Restart the server"
        echo "  status  - Show server status"
        echo "  test    - Test server connection"
        echo "  logs    - Follow server logs"
        echo ""
        echo "Examples:"
        echo "  $0 start 9257    # Start on port 9257"
        echo "  $0 status        # Check if running"
        echo "  $0 test          # Test connection"
        echo "  $0 logs          # View logs"
        exit 1
        ;;
esac