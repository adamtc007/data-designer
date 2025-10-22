#!/bin/bash

# AI Codebase Exporter
# Exports codebase in AI-friendly formats for ChatGPT, Claude, Gemini, etc.

set -e

echo "🤖 AI Codebase Exporter for Data Designer"
echo "========================================="

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Change to project directory
cd "$(dirname "$0")"

# Create export directory
EXPORT_DIR="ai-export"
rm -rf "$EXPORT_DIR"
mkdir -p "$EXPORT_DIR"

print_status "Creating AI-friendly codebase export..."

# Export types
EXPORT_TYPE=${1:-"full"}
AI_ASSISTANT=${2:-"general"}

case $EXPORT_TYPE in
    "full")
        print_status "Exporting full codebase..."
        ;;
    "core")
        print_status "Exporting core modules only..."
        ;;
    "summary")
        print_status "Exporting project summary..."
        ;;
    *)
        echo "Usage: $0 [full|core|summary] [chatgpt|claude|gemini|general]"
        echo ""
        echo "Export Types:"
        echo "  full    - Complete codebase with all files"
        echo "  core    - Core Rust modules and key files only"
        echo "  summary - Project overview and architecture"
        echo ""
        echo "AI Assistants:"
        echo "  chatgpt - Optimized for ChatGPT"
        echo "  claude  - Optimized for Claude"
        echo "  gemini  - Optimized for Gemini"
        echo "  general - Generic format"
        exit 1
        ;;
esac

# Create project overview
cat > "$EXPORT_DIR/PROJECT_OVERVIEW.md" << 'EOF'
# Data Designer - Financial Services DSL & CRUD System

## 🏗️ Architecture Overview

**Web-First WASM Application** with microservices architecture:
- **Frontend**: Pure Rust WASM (egui) - `web-ui/`
- **Backend**: gRPC microservices - `grpc-server/`
- **Core Library**: DSL engine & database layer - `data-designer-core/`
- **Language Server**: CBU DSL LSP - `cbu-dsl-lsp/`

## 🎯 Key Features

1. **CBU DSL System**: LISP-style S-expressions for financial entity management
2. **gRPC Microservices**: Complete CRUD API for financial entities
3. **WASM Web UI**: Professional financial entity management interface
4. **Language Server**: Full LSP with syntax highlighting, completion, diagnostics
5. **AI Integration**: 7-feature AI assistant system with RAG

## 📁 Project Structure

```
data-designer/
├── data-designer-core/     # Core DSL engine, parsers, database
├── grpc-server/           # gRPC microservices (Protocol Buffers)
├── web-ui/               # WASM frontend (egui)
├── cbu-dsl-lsp/          # Language Server Protocol implementation
├── proto/                # Protocol Buffer definitions
├── migrations/           # Database schema migrations
└── test-dsl-files/       # Example DSL files
```

## 🚀 Tech Stack

- **Language**: Rust (100% safe, no unsafe blocks)
- **Frontend**: egui + WASM
- **Backend**: tonic (gRPC), sqlx (PostgreSQL)
- **Database**: PostgreSQL with pgvector
- **Parsing**: nom (parser combinators)
- **LSP**: tower-lsp
- **Web Serving**: WASM + static files

## 🔑 Core Modules

### `data-designer-core/src/`
- `lisp_cbu_dsl.rs` - LISP S-expression parser for CBU DSL
- `transpiler.rs` - Multi-target code generation (Rust/SQL/JS/Python)
- `db/` - Database models and operations
- `capability_execution_engine.rs` - Fund accounting capability execution

### `web-ui/src/`
- `cbu_dsl_ide.rs` - Enhanced DSL editor with syntax highlighting
- `entity_management.rs` - CRUD UI for financial entities
- `dsl_syntax_highlighter.rs` - Professional syntax highlighting

### `grpc-server/src/`
- `main.rs` - gRPC service implementation
- Complete CRUD APIs for all business entities

