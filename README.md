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
- **Real-time Diagnostics** - Instant syntax validation and error reporting
- **AI-powered Assistance** - Optional Gemini/Copilot integration for intelligent suggestions
- **Data Dictionary Support** - Domain-driven completions for KYC attributes
- **Semantic Highlighting** - Advanced token-based syntax coloring

### Development Environment
- **Tauri Desktop App** - Native cross-platform application
- **Monaco Editor Integration** - VS Code-quality editing experience
- **Interactive Testing** - Live rule execution with sample data
- **Grammar Editor** - Visual EBNF rule modification interface

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

## 🎯 Quick Start

### Running the IDE

1. **Start the Language Server:**
```bash
./dsl-lsp/target/release/dsl-lsp-server tcp --port 3030
```

2. **Launch the Tauri App:**
```bash
npm run tauri dev
```

3. **Open the Enhanced IDE:**
   - Navigate to the IDE tab
   - Click "Connect LSP" to enable IntelliSense
   - Start writing DSL rules with full IDE support

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