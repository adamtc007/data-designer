#!/bin/bash

# rundesk.sh - Launch Data Designer Desktop Edition
#
# Native egui application with full debugging capabilities
# Connects to the same gRPC server as the web version

# Navigate to project root (where this script is located)
cd "$(dirname "$0")"

echo "🦀 Starting Data Designer Desktop Edition..."
echo "📊 Database: postgresql://adamtc007@localhost/data_designer"
echo "🌐 gRPC Server: localhost:50051 (gRPC) / localhost:8080 (HTTP)"
echo "🖥️  Native debugging enabled"
echo ""

# Set smart logging environment
# Default to warn for all crates, but enable info for our application code only
# This prevents terminal strobing from verbose tokio/reqwest/eframe logs
export RUST_LOG=warn,data_designer=info,grpc_server=info,web_ui=info

# Change to web-ui directory and run desktop binary
cd web-ui && cargo run --bin data-designer-desktop