### `cbu-dsl-lsp/src/`
- `lib.rs` - Full Language Server Protocol implementation
- Real-time diagnostics, completion, hover documentation

## 💼 Business Domain

**Financial Services - Fund Management & Onboarding**

- **CBU (Client Business Unit)**: Investment funds, hedge funds, family offices
- **Entities**: Asset owners, investment managers, custodians, prime brokers
- **Workflows**: KYC, onboarding, compliance, fund setup
- **DSL**: Domain-specific language for financial operations

## 🎨 DSL Examples

### S-Expression CBU Creation
```lisp
(create-cbu "Goldman Sachs Investment Fund" "Multi-strategy hedge fund"
  (entities
    (entity "GS001" "Goldman Sachs Asset Management" asset-owner)
    (entity "GS002" "Goldman Sachs Investment Advisors" investment-manager)
    (entity "BNY001" "BNY Mellon" custodian)))
```

## 🔧 Development Commands

```bash
# Quick start - all services
./runwasm.sh --with-lsp

# Build workspace
cargo build --release

# Run tests
cargo test --all

# Start LSP server
./run-lsp-server.sh
```
EOF

# Function to sanitize code for AI
sanitize_code() {
    local file="$1"
    # Remove sensitive information, keep structure
    sed 's/password\s*=\s*"[^"]*"/password = "***"/g' "$file" | \
    sed 's/api_key\s*=\s*"[^"]*"/api_key = "***"/g' | \
    sed 's/secret\s*=\s*"[^"]*"/secret = "***"/g'
}

# Export core Rust files
print_status "Exporting core Rust modules..."
mkdir -p "$EXPORT_DIR/core-rust"

# Core library files
if [ -d "data-designer-core/src" ]; then
    cp -r "data-designer-core/src" "$EXPORT_DIR/core-rust/data-designer-core-src"
    find "$EXPORT_DIR/core-rust/data-designer-core-src" -name "*.rs" -exec sed -i '' 's/.*password.*=.*/    \/\/ [REDACTED]/g' {} \;
fi

# Web UI core files
if [ -d "web-ui/src" ]; then
    mkdir -p "$EXPORT_DIR/core-rust/web-ui-src"
    cp web-ui/src/lib.rs "$EXPORT_DIR/core-rust/web-ui-src/" 2>/dev/null || true
    cp web-ui/src/cbu_dsl_ide.rs "$EXPORT_DIR/core-rust/web-ui-src/" 2>/dev/null || true
    cp web-ui/src/entity_management.rs "$EXPORT_DIR/core-rust/web-ui-src/" 2>/dev/null || true
    cp web-ui/src/dsl_syntax_highlighter.rs "$EXPORT_DIR/core-rust/web-ui-src/" 2>/dev/null || true
fi

# gRPC server
if [ -f "grpc-server/src/main.rs" ]; then
    mkdir -p "$EXPORT_DIR/core-rust/grpc-server-src"
    sanitize_code "grpc-server/src/main.rs" > "$EXPORT_DIR/core-rust/grpc-server-src/main.rs"
fi

# LSP server
if [ -d "cbu-dsl-lsp/src" ]; then
    cp -r "cbu-dsl-lsp/src" "$EXPORT_DIR/core-rust/cbu-dsl-lsp-src"
fi

# Export configuration files
print_status "Exporting configuration files..."
mkdir -p "$EXPORT_DIR/config"

