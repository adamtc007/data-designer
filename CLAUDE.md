# CLAUDE.md

## Project Overview

🦀 **Data Designer** - Pure Rust WASM web application with gRPC microservices architecture for designing, testing, and managing dynamic data transformation rules using a soft DSL system for financial services.

### Architecture
- **Web UI**: Pure Rust WASM client with egui (`web-ui/`)
- **gRPC Server**: Financial taxonomy service with Protocol Buffers (`grpc-server/`)
- **Core Library**: Expression engine with database layer (`data-designer-core/`)
- **Database**: PostgreSQL with pgvector for semantic similarity
- **Communication**: gRPC-first (port 50051) with HTTP fallback (port 8080)
- **Build System**: Clean Cargo workspace with WASM support

### Quick Start
```bash
# One command: build + serve + open browser
./runwasm.sh

# Development
cargo build                   # Build entire workspace
cargo test --all             # Run comprehensive test suite
```

### Key Features
- **CBU DSL Management** - Client Business Unit DSL editor and execution (current focus)
- **gRPC Communication** - Type-safe Protocol Buffers with automatic fallback
- **Browser-Native GUI** - egui + WASM, 60fps, dark theme
- **PostgreSQL Integration** - Centralized database operations
- **Advanced Parser** - nom-based parser with 6 extensions

### Key Files
- `web-ui/src/lib.rs` - WASM web application entry point
- `web-ui/src/app.rs` - Main egui application logic (CBU-only after pruning)
- `web-ui/src/cbu_dsl_ide.rs` - CBU DSL IDE with picker functionality
- `grpc-server/src/main.rs` - gRPC server with Protocol Buffers
- `proto/financial_taxonomy.proto` - Complete gRPC API definitions
- `data-designer-core/` - Core expression engine and database layer
- `runwasm.sh` - One-command WASM deployment script

### Database Schema
PostgreSQL database: `data_designer` with CBU records, DSL metadata, and comprehensive entity tables.

### Recent Updates
- **egui Window Fixes** - Fixed entity picker window resizing issues with proper ScrollArea patterns
- **UI Pruning** - Simplified to CBU-only functionality for focused development
- **CBU Picker Fixes** - Fixed refresh and context selection issues
- **Code Quality** - Cargo clippy integration with reduced warnings
- **Build Optimization** - Streamlined web-first architecture

### egui Best Practices Learned
- **Window Resizing**: Avoid `.default_size()` in render loops (resets user sizing every frame)
- **ScrollArea Pattern**: Use `auto_shrink([false, false])` + `max_height()` for proper content control
- **Layout Structure**: Fixed header/footer outside ScrollArea, variable content inside
- **Content Control**: Control content size inside window, not window size itself

### Testing & Quality
- **Test Coverage**: 20+ comprehensive tests including gRPC integration
- **Parser Tests**: Expressions, functions, conditionals, arithmetic
- **Database Tests**: Models, attributes, data dictionary integration
- **gRPC Integration Tests**: Health checks, entity management, CBU operations
- **Code Quality**: Cargo clippy integration with minimal warnings

### Performance
- **Build time**: Sub-second with cargo
- **Runtime**: Native performance, 60fps GUI
- **Memory**: Minimal Rust overhead
- **Distribution**: Single native binary + WASM web bundle
- **Database**: Optimized PostgreSQL with indexes for CRUD operations
- **gRPC**: High-performance Protocol Buffers with type safety