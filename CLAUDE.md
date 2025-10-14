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
- **Core Library**: Expression engine with database layer (`data-designer-core/`)
- **Database**: PostgreSQL with pgvector for semantic similarity
- **Build System**: Clean Cargo workspace

### Development Commands
```bash
cargo build                    # Build entire workspace
cargo run --release           # Run optimized desktop app
cd egui-frontend && cargo run # Run from frontend directory
cargo test --all              # Run comprehensive test suite (16+ tests)
```

### Key Files
- `egui-frontend/src/main.rs` - Main egui application
- `data-designer-core/src/db/mod.rs` - Database operations
- `data-designer-core/src/db/persistence.rs` - Data connection layer
- `data-designer-core/src/config.rs` - Configuration management
- `Cargo.toml` - Workspace configuration

### Current Features - COMPLETED SYSTEM
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

### AI Features Status
**🎯 COMPLETE: All 7 AI features successfully implemented and tested**
1. **AI Assistant Architecture** - Multi-provider system (OpenAI, Anthropic, Offline)
2. **AI Suggestion UI** - Interactive transpiler tab with real-time suggestions
3. **Context-Aware Prompting** - Smart context building from current DSL state
4. **Semantic Search** - Database-backed similar rule discovery
5. **Code Completion** - Intelligent function/attribute/operator suggestions
6. **Error Analysis** - Comprehensive error detection and automatic fixing
7. **RAG Integration** - Retrieval-Augmented Generation with vector similarity

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