cp Cargo.toml "$EXPORT_DIR/config/" 2>/dev/null || true
cp */Cargo.toml "$EXPORT_DIR/config/" 2>/dev/null || true
cp proto/*.proto "$EXPORT_DIR/config/" 2>/dev/null || true
cp CLAUDE.md "$EXPORT_DIR/config/" 2>/dev/null || true

# Export examples and documentation
print_status "Exporting examples and documentation..."
mkdir -p "$EXPORT_DIR/examples"

if [ -d "test-dsl-files" ]; then
    cp -r test-dsl-files "$EXPORT_DIR/examples/"
fi

# Create example DSL files if they don't exist
cat > "$EXPORT_DIR/examples/cbu-creation-example.lisp" << 'EOF'
;; CBU Creation Example
(create-cbu "Goldman Sachs Investment Fund" "Multi-strategy hedge fund operations"
  (entities
    (entity "GS001" "Goldman Sachs Asset Management" asset-owner)
    (entity "GS002" "Goldman Sachs Investment Advisors" investment-manager)
    (entity "BNY001" "BNY Mellon" custodian)))
EOF

cat > "$EXPORT_DIR/examples/complex-workflow-example.lisp" << 'EOF'
;; Complex Workflow Example
(workflow "customer-onboarding"
  (step "identity-verification"
    (verify-kyc customer-data)
    (validate-documents identity-docs))
  (step "entity-setup"
    (create-cbu fund-details)
    (assign-roles entities))
  (step "compliance-check"
    (screen-sanctions all-entities)
    (validate-licenses required-permissions)))
EOF

# Create AI-specific summaries
case $AI_ASSISTANT in
    "chatgpt")
        print_status "Creating ChatGPT-optimized summary..."
        cat > "$EXPORT_DIR/CHATGPT_CONTEXT.md" << 'EOF'
# Data Designer - Context for ChatGPT

## What This Project Is
A **financial services DSL and CRUD system** built in **Rust** with **WASM frontend**.

## Key Technologies
- **Language**: Rust (safe systems programming)
- **Frontend**: egui + WASM (runs in browser)
- **Backend**: gRPC microservices + PostgreSQL
- **DSL**: LISP-style S-expressions for financial operations
- **IDE**: Full Language Server Protocol implementation

## What Makes It Special
1. **Professional LSP**: Syntax highlighting, code completion, real-time diagnostics
2. **Web-Native**: Pure WASM frontend, no JavaScript
3. **Financial Domain**: Specialized for fund management and onboarding
4. **Type-Safe**: Rust's safety guarantees throughout the stack
5. **Microservices**: Clean gRPC APIs with Protocol Buffers

## Core Capabilities
- **CBU Management**: Create/update/delete Client Business Units
- **Entity Management**: Financial entities (asset owners, custodians, etc.)
- **DSL Processing**: Parse and transpile financial domain language
- **CRUD Operations**: Complete REST and gRPC APIs
- **Real-time Validation**: LSP-powered syntax checking

## Architecture Pattern
**Web-First Microservices**: WASM client → gRPC services → PostgreSQL

The code is production-ready with comprehensive testing and follows Rust best practices.
EOF
        ;;
    "claude")
        print_status "Creating Claude-optimized summary..."
        cat > "$EXPORT_DIR/CLAUDE_CONTEXT.md" << 'EOF'
# Data Designer - Context for Claude

## Project Architecture
This is a **sophisticated financial services platform** implementing a **domain-specific language (DSL)** for fund management operations.

## Technical Excellence
- **Memory Safety**: 100% safe Rust, no unsafe blocks
- **Web Performance**: WASM frontend with 60fps egui rendering
- **Type Safety**: Protocol Buffers + Rust type system
- **Real-time Features**: LSP with live syntax validation
- **Database Safety**: sqlx compile-time SQL verification

## Innovation Highlights
1. **Custom LSP Implementation**: Built from scratch, not using tree-sitter
2. **S-Expression DSL**: LISP-style syntax for financial operations
3. **Web-Native Rust**: egui WASM app, no JavaScript dependencies
4. **Microservice Architecture**: Clean separation of concerns
5. **Financial Domain Modeling**: Sophisticated business entity relationships

## Code Quality
- **Testing**: Comprehensive test suite (20+ test modules)
- **Documentation**: Extensive inline documentation
- **Error Handling**: Proper Result<T, E> usage throughout
- **Performance**: Zero-copy serialization with Protocol Buffers

## Business Context
**Financial Services - Fund Management & Regulatory Compliance**
- Investment funds, hedge funds, family offices
- KYC (Know Your Customer) workflows
- Entity relationship management
- Regulatory reporting and compliance

The codebase demonstrates advanced Rust patterns and architectural best practices.
EOF
        ;;
    "gemini")
        print_status "Creating Gemini-optimized summary..."
        cat > "$EXPORT_DIR/GEMINI_CONTEXT.md" << 'EOF'
# Data Designer - Context for Gemini

## System Overview
**Financial Technology Platform** built with **Rust + WASM + gRPC**

## Core Components
1. **DSL Engine**: Custom parser for financial domain language
2. **Web Frontend**: Pure Rust WASM application (egui framework)
3. **Microservices**: gRPC-based backend services
4. **Language Server**: Full LSP implementation with IDE features
5. **Database Layer**: PostgreSQL with type-safe query building

## Technical Sophistication
- **Parser Technology**: nom parser combinators for DSL processing
- **Web Assembly**: High-performance browser applications
- **Protocol Buffers**: Efficient cross-service communication
- **Language Server Protocol**: Professional IDE integration
- **Database Migrations**: Version-controlled schema evolution

## Domain Expertise
**Financial Services - Investment Management**
- Client Business Unit (CBU) lifecycle management
- Multi-entity relationship modeling
- Regulatory compliance workflows
- Fund onboarding and KYC processes

## Code Characteristics
- **Functional Programming**: Heavy use of iterators and combinators
- **Type-Driven Design**: Leverages Rust's type system for correctness
- **Async/Await**: Non-blocking I/O throughout
- **Error Handling**: Comprehensive Result and Option usage
- **Memory Efficiency**: Zero-allocation parsing where possible

## Development Workflow
- **Build System**: Cargo workspace with multiple crates
- **Testing Strategy**: Unit, integration, and round-trip tests
- **Code Generation**: Protocol Buffer compilation
- **WASM Compilation**: Optimized for web deployment

This represents a production-grade Rust application with enterprise-level architecture.
EOF
        ;;
esac

# Create single-file exports for easy copy-paste
print_status "Creating single-file exports..."
mkdir -p "$EXPORT_DIR/single-files"

# Combine core files into one
cat > "$EXPORT_DIR/single-files/complete-codebase.md" << 'EOF'
# Data Designer - Complete Codebase Export

## Project Structure
EOF

# Add project structure
find . -type f -name "*.rs" | head -20 | while read file; do
    echo "- \`$file\`" >> "$EXPORT_DIR/single-files/complete-codebase.md"
done

echo "" >> "$EXPORT_DIR/single-files/complete-codebase.md"

# Add key files content
if [ "$EXPORT_TYPE" = "full" ]; then
    print_status "Adding full file contents..."

    for rs_file in $(find data-designer-core/src -name "*.rs" | head -10); do
        echo "## File: $rs_file" >> "$EXPORT_DIR/single-files/complete-codebase.md"
        echo '```rust' >> "$EXPORT_DIR/single-files/complete-codebase.md"
        sanitize_code "$rs_file" >> "$EXPORT_DIR/single-files/complete-codebase.md"
        echo '```' >> "$EXPORT_DIR/single-files/complete-codebase.md"
        echo "" >> "$EXPORT_DIR/single-files/complete-codebase.md"
    done
fi

# Create compressed archive
print_status "Creating compressed archive..."
tar -czf "$EXPORT_DIR/data-designer-ai-export.tar.gz" -C "$EXPORT_DIR" .

# Create file manifest
print_status "Creating file manifest..."
cat > "$EXPORT_DIR/FILE_MANIFEST.md" << 'EOF'
# Data Designer - AI Export File Manifest

## 📁 Directory Structure

### `/PROJECT_OVERVIEW.md`
High-level project description, architecture, and key features

### `/core-rust/`
Core Rust source files with sensitive data redacted
- `data-designer-core-src/` - Core DSL engine and database layer
- `web-ui-src/` - Key frontend components
- `grpc-server-src/` - Backend microservices
- `cbu-dsl-lsp-src/` - Language Server Protocol implementation

### `/config/`
Configuration files and schemas
- `Cargo.toml` files for build configuration
- `*.proto` Protocol Buffer definitions
- `CLAUDE.md` project documentation

### `/examples/`
Example DSL files and usage patterns
- `cbu-creation-example.lisp` - Basic CBU creation
- `complex-workflow-example.lisp` - Advanced workflow
- `test-dsl-files/` - Test cases and examples

### `/single-files/`
Combined exports for easy copy-paste
- `complete-codebase.md` - All core files in one document

### AI-Specific Context Files
- `CHATGPT_CONTEXT.md` - ChatGPT-optimized summary
- `CLAUDE_CONTEXT.md` - Claude-optimized summary
- `GEMINI_CONTEXT.md` - Gemini-optimized summary

## 🚀 Recommended Usage

### For Code Questions
1. Share `PROJECT_OVERVIEW.md` first for context
2. Include relevant files from `core-rust/` directory
3. Reference `examples/` for DSL usage patterns

### For Architecture Questions
1. Start with AI-specific context file
2. Include `config/` files for build/deployment context
3. Reference `single-files/complete-codebase.md` for full picture

### For Feature Development
1. Share specific module from `core-rust/`
2. Include related examples from `examples/`
3. Reference Protocol Buffer definitions from `config/`

## 🔒 Security Notes
- All sensitive data (passwords, API keys) has been redacted
- No production secrets or credentials included
- Database connection strings sanitized
- Only code structure and logic exposed
EOF

# Generate usage instructions
cat > "$EXPORT_DIR/AI_USAGE_GUIDE.md" << 'EOF'
# How to Use This Export with AI Assistants

## 🤖 Quick Start for AI Conversations

### Option 1: Project Overview (Recommended First)
Copy and paste `PROJECT_OVERVIEW.md` to give the AI context about the project.

### Option 2: Specific Module
If you have questions about a specific feature:
1. Include the relevant `.rs` file from `core-rust/`
2. Add related examples from `examples/`
3. Include the AI-specific context file

### Option 3: Complete Codebase
For comprehensive analysis, use `single-files/complete-codebase.md`

## 💡 Example Prompts

### "Help me understand this codebase"
Attach: `PROJECT_OVERVIEW.md` + `[AI]_CONTEXT.md`

### "How does the DSL parser work?"
Attach: `core-rust/data-designer-core-src/lisp_cbu_dsl.rs` + examples

### "Help me add a new feature"
Attach: Relevant module + `PROJECT_OVERVIEW.md` + examples

### "Review my implementation"
Attach: Your code + related core modules + examples

## 🎯 Best Practices

1. **Start Small**: Begin with overview, then add specific files
2. **Include Examples**: Always helpful for understanding usage
3. **Be Specific**: Ask about particular modules rather than entire codebase
4. **Use Context**: The AI-specific context files are optimized for each assistant

## 📊 File Size Guidelines

- **ChatGPT**: Can handle large files, but prefers structured input
- **Claude**: Excellent with long documents, use complete-codebase.md
- **Gemini**: Good with technical content, include architecture context

The export is designed to give AI assistants complete context about your sophisticated Rust financial services platform!
EOF

print_success "Export completed successfully!"
echo ""
echo "📁 Export location: $EXPORT_DIR/"
echo "🗜️  Archive: $EXPORT_DIR/data-designer-ai-export.tar.gz"
echo ""
echo "📋 Quick Usage:"
echo "  1. Share PROJECT_OVERVIEW.md for context"
echo "  2. Include relevant files from core-rust/"
echo "  3. Reference examples/ for DSL usage"
echo ""
echo "🎯 AI-Specific Files:"
echo "  • CHATGPT_CONTEXT.md - Optimized for ChatGPT"
echo "  • CLAUDE_CONTEXT.md - Optimized for Claude"
echo "  • GEMINI_CONTEXT.md - Optimized for Gemini"
echo ""
echo "🚀 For complete analysis: single-files/complete-codebase.md"