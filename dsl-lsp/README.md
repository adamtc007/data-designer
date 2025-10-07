# DSL Language Server Protocol (LSP) for KYC Rules

A Language Server Protocol implementation for the Data Designer DSL, providing intelligent IDE features for KYC and institutional client onboarding rules.

## Features

### 🎨 Syntax Highlighting
- Keywords: `IF`, `THEN`, `ELSE`, `AND`, `OR`, `NOT`
- Functions: `CONCAT`, `SUBSTRING`, `LOOKUP`, `UPPER`, `LOWER`, etc.
- Operators: Arithmetic, comparison, and logical operators
- KYC-specific attributes with semantic coloring

### ✨ IntelliSense / Auto-completion
- **Keywords**: DSL control flow keywords
- **Functions**: Built-in functions with parameter hints
- **Operators**: Mathematical and logical operators
- **KYC Attributes**: Domain-specific attributes like:
  - `client_id`, `legal_entity_name`
  - `risk_rating`, `aum_usd`
  - `kyc_completeness`, `documents_received`
  - `pep_status`, `sanctions_check`

### 🔍 Hover Information
- Function descriptions and signatures
- Operator explanations
- Keyword documentation

### ⚠️ Diagnostics
- Real-time syntax validation using the nom parser
- Parse error detection and highlighting
- Unparsed content warnings

### 📝 Semantic Tokens
- Differentiated highlighting for:
  - Keywords
  - Functions
  - Variables
  - Strings
  - Numbers
  - Comments
  - Operators

## Architecture

```
dsl-lsp/
├── src/
│   ├── lib.rs        # LSP server implementation
│   └── main.rs       # Server entry point
├── Cargo.toml        # Dependencies
└── build.sh          # Build script
```

## Building

```bash
cd dsl-lsp
./build.sh
```

This will create the LSP server binary at `target/release/dsl-lsp-server`.

## Integration

### Monaco Editor (Web)
The DSL language is already integrated into the Data Designer web app with:
- Custom language definition (`dsl-language.js`)
- Syntax highlighting theme
- Auto-completion providers
- Real-time validation

### VS Code
1. Install a generic LSP client extension
2. Configure it to use: `/path/to/dsl-lsp-server`
3. Associate `.dsl` files with the language server

### Other Editors
Any editor that supports LSP can use this server:
- Neovim (with nvim-lspconfig)
- Sublime Text (with LSP package)
- Emacs (with lsp-mode)
- IntelliJ IDEA (with LSP Support plugin)

## LSP Capabilities

- ✅ Text Document Synchronization
- ✅ Completion Provider
- ✅ Hover Provider
- ✅ Diagnostic Provider
- ✅ Semantic Tokens Provider
- 🔄 Definition Provider (planned)
- 🔄 References Provider (planned)
- 🔄 Rename Provider (planned)

## KYC Domain Support

The LSP is specifically tailored for KYC and custody banking rules:

### Supported Patterns
```dsl
# KYC Completeness Calculation
kyc_score = (documents_received / documents_required) * 100

# Risk Assessment
composite_risk = LOOKUP(country_risk, "risk_ratings") * 0.2 +
                 LOOKUP(industry_risk, "risk_ratings") * 0.3

# Conditional Logic
IF pep_status = true OR sanctions_check != "clear" THEN
    "enhanced_due_diligence"
ELSE
    "standard_due_diligence"

# String Formatting
client_label = CONCAT(legal_entity_name, " [",
                     LOOKUP(entity_type, "entity_types"), "]")

# Fee Calculation
annual_fee = aum_usd * LOOKUP(fee_schedule, "fee_tiers")
```

### Auto-complete Examples

When you type:
- `kyc` → suggests: `kyc_completeness`, `kyc_score`
- `LOOK` → suggests: `LOOKUP(key, table_name)`
- `IF` → suggests: `IF condition THEN result ELSE alternative`

## Development

To extend the LSP with new features:

1. **Add new keywords**: Update `DSL_KEYWORDS` in `lib.rs`
2. **Add new functions**: Update `DSL_FUNCTIONS` with descriptions
3. **Add domain attributes**: Update `kyc_attributes` in `get_completions()`
4. **Enhance diagnostics**: Modify `validate_document()` for better error messages

## Testing

Test the LSP server:
```bash
# Run the server
./target/release/dsl-lsp-server

# In another terminal, send LSP messages
echo '{"jsonrpc":"2.0","method":"initialize","id":1,"params":{"capabilities":{}}}' | ./target/release/dsl-lsp-server
```

## Performance

- **Fast parsing**: Uses nom parser for efficient syntax analysis
- **Incremental updates**: Full document sync with efficient diffing
- **Concurrent document handling**: Thread-safe document map
- **Memory efficient**: Uses rope data structure for large documents

## License

Part of the Data Designer project for KYC and institutional client onboarding.