# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a sophisticated hybrid Rust/JavaScript Tauri application for designing, testing, and managing dynamic data transformation rules using a soft DSL (Domain Specific Language) system. The project features:

- **Dynamic Grammar System**: EBNF-based soft DSL where grammar rules are data-driven and editable through the UI
- **Advanced Parser**: Full Pest-based parser with 5 major extensions (arithmetic, strings, functions, lookups, runtime resolution)
- **Interactive Rule Editor**: Monaco Editor with live rule testing and validation
- **Grammar Editor**: Visual EBNF rule editor for modifying the DSL itself
- **Rules Engine**: Runtime expression evaluation with complex operator precedence and function calls

## Architecture

### Core Components

1. **Dynamic Grammar System** (src-tauri/src/lib.rs:221-290):
   - Load/save EBNF grammar as JSON data
   - Runtime Pest grammar generation
   - Grammar validation and hot-reload

2. **Enhanced Expression Engine** (src/lib.rs:42-192):
   - Complex arithmetic with operator precedence
   - String operations (concatenation, substring)
   - Function calls (CONCAT, SUBSTRING, LOOKUP)
   - Runtime attribute resolution from context

3. **Pest Parser Integration** (src/lib.rs:217-480):
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
   - Pest grammar generation preview

### Key Files

- `src/lib.rs`: Enhanced Rust library with expression engine and Pest parser
- `src/dsl.pest`: Complete EBNF grammar definition with 5 extensions
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
- Pest grammar generation from EBNF rules

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
- `generate_pest_grammar()`: Convert EBNF to Pest format
- `validate_grammar()`: Check grammar correctness

## Test Rules Available

1. **Complex Math**: `100 + 25 * 2 - 10 / 2` → Number(145.0)
2. **String Concatenation**: `"Hello " & name & "!"` → String("Hello World!")
3. **Parentheses Precedence**: `(100 + 50) * 2` → Number(300.0)
4. **SUBSTRING Function**: `SUBSTRING(user_id, 0, 3)` → String("USR")
5. **CONCAT Function**: `CONCAT("User: ", name, " (", role, ")")` → String("User: Alice (Admin)")
6. **LOOKUP Function**: `LOOKUP(country_code, "countries")` → String("United States")
7. **Ultimate Test**: `CONCAT("Rate: ", (base_rate + LOOKUP(tier, "rates")) * 100, "%")` → String("Rate: 20%")
8. **Runtime Calculation**: `price * quantity + tax` → Number(33.65)

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
- Pest grammar generation and preview
- Add/edit/delete grammar rules
- Grammar persistence to JSON

## Current State

- **Production Ready**: Full-featured soft DSL system with working Tauri IDE
- **5 Parser Extensions**: All implemented and tested
- **Dynamic Grammar**: Completely configurable through UI
- **Advanced UI**: Two-tab interface with live editing
- **Comprehensive Testing**: 8 test cases covering all features - all passing
- **Runtime Evaluation**: Complex expression engine with precedence
- **External Integration**: Lookup table system for external data
- **Tauri Integration**: Fully functional desktop app with proper API connectivity

## File Structure

```
src/
├── lib.rs              # Enhanced Rust library with expression engine
├── main.rs             # Comprehensive test suite
├── dsl.pest            # Complete EBNF grammar with 5 extensions
├── index.html          # Two-tab UI (Rules + Grammar) with Tauri API integration
├── main.js             # Advanced frontend with grammar management
├── main-simple.js      # Simplified version for debugging/testing
├── test.html           # Basic test page for Tauri connectivity
└── simple.js           # Simple JavaScript test utilities

src-tauri/
├── src/lib.rs          # Tauri commands for rules and grammar
├── src/main.rs         # Tauri entry point
└── tauri.conf.json     # Tauri config with withGlobalTauri enabled

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