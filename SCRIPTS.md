# Data Designer Shell Scripts Reference

Complete documentation of all build, start, run, and utility scripts in the Data Designer project.

## 🚀 Quick Start Scripts

### `runwasm.sh` - One-Command WASM Deployment
**Location:** Project root
**Purpose:** Complete WASM web application deployment with Docker management
**Usage:** `./runwasm.sh`

**Features:**
- Automatically checks and starts Docker Desktop (macOS)
- Kills existing processes on ports 8080 and 3030
- Starts template API server (port 3030)
- Builds WASM package via `build-web.sh`
- Serves web application via `serve-web.sh`
- Opens browser automatically
- Graceful shutdown with Ctrl+C

**Dependencies:** Docker, cargo, miniserve, curl

## 🏗️ Build Scripts

### `web-ui/build-web.sh` - WASM Package Builder
**Location:** `web-ui/`
**Purpose:** Build Rust WASM web application package
**Usage:** `cd web-ui && ./build-web.sh`

**Features:**
- Installs wasm-pack if missing
- Cleans previous builds (`dist/`, `pkg/`)
- Builds WASM package with `wasm-pack`
- Creates optimized `index.html` with loading UI
- Professional styling with spinner and dark theme

**Output:** `dist/` directory with WASM files and assets

### `dsl-lsp/build.sh` - Language Server Builder
**Location:** `dsl-lsp/`
**Purpose:** Build DSL Language Server for IDE integration
**Usage:** `cd dsl-lsp && ./build.sh`

**Features:**
- Builds release binary with `cargo build --release`
- Provides setup instructions for VS Code integration
- Configures Monaco Editor integration

**Output:** `target/release/dsl-lsp-server` binary

## 🌐 Server Scripts

### `web-ui/serve-web.sh` - Web Server
**Location:** `web-ui/`
**Purpose:** Serve WASM web application with miniserve
**Usage:** `cd web-ui && ./serve-web.sh`

**Features:**
- Installs miniserve if missing
- Auto-builds if `dist/` missing
- Serves on localhost:8080
- CORS and security headers configured
- Cache control headers for development

**Requirements:** miniserve, built WASM package

## 🧪 Test Scripts

### `quick_test.sh` - IDE Functionality Test
**Location:** Project root
**Purpose:** Test Data Designer IDE functionality and components
**Usage:** `./quick_test.sh`

**Tests:**
- Server connectivity (port 1420)
- Test Data tab functionality
- Source/target data files
- JavaScript test functions
- Rust backend commands

### `test-db-integration.sh` - Database Integration Test
**Location:** Project root
**Purpose:** Test PostgreSQL and pgvector integration
**Usage:** `./test-db-integration.sh`

**Tests:**
- Database connection (user: adamtc007, db: data_designer)
- pgvector extension status
- Rule, attribute, and derived attribute counts
- Embedding column accessibility
- Vector operations functionality

### `test-lsp.sh` - Language Server Test
**Location:** Project root
**Purpose:** Test DSL Language Server functionality
**Usage:** `./test-lsp.sh`

**Features:**
- Builds LSP server if missing
- Starts server on port 3030
- Monitors server process and port listening
- Provides connection instructions for IDE

### `test-upgrade-verification.sh` - Database Upgrade Verification
**Location:** Project root
**Purpose:** Verify PostgreSQL 17 and pgvector 0.8.1 upgrade
**Usage:** `./test-upgrade-verification.sh`

**Tests:**
- PostgreSQL version verification (14.19 → 17.6)
- pgvector version verification (0.8.0 → 0.8.1)
- Data integrity post-upgrade
- Vector operations and HNSW index status

### `test-new-ide-features.sh` - New IDE Feature Test
**Location:** Project root
**Purpose:** Test new IDE features after upgrades
**Usage:** `./test-new-ide-features.sh`

**Tests:**
- Tauri app running status (port 1420)
- New UI components (Find Similar, Rules Catalogue, AI Agent)
- Database integration with rule counts
- Feature summary and usage instructions

## 🔧 Setup Scripts

### `setup.sh` - Project Setup
**Location:** Project root
**Purpose:** Create complete Data Designer MVP project structure
**Usage:** `./setup.sh` (in empty project directory)

**Creates:**
- Cargo.toml with dependencies
- DSL grammar file (pest)
- Core library with models, parser, engine
- Test harness in main.rs
- Complete project scaffold

### `database/setup.sh` - Database Setup
**Location:** `database/`
**Purpose:** Initialize PostgreSQL database for Data Designer
**Usage:** `cd database && ./setup.sh`

**Features:**
- Creates `data_designer` database
- Applies schema from `database/schema.sql`
- Handles user permissions and fallbacks
- Verification instructions

