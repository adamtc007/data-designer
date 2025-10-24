#!/bin/bash

# Start AI Context Server for Data Designer
# Provides real-time codebase context to AI assistants

set -e

echo "🤖 Starting AI Context Server for Data Designer"
echo "=============================================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
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

# Default configuration
PORT=${1:-3737}
SCAN_INTERVAL=${2:-30}

# Check Python dependencies
print_status "Checking Python dependencies..."

python3 -c "import flask, flask_cors" 2>/dev/null || {
    print_warning "Installing required Python packages..."
    pip3 install flask flask-cors || {
        print_error "Failed to install Python dependencies"
        echo ""
        echo "Please install manually:"
        echo "  pip3 install flask flask-cors"
        exit 1
    }
}

print_success "Python dependencies available"

# Check if port is available
if lsof -ti:$PORT >/dev/null 2>&1; then
    print_warning "Port $PORT is already in use"
    print_status "Attempting to stop existing process..."
    lsof -ti:$PORT | xargs kill -9 2>/dev/null || true
    sleep 2
fi

# Create logs directory
mkdir -p logs

# Export environment variables for the server
export PYTHONPATH="$PWD:$PYTHONPATH"
export FLASK_ENV=production

print_status "Starting AI Context Server..."
print_status "Port: $PORT"
print_status "Scan interval: ${SCAN_INTERVAL}s"
print_status "Project root: $(pwd)"

# Start the server
python3 ai-context-server.py \
    --port $PORT \
    --project-root . \
    --scan-interval $SCAN_INTERVAL \
    2>&1 | tee logs/ai-context-server.log &

SERVER_PID=$!

# Wait for server to start
echo ""
print_status "Waiting for server to start..."
sleep 3

# Check if server is running
if kill -0 $SERVER_PID 2>/dev/null; then
    # Test server health
    if curl -s http://localhost:$PORT/health >/dev/null 2>&1; then
        print_success "AI Context Server started successfully!"
        echo ""
        echo "🌐 Server Information:"
        echo "   URL: http://localhost:$PORT"
        echo "   PID: $SERVER_PID"
        echo "   Logs: logs/ai-context-server.log"
        echo ""
        echo "📡 Available Endpoints:"
        echo "   • GET /health - Server health check"
        echo "   • GET /api/codebase/current - Current codebase snapshot"
        echo "   • GET /api/codebase/files - List all files with filtering"
        echo "   • GET /api/codebase/file/<path> - Get specific file content"
        echo "   • GET /api/codebase/search?q=<query> - Search file contents"
        echo "   • GET /api/codebase/exports - List AI export files"
        echo "   • GET /api/codebase/history - Codebase change history"
        echo "   • GET /api/codebase/stats - Comprehensive statistics"
        echo "   • GET /api/ai/context - AI-specific context information"
        echo ""
        echo "🤖 AI Assistant Usage:"
        echo "   1. Access real-time codebase information via HTTP API"
        echo "   2. Search code content across all files"
        echo "   3. Monitor file changes and history"
        echo "   4. Download AI-friendly exports automatically"
        echo "   5. Get project statistics and metadata"
        echo ""
        echo "💡 Example API calls:"
        echo "   curl http://localhost:$PORT/api/ai/context"
        echo "   curl http://localhost:$PORT/api/codebase/search?q=create-cbu"
        echo "   curl http://localhost:$PORT/api/codebase/files?type=rust"
        echo ""
        echo "🔧 Management Commands:"
        echo "   • View logs: tail -f logs/ai-context-server.log"
        echo "   • Stop server: kill $SERVER_PID"
        echo "   • Restart: $0 $PORT $SCAN_INTERVAL"
        echo ""
        print_success "AI Context Server ready for AI assistant integration!"

        # Save PID for management
        echo $SERVER_PID > logs/ai-context-server.pid

        # Keep script running to handle Ctrl+C
        trap "echo ''; echo '🛑 Stopping AI Context Server...'; kill $SERVER_PID 2>/dev/null || true; rm -f logs/ai-context-server.pid; echo '✅ Server stopped'; exit 0" INT

        # Wait for server process
        wait $SERVER_PID

    else
        print_error "Server started but health check failed"
        kill $SERVER_PID 2>/dev/null || true
        exit 1
    fi
else
    print_error "Failed to start AI Context Server"
    echo ""
    echo "Check logs for details:"
    echo "  tail logs/ai-context-server.log"
    exit 1
fi