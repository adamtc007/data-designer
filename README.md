# Data Designer - Cross-Platform Financial Data DSL Platform

🦀 **Cross-Platform Data Designer** - Native Desktop + WASM web application with unified HTTP client and gRPC microservices architecture for designing, testing, and managing dynamic data transformation rules using a soft DSL system with comprehensive AI assistance.

## 🚀 Key Features

### Cross-Platform Architecture
- **Native Desktop** - Full debugging capabilities with IDE integration (`./rundesk.sh`)
- **Browser-Native GUI** - egui + WASM, 60fps, dark theme with enhanced font rendering (`./runwasm.sh`)
- **Unified HTTP Client** - Identical reqwest-based client for both platforms
- **Staged Loading** - Smart progressive data loading (8 CBUs on startup, 100 entities on-demand)
- **gRPC Microservices** - Type-safe Protocol Buffers with HTTP bridge (ports 50051/8080)
- **99.5% Code Sharing** - Minimal conditional compilation between platforms

### Financial Services Focus
- **Investment Mandate Management** - Complete drill-down system with interactive cards
- **Comprehensive CRUD API** - Complete entity management for CBU, Product, Service, Resource, Workflow
- **Fund Accounting DSL** - Capability-driven execution with "remote control" analogy
- **White Truffle Architecture** - Advanced execution engine with orchestration

### Advanced DSL System
- **Multiple DSL Domains** - CBU, Deal Record, KYC, Onboarding, Opportunity, and Orchestration DSLs
- **Dynamic Grammar System** - EBNF-based soft DSL editable through UI
- **Advanced Parser** - nom-based parser with 6 extensions (arithmetic, strings, functions, lookups, runtime resolution, regex)
- **Template Designer IDE** - Professional two-pane layout with syntax highlighting
- **Capability Execution Engine** - Trait-based architecture with built-in fund accounting capabilities

### AI-Powered Development
- **Complete AI Assistant System** - All 7 AI features implemented with gRPC integration
- **Secure Key Management** - System keychain integration with gRPC API
- **RAG Integration** - Retrieval-Augmented Generation with database similarity search
- **Context-Aware Prompting** - Smart context building from current DSL state

### Database & Performance
- **PostgreSQL Integration** - Centralized database operations with vector embeddings
- **Configuration-Driven UI** - Multi-layered Resource Dictionary with perspective switching
- **Real-Time Execution** - Sub-second capability execution with comprehensive logging
- **Hybrid Reliability** - gRPC-first with automatic database fallback

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│         Browser (Any Device)            │
│  ┌─────────────────────────────────────┐ │
│  │   Pure Rust WASM Web Client        │ │
│  │   • egui GUI (60fps)               │ │
│  │   • Enhanced Font Rendering        │ │
│  │   • Entity Management UI           │ │
│  │   • Capability Management          │ │
│  │   • Professional Code Editor       │ │
│  └─────────────┬───────────────────────┘ │
└─────────────────┼───────────────────────────┘
                  │ gRPC (Port 50051)
┌─────────────────▼───────────────────────────┐
│         gRPC Microservices Server           │
│  ┌─────────────────────────────────────────┐ │
│  │    Financial Taxonomy Service           │ │
│  │    • Protocol Buffers API (900+ lines) │ │
│  │    • CRUD Operations                    │ │
│  │    • AI Key Management                  │ │
│  │    • Workflow Orchestration            │ │
│  └─────────────┬───────────────────────────┘ │
└─────────────────┼───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│         Core Library & Database             │
│  ┌─────────────────┐ ┌─────────────────────┐ │
│  │ Capability      │ │ PostgreSQL Database │ │
│  │ Execution       │ │ • Vector Embeddings │ │
│  │ Engine          │ │ • Entity Storage    │ │
│  │ • 10+ Built-in  │ │ • DSL Templates     │ │
│  │   Capabilities  │ │ • Audit Logs        │ │
│  └─────────────────┘ └─────────────────────┘ │
└─────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+
- PostgreSQL 17 (for database persistence)
- pgvector 0.8.1+ (for vector embeddings)

### Database Setup
```bash
# macOS:
brew install postgresql@17

# Install pgvector extension
cd /tmp
git clone --branch v0.8.1 https://github.com/pgvector/pgvector.git
cd pgvector
PG_CONFIG=/opt/homebrew/opt/postgresql@17/bin/pg_config make
PG_CONFIG=/opt/homebrew/opt/postgresql@17/bin/pg_config make install

# Create database and run migrations
createdb data_designer
export DATABASE_URL="postgres://$(whoami)@localhost/data_designer"
```

