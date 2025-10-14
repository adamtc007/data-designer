# CLAUDE.md

## Project Overview

🦀 **Pure Rust Data Designer** - Native desktop application for designing, testing, and managing dynamic data transformation rules using a soft DSL system.

### Key Features
- **Native egui GUI** - immediate mode, 60fps, dark theme
- **Dynamic Grammar System** - EBNF-based soft DSL editable through UI
- **Advanced Parser** - nom-based parser with 6 extensions (arithmetic, strings, functions, lookups, runtime resolution, regex)
- **PostgreSQL Integration** - centralized database operations with vector embeddings
- **Configuration-Driven UI** - multi-layered Resource Dictionary with perspective switching

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
```

### Key Files
- `egui-frontend/src/main.rs` - Main egui application
- `data-designer-core/src/db/mod.rs` - Database operations
- `data-designer-core/src/db/persistence.rs` - Data connection layer
- `data-designer-core/src/config.rs` - Configuration management
- `Cargo.toml` - Workspace configuration

### Current Features
- ✅ Native egui desktop application with dashboard
- ✅ Clean Cargo workspace structure
- ✅ PostgreSQL database layer ready for integration
- ✅ Configuration management system
- ✅ Advanced parser engine with 6 extensions
- ✅ Live data connection layer (PersistenceService trait)
- ✅ Vector similarity search capabilities

### Next Steps
- Connect egui app to database
- Implement full CRUD operations
- Add rule editor interface
- Integrate parser/engine into GUI

### Database Schema
PostgreSQL database: `data_designer` with rules, attributes, embeddings, and business entity tables.

### Performance
- **Build time**: Sub-second with cargo
- **Runtime**: Native performance, 60fps GUI
- **Memory**: Minimal Rust overhead
- **Distribution**: Single native binary