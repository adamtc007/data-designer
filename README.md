# Data Designer - DSL for KYC & Data Transformation

A sophisticated Domain-Specific Language (DSL) system for designing, testing, and managing data transformation rules with a focus on KYC (Know Your Customer) and institutional client onboarding workflows.

## 🚀 Features

### Core DSL Capabilities
- **Dynamic Grammar System** - EBNF-based soft DSL with runtime-modifiable grammar
- **Advanced Parser** - Full nom-based parser with comprehensive expression support
- **Regex Support** - Pattern matching and validation with KYC-specific functions
- **Expression Engine** - Complex arithmetic, string operations, and function calls
- **Runtime Resolution** - Dynamic attribute resolution from context data

### Language Server Protocol (LSP)
- **Full IDE Integration** - Professional development environment with IntelliSense
- **WebSocket Support** - Browser-compatible LSP connection
- **Enhanced Type System** - SQL types, Rust types, format masks, and validation patterns
- **Rich Hover Information** - Comprehensive type details on hover
- **Real-time Diagnostics** - Instant syntax validation and error reporting
- **Semantic Highlighting** - Advanced token-based syntax coloring

### AI Assistant Integration
- **Intelligent Code Assistant** - Built-in AI agent for DSL development help
- **Multiple API Support** - Works with OpenAI (GPT-4) and Anthropic (Claude)
- **Automatic API Key Detection** - Uses system environment variables (ANTHROPIC_API_KEY, OPENAI_API_KEY)
- **Robust Fallback System** - Comprehensive offline mode with intelligent responses
- **Context-Aware Help** - Understands current rule and provides relevant suggestions
- **Never Fails** - Always provides helpful responses, even without API keys

### Database & Vector Search
- **PostgreSQL Integration** - Full database persistence for rules and attributes
- **pgvector Extension** - Vector similarity search with 1536-dimensional embeddings
- **Semantic Search** - Find similar rules using cosine similarity
- **Automatic Embeddings** - Generate embeddings using OpenAI/Anthropic APIs
- **Rule Versioning** - Track rule changes over time
- **Execution Logging** - Audit trail for all rule executions

### Development Environment
- **Tauri Desktop App** - Native cross-platform application
- **Monaco Editor Integration** - VS Code-quality editing experience
- **Interactive Testing** - Live rule execution with sample data
- **Grammar Editor** - Visual EBNF rule modification interface
- **Resizable Panes** - Adjustable layout with persistent preferences
- **Rules Catalogue** - Comprehensive rule management with source/target tracking

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│         Tauri Desktop Application        │
├─────────────────────────────────────────┤
│  ┌──────────┐  ┌────────────────────┐  │
│  │  Monaco  │  │   Grammar Editor    │  │
│  │  Editor  │  │  (EBNF Modification)│  │
│  └─────┬────┘  └──────────────────┘  │
│        │                              │
│  ┌─────▼────────────────────────────┐  │
│  │    Language Server Protocol      │  │
│  │  (tower-lsp implementation)      │  │
│  └─────┬────────────────────────────┘  │
│        │                              │
│  ┌─────▼────────────────────────────┐  │
│  │     Nom Parser & Engine          │  │
│  │  • Expression evaluation         │  │
│  │  • Regex validation              │  │
│  │  • KYC domain functions          │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## 📦 Installation

### Prerequisites
- Rust 1.70+
- Node.js 18+
- Tauri CLI
- PostgreSQL 17 (for database persistence)
- pgvector 0.8.1+ (for semantic search)

### Database Setup

```bash
# Install PostgreSQL (if not already installed)
# macOS:
brew install postgresql@17

# Install pgvector extension
cd /tmp
git clone --branch v0.8.1 https://github.com/pgvector/pgvector.git
cd pgvector
PG_CONFIG=/opt/homebrew/opt/postgresql@17/bin/pg_config make
PG_CONFIG=/opt/homebrew/opt/postgresql@17/bin/pg_config make install

# Create database
createdb data_designer

# Initialize schema
cd /path/to/data-designer
psql -d data_designer < database/schema-simple.sql

# Load sample data (optional)
psql -d data_designer < database/init-sample-data.sql

# Set environment variable
export DATABASE_URL="postgres://$(whoami)@localhost/data_designer"
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/data-designer.git
cd data-designer

# Install dependencies
npm install

# Build the Language Server
cd dsl-lsp
cargo build --release
cd ..

# Run development version
npm run tauri dev

# Build for production
npm run tauri build
```