### `ast-setup.sh` - AST Setup
**Location:** Project root
**Purpose:** Create complete data-designer Rust workspace
**Usage:** `./ast-setup.sh`

**Creates:**
- Workspace with 3 members: `data-designer-core`, `data-designer-cli`, `data-designer-lsp`
- Core library with models, parser, evaluator, engine
- CLI with test-rule command
- LSP server foundation
- Complete dependency management

## 🔍 Service Management Scripts

### `elasticsearch.sh` - Elasticsearch Management
**Location:** Project root
**Purpose:** Complete Elasticsearch and Kibana management
**Usage:** `./elasticsearch.sh [command]`

**Commands:**
- `start` - Start containers with Docker check
- `stop` - Stop containers
- `restart` - Restart services
- `status` - Show container and health status
- `logs` - Show recent logs (last 20 lines)
- `health` - Detailed cluster health check
- `clean` - Remove containers and volumes (destructive)
- `help` - Show usage information

**Features:**
- Automatic Docker Desktop startup (macOS)
- Health monitoring for Elasticsearch (port 9200) and Kibana (port 5601)
- JSON parsing for cluster stats
- Interactive confirmation for destructive operations

## 📊 Development Commands Summary

```bash
# Quick Start - Complete WASM App
./runwasm.sh                   # One command: build + serve + open browser

# Manual Development
cd grpc-server && cargo run    # Start gRPC server (port 50051)
cd web-ui && ./build-web.sh    # Build WASM package
cd web-ui && ./serve-web.sh    # Serve on localhost:8080

# Database Setup
cd database && ./setup.sh     # Initialize PostgreSQL database

# Testing
./quick_test.sh               # Test IDE functionality
./test-db-integration.sh      # Test database integration
./test-lsp.sh                 # Test language server

# Services
./elasticsearch.sh start      # Start Elasticsearch/Kibana
./elasticsearch.sh status     # Check service health

# Build Tools
cd dsl-lsp && ./build.sh      # Build language server
```

## 🧹 Log Management Scripts

### `elasticsearch-autopurge.sh` - Automatic Log Cleanup
**Location:** Project root
**Purpose:** Automatically remove test logs older than 7 days
**Usage:** `./elasticsearch-autopurge.sh [OPTIONS]`

**Features:**
- Configurable retention period (default: 7 days)
- Dry run mode for testing
- Index-level or document-level cleanup
- Performance optimization after cleanup
- Detailed logging and reporting
- Safety checks and error handling

**Examples:**
```bash
# Dry run to see what would be deleted
./elasticsearch-autopurge.sh --dry-run

# Delete logs older than 14 days
./elasticsearch-autopurge.sh --retention-days 14

# Delete only documents, keep indices
./elasticsearch-autopurge.sh --documents-only
```

### `elasticsearch-autopurge-cron.sh` - Cron Job Management
**Location:** Project root
**Purpose:** Set up automated scheduled log cleanup
**Usage:** `./elasticsearch-autopurge-cron.sh [COMMAND]`

**Commands:**
- `setup` - Configure automatic cleanup schedule
- `remove` - Remove scheduled cleanup
- `status` - Show current configuration and logs
- `test` - Test cleanup script (dry run)

**Features:**
- Interactive schedule selection
- Pre-configured schedules (daily, weekly)
- Status monitoring and log viewing
- Easy setup and removal

## 🔗 Dependencies

**Required Tools:**
- Rust/Cargo
- Docker Desktop (for runwasm.sh, elasticsearch.sh)
- PostgreSQL 17+ with pgvector
- wasm-pack
- miniserve
- curl, lsof, psql

**Optional Tools:**
- jq (for JSON parsing in elasticsearch.sh, autopurge scripts)
- cron (for automated log cleanup)
- VS Code (for LSP integration)

## 📁 Script Locations

```
data-designer/
├── runwasm.sh                    # Main deployment script
├── setup.sh                     # Project scaffolding
├── elasticsearch.sh              # Service management
├── quick_test.sh                 # IDE testing
├── test-db-integration.sh        # Database testing
├── test-lsp.sh                   # LSP testing
├── test-upgrade-verification.sh  # Upgrade verification
├── test-new-ide-features.sh      # Feature testing
├── ast-setup.sh                  # Workspace creation
├── database/
│   └── setup.sh                  # Database initialization
├── dsl-lsp/
│   └── build.sh                  # LSP server build
└── web-ui/
    ├── build-web.sh              # WASM build
    └── serve-web.sh              # Web server
```

All scripts include error handling, status reporting, and user-friendly output with emoji indicators for better developer experience.