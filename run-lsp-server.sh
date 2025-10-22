#!/bin/bash

# CBU DSL Language Server Runner
# Builds and starts the CBU DSL Language Server Protocol server

set -e

echo "🚀 CBU DSL Language Server"
echo "=========================="

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

# Change to project directory
cd "$(dirname "$0")"

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Rust and Cargo."
    exit 1
fi

print_status "Building CBU DSL Language Server..."

# Build the LSP server
if cargo build --release --bin cbu-dsl-lsp-server; then
    print_success "LSP server built successfully"
else
    print_error "Failed to build LSP server"
    exit 1
fi

# Check if the binary exists
LSP_BINARY="./target/release/cbu-dsl-lsp-server"
if [ ! -f "$LSP_BINARY" ]; then
    print_error "LSP server binary not found at $LSP_BINARY"
    exit 1
fi

print_status "Starting CBU DSL Language Server..."
print_status "Server will listen on stdin/stdout for LSP communication"
print_warning "Press Ctrl+C to stop the server"

echo ""
echo "📋 Language Server Features:"
echo "  • Syntax highlighting with semantic tokens"
echo "  • Code completion for CBU DSL functions and entity roles"
echo "  • Hover documentation"
echo "  • Real-time error diagnostics"
echo "  • S-expression validation"

echo ""
echo "🔧 LSP Client Configuration:"
echo "  Language ID: cbu-dsl"
echo "  File Extensions: .lisp, .cbu"
echo "  Command: $LSP_BINARY"
echo "  Transport: stdio"

echo ""
echo "💡 VS Code Integration:"
echo "  Add to settings.json:"
echo '  "cbu-dsl-lsp.serverPath": "'$(pwd)'/target/release/cbu-dsl-lsp-server"'

echo ""
print_status "Starting server (stdio mode)..."

# Start the LSP server
# The server communicates via stdin/stdout following LSP protocol
exec "$LSP_BINARY"