# CLAUDE.md

## Project Overview

Pure desktop Rust application using egui for designing, testing, and managing dynamic data transformation rules using a soft DSL system.

### Key Features
- **Dynamic Grammar System**: EBNF-based soft DSL editable through UI
- **Advanced Parser**: nom-based parser with 6 extensions (arithmetic, strings, functions, lookups, runtime resolution, regex)
- **Native GUI**: egui immediate mode GUI with CRUD operations
- **Configuration-Driven UI**: Multi-layered Resource Dictionary with perspective switching
- **PostgreSQL Integration**: Centralized database operations with vector embeddings

### Architecture
- **Frontend**: Rust egui immediate mode GUI (data-designer/egui-frontend/)
- **Core Library**: Enhanced expression engine with database layer (data-designer/data-designer-core/)
- **Database**: PostgreSQL with pgvector for semantic similarity
- **Workspace Structure**: Cargo workspace with shared library architecture

### Development Commands
```bash
npm run dev          # Run egui desktop app
npm run build        # Build for production
cd data-designer/egui-frontend
cargo run            # Run directly with cargo
cargo run --release  # Run optimized build
```

### Key Files
- `data-designer/egui-frontend/src/main.rs`: Main egui application with CRUD UI
- `data-designer/data-designer-core/src/db/mod.rs`: Centralized database operations
- `data-designer/data-designer-core/src/db/persistence.rs`: Live data connection layer
- `data-designer/data-designer-core/src/config.rs`: Configuration management
- `data-designer/Cargo.toml`: Workspace configuration

### Current Features
- ✅ Native egui CRUD interface for CBUs, Products, and Resources
- ✅ PostgreSQL-backed data dictionary with 94+ attributes
- ✅ Rust workspace structure with shared core library
- ✅ Configuration-driven UI system
- ✅ Live Data Connection layer with PersistenceService trait
- ✅ Vector similarity search for rules
- ✅ Database-driven workflow (no static JSON)
- ✅ All compilation errors fixed - application running successfully
- ✅ Persistence services initialized with PostgreSQL and Redis support
- ✅ Centralized database operations through unified DbOperations layer
- ✅ Tauri dependencies removed - pure Rust implementation

### Database Schema
PostgreSQL database: `data_designer` with rules, attributes, embeddings, and business entity tables.

### Test Rules Available
15+ test cases including arithmetic, string operations, regex validation, KYC functions, and complex mixed operations.