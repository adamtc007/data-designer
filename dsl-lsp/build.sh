#!/bin/bash

# Build the DSL Language Server
echo "Building DSL Language Server..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "To run the LSP server:"
    echo "  ./target/release/dsl-lsp-server"
    echo ""
    echo "To use with VS Code:"
    echo "  1. Install the 'Generic LSP Client' extension"
    echo "  2. Configure it to use: $PWD/target/release/dsl-lsp-server"
    echo ""
    echo "For Monaco Editor integration in the web app:"
    echo "  The DSL language is already configured with:"
    echo "  - Syntax highlighting"
    echo "  - Auto-completion for KYC attributes"
    echo "  - Function and operator hints"
else
    echo "❌ Build failed"
    exit 1
fi