# CLAUDE.md

## Project Overview

Pure desktop Tauri application for designing, testing, and managing dynamic data transformation rules using a soft DSL system.

### Key Features
- **Dynamic Grammar System**: EBNF-based soft DSL editable through UI
- **Advanced Parser**: nom-based parser with 6 extensions (arithmetic, strings, functions, lookups, runtime resolution, regex)
- **Interactive Rule Editor**: Monaco Editor with live testing
- **Configuration-Driven UI**: Multi-layered Resource Dictionary with perspective switching
- **PostgreSQL Integration**: Centralized database operations with vector embeddings

### Architecture
- **Frontend**: TypeScript modules with Monaco Editor (src/main.ts, src/ui-components.ts)
- **Backend**: Rust Tauri with centralized database layer (src-tauri/src/db/)
- **Core Library**: Enhanced expression engine (data-designer-core/)
- **Database**: PostgreSQL with pgvector for semantic similarity

### Development Commands
```bash
npm run build        # Build frontend
cd src-tauri
cargo tauri dev      # Run desktop app
cargo tauri build    # Build for production
```

### Key Files
- `src-tauri/src/lib.rs`: Tauri commands and rule testing
- `src-tauri/src/db/mod.rs`: Centralized database operations
- `src-tauri/src/db/persistence.rs`: Live data connection layer
- `src/main.ts`: Main application logic
- `src/config-driven-renderer.ts`: Dynamic UI renderer
- `grammar_rules.json`: Dynamic grammar storage

### Current Features
- ✅ PostgreSQL-backed data dictionary with 94+ attributes
- ✅ Live rule testing with context data
- ✅ AST visualization and export
- ✅ Dynamic grammar editing
- ✅ Configuration-driven UI system
- ✅ Live Data Connection layer with PersistenceService trait
- ✅ Vector similarity search for rules
- ✅ Database-driven workflow (no static JSON)
- ✅ All compilation errors fixed - application running successfully
- ✅ Persistence services initialized with PostgreSQL and Redis support
- ✅ Centralized database operations through unified DbOperations layer

### Database Schema
PostgreSQL database: `data_designer` with rules, attributes, embeddings, and business entity tables.

### Test Rules Available
15+ test cases including arithmetic, string operations, regex validation, KYC functions, and complex mixed operations.