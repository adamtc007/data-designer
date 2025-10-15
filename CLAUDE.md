# CLAUDE.md

## Project Overview

🦀 **Pure Rust Data Designer** - Native desktop application for designing, testing, and managing dynamic data transformation rules using a soft DSL system with comprehensive AI assistance.

### Key Features
- **Native egui GUI** - immediate mode, 60fps, dark theme with enhanced font rendering
- **Dynamic Grammar System** - EBNF-based soft DSL editable through UI
- **Advanced Parser** - nom-based parser with 6 extensions (arithmetic, strings, functions, lookups, runtime resolution, regex)
- **PostgreSQL Integration** - centralized database operations with vector embeddings
- **Configuration-Driven UI** - multi-layered Resource Dictionary with perspective switching
- **Complete AI Assistant System** - 7 AI features with environment variable API key support
- **RAG Integration** - Retrieval-Augmented Generation with database similarity search
- **Enhanced Code Editor** - Professional font rendering with syntax highlighting

### Architecture
- **Frontend**: Pure Rust egui immediate mode GUI (`egui-frontend/`)
- **Backend**: gRPC server with Protocol Buffers (`egui-frontend/grpc-server/`)
- **Core Library**: Expression engine with database layer (`data-designer-core/`)
- **Database**: PostgreSQL with pgvector for semantic similarity
- **Communication**: gRPC-first with database fallback (hybrid reliability)
- **Build System**: Clean Cargo workspace

### Development Commands
```bash
# gRPC Server (run first)
cd egui-frontend/grpc-server && cargo run

# Desktop Client (gRPC-enabled)
cd egui-frontend && cargo run --release --bin data-designer-egui

# Development
cargo build                    # Build entire workspace
cargo test --all              # Run comprehensive test suite (16+ tests)
```

### Key Files
- `egui-frontend/src/main.rs` - Main egui application (gRPC-enabled)
- `egui-frontend/grpc-server/` - gRPC server with Protocol Buffers
- `egui-frontend/shared-components/` - Shared UI components with gRPC client
- `data-designer-core/src/db/mod.rs` - Database operations
- `data-designer-core/src/db/persistence.rs` - Data connection layer
- `data-designer-core/src/config.rs` - Configuration management
- `Cargo.toml` - Workspace configuration

### Current Features - COMPLETED SYSTEM
- ✅ **gRPC-Enabled Desktop Application** - Hybrid architecture with Protocol Buffers
- ✅ **Microservices Architecture** - gRPC server + egui client (port 50051)
- ✅ **Hybrid Reliability** - gRPC-first with automatic database fallback
- ✅ **Type-Safe Communication** - Protocol Buffers with zero-copy performance
- ✅ Native egui desktop application with enhanced font rendering
- ✅ Clean Cargo workspace structure
- ✅ PostgreSQL database integration with full CRUD operations
- ✅ Configuration management with environment variable support
- ✅ Advanced parser engine with 6 extensions (fully tested)
- ✅ Live data connection layer (PersistenceService trait)
- ✅ Vector similarity search with pgvector integration
- ✅ **COMPLETE AI ASSISTANT SYSTEM** - All 7 features implemented:
  - ✅ AI assistant architecture for DSL help
  - ✅ AI suggestion UI in transpiler tab
  - ✅ Context-aware prompt engineering
  - ✅ Semantic search for similar rules/patterns
  - ✅ Intelligent code completion suggestions
  - ✅ AI-powered error explanations and fixes
  - ✅ RAG with database for contextual help
- ✅ Environment variable API key detection (ANTHROPIC_API_KEY, OPENAI_API_KEY)
- ✅ Enhanced code editor with 16pt monospace font
- ✅ Professional transpiler interface with clean rendering
- ✅ Rule testing and execution interface
- ✅ Comprehensive database management
- ✅ Both Tauri and Pure Rust versions fully operational
- ✅ **Investment Mandate Drill-Down System** - Interactive mandate exploration with detailed views

### AI Features Status
**🎯 COMPLETE: All 7 AI features successfully implemented and tested**
1. **AI Assistant Architecture** - Multi-provider system (OpenAI, Anthropic, Offline)
2. **AI Suggestion UI** - Interactive transpiler tab with real-time suggestions
3. **Context-Aware Prompting** - Smart context building from current DSL state
4. **Semantic Search** - Database-backed similar rule discovery
5. **Code Completion** - Intelligent function/attribute/operator suggestions
6. **Error Analysis** - Comprehensive error detection and automatic fixing
7. **RAG Integration** - Retrieval-Augmented Generation with vector similarity

### Financial Services Features
- **🎯 Investment Mandate Management** - Complete drill-down system with:
  - Interactive mandate cards with "View Details" buttons
  - Comprehensive detailed views (business units, parties, investment details)
  - Related member roles and trading/settlement authorities
  - Back navigation and breadcrumb display
  - Robust error handling and crash prevention
- **📦 Product Taxonomy** - Complete hierarchical system for financial products
- **🏢 CBU Management** - Client Business Unit organization and member roles
- **💼 Interactive Editing** - Full CRUD operations with database persistence

### Database Schema
PostgreSQL database: `data_designer` with rules, attributes, embeddings, and business entity tables.

### Testing & Quality
- **Test Coverage**: 16+ comprehensive tests
- **Parser Tests**: Expressions, functions, conditionals, arithmetic
- **Database Tests**: Models, attributes, data dictionary integration
- **UI Tests**: Syntax highlighting, component state management
- **Integration Tests**: Complete rule evaluation and AST processing

### Performance
- **Build time**: Sub-second with cargo
- **Runtime**: Native performance, 60fps GUI
- **Memory**: Minimal Rust overhead
- **Distribution**: Single native binary
- **Testing**: Much superior to Tauri - full testability achieved