### One-Command Deployment
```bash
# Clone and run in one step
git clone https://github.com/yourusername/data-designer.git
cd data-designer

# Option 1: Web Application (Recommended)
./runwasm.sh                   # Build + serve + open browser

# Option 2: Manual Steps
cd grpc-server && cargo run   # Start gRPC server (port 50051)
cd web-ui && ./build-web.sh   # Build WASM package
cd web-ui && ./serve-web.sh   # Serve on localhost:8080
```

### Access Points
- **Web UI**: http://localhost:8080
- **gRPC Server**: localhost:50051

## 🤖 Complete AI Assistant System

**🎯 All 7 AI features implemented with gRPC integration:**

### Core Features
- **AI Assistant Architecture** - Multi-provider system (OpenAI, Anthropic, Offline)
- **AI Suggestion UI** - Interactive suggestions via gRPC
- **Context-Aware Prompting** - Smart context building from DSL state
- **Semantic Search** - Database-backed similar rule discovery
- **Code Completion** - Intelligent function/attribute suggestions
- **Error Analysis** - Comprehensive error detection and fixing
- **RAG Integration** - Retrieval-Augmented Generation with vector similarity

### Secure Key Management
- **🔐 System Keychain Integration** - Platform keyring storage
- **🔑 gRPC Key Management API** - Store/retrieve/delete via gRPC
- **🛡️ Cross-Platform Support** - Windows, macOS, Linux
- **⚡ Automatic Key Loading** - AI assistant auto-loads on startup

## 🎯 Usage Examples

### Financial DSL Workflows
```rust
// Fund Accounting Capability Execution
CONFIGURE_SYSTEM("custody_platform", {
    "environment": "production",
    "region": "us-east-1"
})

ACTIVATE("account_setup", {
    "client_id": "HF-001",
    "product_type": "prime_brokerage"
})

WORKFLOW("onboarding_flow", {
    "approvals": ["compliance", "operations"],
    "sla_hours": 24
})
```

### Investment Mandate Management
- Interactive mandate cards with "View Details" buttons
- Comprehensive detailed views (business units, parties, investment details)
- Related member roles and trading/settlement authorities
- Back navigation and breadcrumb display

### Entity Management
- **🏢 CBU Management** - Client Business Unit organization
- **📦 Product Catalog** - Financial products with line of business
- **⚙️ Service Lifecycle** - Public services with billing models
- **🔧 Resource Templates** - Private implementations with capabilities
- **📋 Workflow Orchestration** - Dependencies and approvals

### Example DSL Rules

```dsl
# KYC Risk Assessment
risk_score = 0

IF Client.risk_rating == "HIGH" THEN
    risk_score = risk_score + 50
ELSE
    risk_score = risk_score + 10

# Email Validation
email_valid = IS_EMAIL(Client.email)

# Pattern Matching
lei_valid = Client.lei_code ~ /^[A-Z0-9]{20}$/

# Data Lookup
country_name = LOOKUP(Client.country_code, "countries")

# Result Formatting
result = CONCAT("Risk Score: ", risk_score, " - Status: ",
                IF risk_score > 50 THEN "Review Required" ELSE "Approved")
```

## 🔧 DSL Functions

### Validation Functions
- `IS_EMAIL(email)` - Validates email format
- `IS_LEI(lei)` - Validates Legal Entity Identifier
- `IS_SWIFT(code)` - Validates SWIFT/BIC code
- `IS_PHONE(number)` - Validates phone number
- `VALIDATE(value, pattern)` - Generic pattern validation
- `EXTRACT(text, pattern)` - Extract pattern matches

### String Functions
- `CONCAT(...)` - Concatenate multiple values
- `SUBSTRING(str, start, end)` - Extract substring
- `UPPER(str)` - Convert to uppercase
- `LOWER(str)` - Convert to lowercase

### Data Functions
- `LOOKUP(key, table)` - Look up value from table
- `LENGTH(str)` - Get string length
- `ROUND(number, decimals)` - Round number

### Operators
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logical: `AND`, `OR`, `NOT`
- String: `&` (concatenation)
- Regex: `~` or `MATCHES`

## 🌳 AST Visualization

The IDE includes a powerful Abstract Syntax Tree (AST) visualization feature that helps understand how DSL rules are parsed and interpreted:

### Features
- **Interactive Viewer** - Click "🌳 Show AST" to visualize any DSL rule
- **Multiple Formats**:
  - **Tree View** - ASCII art hierarchical representation
  - **JSON View** - Complete AST structure with metadata
  - **DOT Graph** - GraphViz format for visual diagrams
- **Export Options** - Download AST as JSON or DOT files
- **Copy to Clipboard** - Quick copy of tree representation

### Example AST Output

For the rule `result = price * quantity + tax`:

