# CLAUDE.md

## Project Overview

🦀 **Hybrid Desktop/Web Data Designer** - Pure Rust desktop application with gRPC microservices architecture for designing, testing, and managing dynamic data transformation rules using a soft DSL system with comprehensive AI assistance.

### Key Features
- **Hybrid Architecture** - gRPC microservices with egui desktop client and future web support
- **Native egui GUI** - immediate mode, 60fps, dark theme with enhanced font rendering
- **gRPC Communication** - Type-safe Protocol Buffers with automatic fallback
- **Secure Key Management** - System keychain integration with gRPC API
- **Dynamic Grammar System** - EBNF-based soft DSL editable through UI
- **Advanced Parser** - nom-based parser with 6 extensions (arithmetic, strings, functions, lookups, runtime resolution, regex)
- **PostgreSQL Integration** - centralized database operations with vector embeddings
- **Configuration-Driven UI** - multi-layered Resource Dictionary with perspective switching
- **Complete AI Assistant System** - 7 AI features with gRPC-based API key management
- **RAG Integration** - Retrieval-Augmented Generation with database similarity search
- **Enhanced Code Editor** - Professional font rendering with syntax highlighting

### Architecture
- **Frontend**: Pure Rust egui immediate mode GUI (`egui-frontend/`)
- **gRPC Server**: Financial taxonomy service with Protocol Buffers (`egui-frontend/grpc-server/`)
- **Shared Components**: Common UI library with gRPC client (`egui-frontend/shared-components/`)
- **Core Library**: Expression engine with database layer (`data-designer-core/`)
- **Database**: PostgreSQL with pgvector for semantic similarity
- **Communication**: gRPC-first (port 50051) with database fallback (hybrid reliability)
- **Key Management**: System keychain with security command fallback
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
- ✅ **Secure Key Management** - System keychain integration with gRPC API
- ✅ Native egui desktop application with enhanced font rendering
- ✅ Clean Cargo workspace structure
- ✅ PostgreSQL database integration with full CRUD operations
- ✅ Configuration management with environment variable support
- ✅ Advanced parser engine with 6 extensions (fully tested)
- ✅ Live data connection layer (PersistenceService trait)
- ✅ Vector similarity search with pgvector integration
- ✅ **COMPLETE AI ASSISTANT SYSTEM** - All 7 features implemented with gRPC integration:
  - ✅ AI assistant architecture for DSL help (multi-provider via gRPC)
  - ✅ AI suggestion UI in transpiler tab (gRPC-based)
  - ✅ Context-aware prompt engineering
  - ✅ Semantic search for similar rules/patterns
  - ✅ Intelligent code completion suggestions
  - ✅ AI-powered error explanations and fixes
  - ✅ RAG with database for contextual help
- ✅ **Keychain Integration** - Secure API key storage and retrieval via gRPC
- ✅ **Security Command Fallback** - Cross-platform keychain access with macOS security command
- ✅ Enhanced code editor with 16pt monospace font
- ✅ Professional transpiler interface with clean rendering
- ✅ Rule testing and execution interface
- ✅ Comprehensive database management
- ✅ Both Tauri and Pure Rust versions fully operational
- ✅ **Investment Mandate Drill-Down System** - Interactive mandate exploration with detailed views
- ✅ **Code Quality** - Cargo clippy integration with 40+ automated fixes applied

### AI Features Status
**🎯 COMPLETE: All 7 AI features successfully implemented and tested with gRPC integration**
1. **AI Assistant Architecture** - Multi-provider system (OpenAI, Anthropic, Offline) with gRPC API key management
2. **AI Suggestion UI** - Interactive transpiler tab with real-time suggestions via gRPC
3. **Context-Aware Prompting** - Smart context building from current DSL state
4. **Semantic Search** - Database-backed similar rule discovery
5. **Code Completion** - Intelligent function/attribute/operator suggestions
6. **Error Analysis** - Comprehensive error detection and automatic fixing
7. **RAG Integration** - Retrieval-Augmented Generation with vector similarity

### Security & Key Management
- **🔐 System Keychain Integration** - Secure storage of API keys using platform keyring
- **🔑 gRPC Key Management API** - Store, retrieve, delete, and list API keys via gRPC
- **🛡️ Security Command Fallback** - macOS security command integration for robust key access
- **🔒 Cross-Platform Support** - Windows Credential Manager, macOS Keychain, Linux Secret Service
- **⚡ Automatic Key Loading** - AI assistant automatically loads keys on gRPC client setup

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
- **Test Coverage**: 20+ comprehensive tests including gRPC integration
- **Parser Tests**: Expressions, functions, conditionals, arithmetic
- **Database Tests**: Models, attributes, data dictionary integration
- **UI Tests**: Syntax highlighting, component state management
- **Integration Tests**: Complete rule evaluation and AST processing
- **gRPC Integration Tests**: 16 comprehensive tests covering:
  - Health checks and server connectivity
  - Financial taxonomy data retrieval (products, services, CBU structures)
  - AI suggestions and provider integration
  - Keychain integration and API key management
  - Error handling and graceful degradation
  - Pagination and concurrent request handling
  - Performance benchmarking and end-to-end data flow
- **Code Quality**: Cargo clippy integration with automated fixes

### Performance
- **Build time**: Sub-second with cargo
- **Runtime**: Native performance, 60fps GUI
- **Memory**: Minimal Rust overhead
- **Distribution**: Single native binary
- **Testing**: Much superior to Tauri - full testability achieved