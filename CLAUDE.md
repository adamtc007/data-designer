# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a sophisticated hybrid Rust/JavaScript Tauri application for designing, testing, and managing dynamic data transformation rules using a soft DSL (Domain Specific Language) system. The project features:

- **Dynamic Grammar System**: EBNF-based soft DSL where grammar rules are data-driven and editable through the UI
- **Advanced Parser**: Full nom-based parser with 6 major extensions (arithmetic, strings, functions, lookups, runtime resolution, regex)
- **Interactive Rule Editor**: Monaco Editor with live rule testing and validation
- **Grammar Editor**: Visual EBNF rule editor for modifying the DSL itself
- **Rules Engine**: Runtime expression evaluation with complex operator precedence and function calls

## Architecture

### Core Components

1. **Dynamic Grammar System** (src-tauri/src/lib.rs:221-290):
   - Load/save EBNF grammar as JSON data
   - Runtime grammar generation from EBNF
   - Grammar validation and hot-reload

2. **Enhanced Expression Engine** (src/lib.rs:42-192):
   - Complex arithmetic with operator precedence
   - String operations (concatenation, substring)
   - Function calls (CONCAT, SUBSTRING, LOOKUP)
   - Runtime attribute resolution from context

3. **Nom Parser Integration** (src/lib.rs:217-480):
   - Enhanced transpiler with full grammar support
   - Expression parsing with precedence handling
   - Runtime expression evaluation

4. **Dual Rules Engines** (src/lib.rs:483-537):
   - EnhancedRulesEngine: Supports complex expressions
   - RulesEngine: Backward compatibility for simple rules

5. **Advanced UI** (src/index.html, src/main.js):
   - Two-tab interface: Rules + Grammar editors
   - Live grammar rule editing and validation
   - Test rule execution with results display
   - Grammar visualization preview

### Key Files

- `src/lib.rs`: Enhanced Rust library with expression engine and nom parser
- `src/parser.rs`: Complete nom parser with 6 extensions including regex
- `grammar_rules.json`: Dynamic grammar storage with metadata
- `src/index.html`: Two-tab UI with Rules and Grammar editors
- `src/main.js`: Advanced frontend with grammar management and live testing
- `src-tauri/src/lib.rs`: Tauri commands for rule testing and grammar management
- `src/main.rs`: Comprehensive test suite demonstrating all 5 parser extensions

## Development Commands

### Frontend Development
```bash
npm run dev          # Start Vite dev server on port 1420
npm run build        # Build frontend assets
```

### Rust Development
```bash
# From project root (for enhanced parser testing)
cargo run            # Run comprehensive test suite with all 5 extensions
cargo build          # Build the enhanced Rust library
cargo test           # Run Rust tests

# From src-tauri directory (for Tauri app)
cd src-tauri
cargo tauri dev      # Run enhanced Tauri app with grammar editor
cargo tauri build    # Build Tauri app for production
```

### Full Application
The Tauri app automatically runs `npm run dev` and provides:
- Interactive rule testing with dropdown selection
- Live grammar editing and validation
- Grammar visualization from EBNF rules

## Enhanced DSL Features

### 1. Arithmetic Operations with Precedence
```
result = 100 + 25 * 2 - 10 / 2    # → 145.0
total = (100 + 50) * 2            # → 300.0
```

### 2. String Operations
```
message = "Hello " & name & "!"                    # Concatenation
code = SUBSTRING(user_id, 0, 3)                   # Substring extraction
full_msg = CONCAT("User: ", name, " (", role, ")") # Multi-argument concat
```

### 3. External Data Lookup
```
country_name = LOOKUP(country_code, "countries")   # External table lookup
rate = LOOKUP(tier, "rates")                       # Dynamic rate lookup
```

### 4. Runtime Attribute Resolution
```
computed = price * quantity + tax    # Attributes resolved from runtime context
complex = (base_rate + LOOKUP(tier, "rates")) * 100
```

### 5. Complex Mixed Operations
```
result = CONCAT("Rate: ", (base_rate + LOOKUP(tier, "rates")) * 100, "%")
# Combines arithmetic, lookup, and string operations
```

### 6. Regex Support and Pattern Matching
```
# Regex literals and MATCHES operator
valid_email = email ~ /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/
matches_pattern = text MATCHES r"\d{3}-\d{2}-\d{4}"  # SSN pattern

# KYC validation functions
is_valid_email = IS_EMAIL(email_address)
is_valid_lei = IS_LEI(legal_entity_id)
is_valid_swift = IS_SWIFT(swift_code)
is_valid_phone = IS_PHONE(phone_number)

# Pattern extraction and validation
extracted_code = EXTRACT(text, r"CODE-(\d+)")  # Extracts captured group
is_valid_format = VALIDATE(input, r"^[A-Z]{2}\d{6}$")
```