```
└─ Assignment: result =
   └─ BinaryOp: Add
      ├─ BinaryOp: Multiply
      │  ├─ Identifier: price
      │  └─ Identifier: quantity
      └─ Identifier: tax
```

### Use Cases
- **Debug Complex Rules** - Understand operator precedence and expression nesting
- **Educational Tool** - Learn how the parser interprets DSL expressions
- **Documentation** - Generate visual representations for rule documentation
- **Validation** - Verify that rules are parsed as intended

## 📚 Documentation

### Language Server Features

The LSP provides professional IDE features:

- **IntelliSense**: Context-aware code completion
- **Diagnostics**: Real-time error detection
- **Hover Info**: Detailed tooltips for functions and attributes
- **Semantic Tokens**: Advanced syntax highlighting
- **Code Actions**: AI-powered explanations and optimizations

### Enhanced Type System

The Data Dictionary includes comprehensive type information for all attributes:

| Attribute | SQL Type | Rust Type | Format Mask | Pattern |
|-----------|----------|-----------|-------------|---------|
| `Client.client_id` | `VARCHAR(50) PRIMARY KEY` | `String` | `XXX-999` | `^[A-Z]{3}-\d{3,}$` |
| `Client.legal_entity_name` | `VARCHAR(255) NOT NULL` | `String` | - | - |
| `Client.lei_code` | `CHAR(20)` | `String` | `XXXXXXXXXXXXXXXXXXXX` | `^[A-Z0-9]{20}$` |
| `Client.email` | `VARCHAR(255) NOT NULL` | `String` | `xxx@xxx.xxx` | Email pattern |
| `Client.risk_rating` | `ENUM('LOW','MEDIUM','HIGH')` | `RiskLevel` | - | - |
| `Client.aum_usd` | `DECIMAL(18,2)` | `rust_decimal::Decimal` | `$999,999,999,999.99` | - |
| `Client.kyc_status` | `ENUM('PENDING','APPROVED','REJECTED')` | `KycStatus` | - | - |
| `Client.pep_status` | `BOOLEAN NOT NULL` | `bool` | - | - |

### Data Dictionary

The system includes a comprehensive KYC data dictionary:

```json
{
  "entities": {
    "Client": {
      "attributes": [
        {"name": "client_id", "type": "String"},
        {"name": "risk_rating", "type": "Enum", "domain": "RiskLevel"},
        {"name": "aum_usd", "type": "Number"},
        {"name": "pep_status", "type": "Boolean"}
      ]
    }
  },
  "domains": {
    "RiskLevel": {
      "values": ["LOW", "MEDIUM", "HIGH"]
    }
  }
}
```

## 🧪 Development Commands

### Essential Commands
```bash
# Quick Start - WASM Web App
./runwasm.sh                   # One command: build + serve + open browser

# Manual Development
cd grpc-server && cargo run   # Start gRPC server (port 50051)
cd web-ui && ./build-web.sh   # Build WASM package
cd web-ui && ./serve-web.sh   # Serve on localhost:8080

# Testing & Quality
cargo build                   # Build entire workspace
cargo test --all             # Run comprehensive test suite (20+ tests)
./test-web-app.sh            # Test Web-First architecture components
cargo clippy                 # Code quality checks
```

### Database Commands
```bash
export DATABASE_URL="postgresql://$(whoami)@localhost/data_designer"
psql -d data_designer        # Connect to database
```

## 🏗️ Current Implementation Status

### ✅ COMPLETED SYSTEM
- **Web-First Architecture** - Pure Rust WASM web application
- **gRPC Microservices** - Complete Protocol Buffers API (900+ lines)
- **White Truffle Architecture** - All 3 advanced execution components
- **Complete AI Assistant System** - All 7 AI features with gRPC integration
- **Comprehensive CRUD API** - Full entity management infrastructure
- **Test Data Ecosystem** - Realistic financial services data
- **Enhanced Template Editor** - Professional DSL IDE with syntax highlighting

### Key Technologies
- **Rust** - Zero-cost abstractions with memory safety
- **WebAssembly** - Near-native performance in browsers
- **egui** - Immediate mode GUI with 60fps rendering
- **gRPC** - High-performance RPC with Protocol Buffers
- **PostgreSQL** - Vector embeddings with pgvector

## 📞 Support & Development

**Current Repository**: `/Users/adamtc007/Developer/data-designer`

For development and collaboration:
- See [CLAUDE.md](CLAUDE.md) for comprehensive project documentation
- Check the [Protocol Buffers](proto/financial_taxonomy.proto) for API definitions
- Review test data in `migrations/011_test_data_seeding.sql`

---

**Data Designer** - Production-ready web-first financial DSL platform with advanced AI assistance.