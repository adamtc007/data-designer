# Data Designer - AI Assistant Context

## Quick Project Overview
🦀 **Web-First Data Designer** - Pure Rust WASM application for financial DSL development with comprehensive AI assistance.

### Architecture
- **Web UI**: WASM client with egui (`web-ui/`)
- **gRPC Server**: Financial services with Protocol Buffers (`grpc-server/`)
- **Core Library**: DSL engine and database (`data-designer-core/`)
- **Database**: PostgreSQL with pgvector

### Key Features
- **S-Expression DSL**: LISP-style syntax for financial workflows
- **Language Server Protocol**: Full LSP with syntax highlighting, completion, diagnostics
- **Microservices**: gRPC-first with HTTP fallback
- **Entity Management**: Complete CRUD for CBU, Products, Services, Resources
- **AI Integration**: 7 AI features with semantic search and RAG

## Quick Start Commands
```bash
# Start everything
./runwasm.sh --with-lsp

# Start just the LSP server
./run-lsp-server.sh

# Export codebase for AI
./export-for-ai.sh

# Build and test
cargo build --all
cargo test --all
```

## Current Context
The system is **production-ready** with:
- Complete LSP implementation with professional IDE features
- Full AI assistant integration (OpenAI, Anthropic, Offline)
- Comprehensive test coverage (20+ tests)
- Clean microservice architecture
- S-expression DSL with round-trip testing

## Recent Work
- ✅ S-expression DSL implementation
- ✅ Full Language Server Protocol with syntax highlighting
- ✅ AI codebase export utility
- 🔄 Zed editor integration setup (current)

## File Structure Summary
```
data-designer/
├── web-ui/                 # WASM web application
├── grpc-server/           # gRPC microservices
├── data-designer-core/    # Core engine and database
├── proto/                 # Protocol Buffer definitions
├── migrations/            # Database schema
├── runwasm.sh            # One-command deployment
├── export-for-ai.sh      # AI assistant integration
└── .zed/                 # Zed editor configuration
```

## API Key Setup
The system uses secure keychain integration. To set up AI providers:

1. **OpenAI**: Store as `OPENAI_API_KEY` in system keychain
2. **Anthropic**: Store as `ANTHROPIC_API_KEY` in system keychain
3. **Access via gRPC**: Use `StoreApiKey` and `GetApiKey` endpoints

## DSL Examples
```lisp
;; Create a CBU with entities
(create-cbu "Investment Fund" "Multi-strategy operations"
  (entities
    (entity "GS001" "Goldman Sachs" asset-owner)
    (entity "BNY001" "BNY Mellon" custodian)))

;; Update CBU metadata
(update-cbu "CBU001"
  (add-entities
    (entity "NEW001" "Prime Broker" prime-broker))
  (update-metadata
    (aum 1500000000)
    (status "active")))
```

## Available Assistance
- **Code completion**: Context-aware DSL suggestions
- **Error diagnostics**: Real-time syntax validation
- **Semantic search**: Find similar patterns in database
- **AI explanations**: Detailed error analysis and fixes
- **RAG integration**: Contextual help from knowledge base

The system is designed for **financial services** with focus on fund accounting, entity management, and regulatory compliance workflows.