## Dynamic Grammar System

### Grammar Storage Format (grammar_rules.json)
- **Metadata**: Version, description, creation info
- **Rules**: EBNF rule definitions with types (normal, silent, atomic)
- **Extensions**: Operators, functions, and keywords catalog

### Grammar Rule Types
- **Normal**: Standard parsing rules
- **Silent (_)**: Rules that don't appear in parse tree
- **Atomic (@)**: Rules that capture input as single token

### Grammar Management Commands
- `load_grammar()`: Load EBNF rules from JSON
- `save_grammar()`: Persist grammar changes
- `get_grammar_rules()`: List all grammar rules
- `update_grammar_rule()`: Modify individual rules
- `generate_grammar_visualization()`: Generate grammar representation
- `validate_grammar()`: Check grammar correctness

## Test Rules Available

### Basic Operations
1. **Complex Math**: `100 + 25 * 2 - 10 / 2` → Number(145.0)
2. **String Concatenation**: `"Hello " & name & "!"` → String("Hello World!")
3. **Parentheses Precedence**: `(100 + 50) * 2` → Number(300.0)
4. **SUBSTRING Function**: `SUBSTRING(user_id, 0, 3)` → String("USR")
5. **CONCAT Function**: `CONCAT("User: ", name, " (", role, ")")` → String("User: Alice (Admin)")
6. **LOOKUP Function**: `LOOKUP(country_code, "countries")` → String("United States")
7. **Ultimate Test**: `CONCAT("Rate: ", (base_rate + LOOKUP(tier, "rates")) * 100, "%")` → String("Rate: 20%")
8. **Runtime Calculation**: `price * quantity + tax` → Number(33.65)

### Regex and KYC Validation
9. **Email Validation**: `IS_EMAIL("user@example.com")` → Boolean(true)
10. **LEI Validation**: `IS_LEI("529900T8BM49AURSDO55")` → Boolean(true)
11. **SWIFT Validation**: `IS_SWIFT("DEUTDEFF")` → Boolean(true)
12. **Phone Validation**: `IS_PHONE("+1-555-0123")` → Boolean(true)
13. **Pattern Matching**: `"ABC123" ~ /^[A-Z]+\d+$/` → Boolean(true)
14. **Pattern Extraction**: `EXTRACT("CODE-789", r"CODE-(\d+)")` → String("789")
15. **Generic Validation**: `VALIDATE("XY123456", r"^[A-Z]{2}\d{6}$")` → Boolean(true)

## UI Features

### Rules Tab
- Dropdown selection of 8 test rules
- Monaco Editor with syntax highlighting
- Live test execution with context data
- Visual results display (success/failure)
- Save functionality for custom rules

### Grammar Tab
- Visual EBNF rule browser and editor
- Rule type selection (normal/silent/atomic)
- Live grammar validation
- Grammar generation and preview
- Add/edit/delete grammar rules
- Grammar persistence to JSON

## Current State

- **Production Ready**: Full-featured soft DSL system with working Tauri IDE
- **6 Parser Extensions**: All implemented and tested including comprehensive regex support
- **Language Server Protocol**: Full LSP implementation with WebSocket connection and status indicators
- **Dynamic Grammar**: Completely configurable through UI
- **Advanced UI**: Two-tab interface with live editing
- **Comprehensive Testing**: 15+ test cases covering all features including regex/KYC validation
- **Runtime Evaluation**: Complex expression engine with precedence
- **External Integration**: Lookup table system for external data
- **Tauri Integration**: Fully functional desktop app with proper API connectivity
- **PostgreSQL Database**: Full persistence layer with rules, attributes, and categories
- **Vector Search**: pgvector integration for semantic similarity search (1536 dimensions)
- **AI Embeddings**: Automatic embedding generation using OpenAI/Anthropic APIs
- **Similar Rules Finder**: Find semantically similar rules using cosine similarity
- **Derived Attribute Builder**: Interactive UI for creating new derived attributes with dependency management
- **AST-Runtime System**: New modular architecture in data-designer-core with complete feature parity

## File Structure

