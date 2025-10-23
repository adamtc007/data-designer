#!/bin/bash

# rundesk.sh - Launch Data Designer Desktop Edition
#
# Native egui application with full debugging capabilities
# Connects to the same gRPC server as the web version

echo "🦀 Starting Data Designer Desktop Edition..."
echo "📊 Database: postgresql://adamtc007@localhost/data_designer"
echo "🌐 gRPC Server: localhost:50051 (gRPC) / localhost:8080 (HTTP)"
echo "🖥️  Native debugging enabled"
echo ""

# Set debugging environment
export RUST_LOG=debug

# Change to web-ui directory and run desktop binary
cd web-ui && cargo run --bin data-designer-desktop