## 🤖 AI Assistant

The IDE includes a powerful AI Assistant that helps with DSL development:

### Features
- **Always Available** - Works with or without API keys
- **Intelligent Fallback** - Provides helpful responses even offline
- **Context-Aware** - Understands your current rule and data dictionary
- **Multiple Providers** - Supports OpenAI and Anthropic APIs

### Setup Options

#### Option 1: Automatic (Recommended)
Set environment variable before running:
```bash
export ANTHROPIC_API_KEY="your-key-here"  # For Claude
# or
export OPENAI_API_KEY="your-key-here"     # For GPT-4

npm run tauri dev
```

#### Option 2: Manual Configuration
1. Click "⚙ Settings" in the IDE
2. Choose provider (OpenAI or Anthropic)
3. Enter API key (optional - leave blank for offline mode)
4. Save settings

#### Option 3: Offline Mode (Always Works)
No setup needed! The assistant provides intelligent responses using:
- Pattern recognition for common questions
- Context analysis of your current rule
- Comprehensive DSL examples and explanations
- Debugging assistance and optimization tips

### What the Assistant Can Do
- **Explain** - DSL syntax, functions, and patterns
- **Debug** - Identify and fix common errors
- **Examples** - Provide relevant code samples
- **Optimize** - Suggest improvements to your rules
- **Functions** - List and explain available functions
- **Attributes** - Show business vs derived data

## 🎯 Quick Start

### Running the Web IDE (Recommended)

1. **Build and Start the Language Server:**
```bash
# Build the LSP server
cd dsl-lsp
cargo build --release

# Start with WebSocket support for browser
./target/release/dsl-lsp-server --port 3030 websocket
```

2. **Open the IDE in Browser:**
```bash
# From project root
open src/ide.html  # macOS
# or
xdg-open src/ide.html  # Linux
# or
start src/ide.html  # Windows
```

3. **Using the IDE:**
   - The IDE auto-connects to LSP on load
   - Hover over attributes like `Client.client_id` to see:
     - SQL type: `VARCHAR(50) PRIMARY KEY`
     - Rust type: `String`
     - Format mask: `XXX-999`
     - Pattern: `^[A-Z]{3}-\d{3,}$`
     - Required status and constraints
   - Type `Client.` for intelligent completions
   - Get real-time syntax validation

### Running the Tauri Desktop App (Alternative)

1. **Start the Language Server:**
```bash
./dsl-lsp/target/release/dsl-lsp-server tcp --port 3030
```

2. **Launch the Tauri App:**
```bash
npm run tauri dev
```

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

## 🧪 Testing

### Running Tests

```bash
# Run Rust tests
cargo test

# Run parser tests
cargo run --example test_parser

# Run regex validation tests
cargo test test_regex

# Run LSP tests
cd dsl-lsp && cargo test
```

### Test Coverage

- Parser: 15+ comprehensive test cases
- Regex validation: KYC-specific pattern tests
- Expression evaluation: Arithmetic and string operations
- LSP features: Completion, diagnostics, hover

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details

## 🙏 Acknowledgments

- Built with [Tauri](https://tauri.app/) for cross-platform desktop apps
- [tower-lsp](https://github.com/tower-lsp/tower-lsp) for Language Server Protocol
- [nom](https://github.com/Geal/nom) for parser combinators
- [Monaco Editor](https://microsoft.github.io/monaco-editor/) for web-based editing

## 📞 Support

For issues, questions, or suggestions:
- Open an issue on GitHub
- Check the [documentation](docs/)
- See [CLAUDE.md](CLAUDE.md) for AI assistant guidance

---

**Data Designer** - Empowering KYC and data transformation workflows with an intelligent DSL platform.