```
src/
├── lib.rs              # Enhanced Rust library with expression engine
├── main.rs             # Comprehensive test suite
├── parser.rs           # Complete nom parser with 6 extensions including regex
├── test_regex.rs       # Comprehensive regex and KYC validation test suite
├── index.html          # Two-tab UI (Rules + Grammar) with Tauri API integration
├── main.js             # Advanced frontend with grammar management
├── dsl-language.js     # Monaco Editor DSL language definition
├── main-simple.js      # Simplified version for debugging/testing
├── test.html           # Basic test page for Tauri connectivity
└── simple.js           # Simple JavaScript test utilities

src-tauri/
├── src/lib.rs          # Tauri commands for rules and grammar
├── src/main.rs         # Tauri entry point
├── src/database.rs     # PostgreSQL database layer with SQLx
├── src/embeddings.rs   # Vector embedding generation and similarity search
└── tauri.conf.json     # Tauri config with withGlobalTauri enabled

dsl-lsp/                # Language Server Protocol implementation
├── src/lib.rs          # LSP server with IntelliSense and diagnostics
├── Cargo.toml          # LSP dependencies
└── build.sh            # Build script for LSP server

database/
├── schema-simple.sql   # PostgreSQL schema with pgvector
├── init-sample-data.sql # Sample rules and attributes
└── migrations/         # Database migration scripts

examples/
└── regex_kyc_validation.dsl  # Regex and KYC validation examples

test_data/              # KYC domain test data
├── source_attributes.json    # Source data for testing
└── target_attributes.json    # Target attribute mappings

grammar_rules.json      # Dynamic grammar storage
```

## Grammar Extension Examples

### Adding New Operators
```json
{
  "name": "power_op",
  "definition": "{ \"**\" }",
  "type": "normal",
  "description": "Power/exponentiation operator"
}
```

### Adding New Functions
```json
{
  "name": "uppercase_fn",
  "definition": "{ \"UPPER\" ~ \"(\" ~ expression ~ \")\" }",
  "type": "normal",
  "description": "Convert string to uppercase"
}
```

The DSL is now completely "soft" - modifiable through the UI without code changes!

## Language Server Protocol (LSP)

The project includes a comprehensive Language Server Protocol implementation using tower-lsp for professional IDE features:

### Architecture
```
Browser IDE (src/ide.html)
    ↓ WebSocket (ws://localhost:3030)
LSP Client (src/lsp-client.js)
    ↓
WebSocket Server (dsl-lsp/src/websocket_server.rs)
    ↓
Language Server (tower-lsp)
    ├── nom Parser
    ├── Data Dictionary
    ├── Diagnostics Engine
    └── AI Agents
```

### Features
- **WebSocket Support**: Browser-compatible WebSocket server for LSP communication
- **Auto-Connection**: LSP automatically connects when IDE loads
- **Smart Formatting**: Preserves line breaks and indentation during code formatting
- **Debounced Updates**: 300ms delay prevents message flooding on keystrokes
- **Auto-Reconnect**: Automatic reconnection with exponential backoff (max 3 attempts)
- **Offline Mode**: IDE gracefully falls back to client-side features when LSP unavailable
- **IntelliSense**: Context-aware auto-completion for KYC attributes and DSL functions
- **Real-time Diagnostics**: Instant validation using nom parser
- **Hover Information**: Detailed tooltips for functions, operators, and attributes
- **Semantic Tokens**: Enhanced syntax highlighting
- **Data Dictionary**: Domain-driven completions with KYC-specific attributes

### LSP Components
- `dsl-lsp/src/lib.rs` - Main LSP server implementation with tower-lsp
- `dsl-lsp/src/websocket_server.rs` - WebSocket wrapper for browser compatibility
- `dsl-lsp/src/data_dictionary.rs` - KYC domain model and attribute management
- `dsl-lsp/src/ai_agent.rs` - AI agent interfaces for Gemini/Copilot
- `dsl-lsp/src/main.rs` - Server entry point with stdio/TCP/WebSocket modes

### Running the LSP Server
```bash
cd dsl-lsp
cargo build --release

# WebSocket mode (for browser IDE)
./target/release/dsl-lsp-server --port 3030 websocket

# TCP mode (for desktop IDE clients)
./target/release/dsl-lsp-server --port 3030 tcp

# Stdio mode (for VS Code and other editors)
./target/release/dsl-lsp-server

# Generate data dictionary
./target/release/dsl-lsp-server generate-dict
```

### IDE Integration
- **Browser IDE**: Full integration via `src/ide.html` with auto-connect on page load
- **Monaco Editor**: Complete LSP client implementation in `src/lsp-client.js`
- **VS Code**: Compatible with any LSP client extension
- **Other Editors**: Works with any LSP-compatible editor

### IDE Improvements
- **Fixed Code Formatting**: Preserves line breaks and indentation
- **No Juddering**: Eliminated visual artifacts with proper debouncing
- **Connection Status**: Clear visual indicators for LSP connection state (green dot when online)
- **Graceful Degradation**: IDE remains functional even without LSP server

## Derived Attribute Builder

The IDE includes an interactive UI for creating new derived attributes with proper dependency management:

### Features
- **Visual Attribute Creation**: Click "+ New" button in the Data Dictionary sidebar
- **Dependency Selection**: Choose business entity attributes that the rule will use
- **Type System**: Select return type (String, Number, Boolean, List)
- **Rule Template Generation**: Automatically creates starter code with examples
- **Context Management**: Loads selected dependencies with sample test data
- **Test Data Panel**: Shows loaded business attributes with example values

### Workflow
1. Click "+ New" button next to "Data Dictionary"
2. Enter attribute details:
   - Name (e.g., `risk_score`, `client_category`)
   - Return type
   - Description
3. Select business attributes as dependencies (checkboxes)
4. Click "Create Attribute" to:
   - Generate rule template in editor
   - Load test context with sample data
   - Add new attribute to sidebar
   - Update tab name to show rule name

### Test Context Generation
The system automatically generates smart sample data based on attribute names:
- `*_id` fields → "CUST_12345"
- `age` fields → 35
- `income/balance/amount` → 50000
- `is_*/has_*` → true
- `type/category/status` → "standard"
- `country` → "USA"

### Rule Testing Integration
When clicking "Run Code", the system:
- Uses the loaded test context
- Passes business attributes to the rule engine
- Shows computed value for the derived attribute
- Displays test data being used

## Tauri Configuration Notes

### Important Settings
- **withGlobalTauri**: Must be set to `true` in `tauri.conf.json` to enable Tauri API injection
- **Frontend Import**: Tauri API must be imported from `@tauri-apps/api/core` for proper connectivity
- **Parameter Naming**: Backend `test_rule` command expects `dslText` parameter (not `rule`)

### Troubleshooting
- If IDE shows no content: Check `withGlobalTauri` is enabled in tauri.conf.json
- If Tauri API not detected: Verify import from '@tauri-apps/api/core' in index.html
- If rules don't load: Check invoke function is properly accessible via window.__TAURI__.invoke
- If tests fail with parameter error: Ensure frontend sends `dslText` not `rule` to test_rule command
- If editor content doesn't refresh: Ensure Monaco editor is exposed to window.editor and use setValue() with layout() and focus()

## Recent Updates (October 2025)

### AST Visualization Feature (NEW)
- **Interactive AST Viewer**: Click "🌳 Show AST" button to visualize Abstract Syntax Trees
- **Multiple View Formats**:
  - Tree View: ASCII art hierarchical representation
  - JSON View: Complete AST structure with all node metadata
  - DOT Graph: GraphViz format for rendering visual diagrams
- **Export Capabilities**: Export AST as JSON or DOT files for documentation
- **Implementation Details**:
  - Backend command: `visualize_ast(dslText: String)`
  - Uses nom parser from `data_designer::parser::parse_rule`
  - Converts parser AST to visualization-friendly format
  - Generates three output formats simultaneously
- **UI Integration**:
  - New tab in output panel: "AST View 🌳"
  - Switchable views with export/copy buttons
  - Debug output in both terminal and browser console
- **Key Files Modified**:
  - `src-tauri/src/lib.rs`: Added `visualize_ast` command and AST conversion functions
  - `src/index.html`: Added AST UI panel and JavaScript functions
  - Functions exposed globally: `showAST()`, `switchASTView()`, `exportAST()`, `copyAST()`

### Derived Attribute Builder Enhancements
- **Editor Content Refresh Fix**: Fixed issue where editor wouldn't update with rule template
  - Monaco editor now properly exposed to `window.editor` for global access
  - Editor clears before setting new content to force refresh
  - Uses `editor.layout()` and `editor.focus()` to ensure visual update
  - Tab name correctly updates to show new attribute name
  - Rule template with business attributes now loads properly

### Key UI Improvements
- **Single Tab Design**: Simplified to "Current Rule" tab that updates dynamically
- **Smart Template Generation**: Creates context-aware rule templates based on attribute types
- **Test Data Panel**: Shows live context values for selected dependencies
- **Auto-Save**: Rules automatically saved as user types