# CBU DSL Language Server Usage Guide

The CBU DSL now has full Language Server Protocol (LSP) support with advanced IDE features!

## 🚀 Quick Start Scripts

### 1. **Enhanced WASM Runner** (`./runwasm.sh`)
Start the full WASM application with optional LSP server:

```bash
# Basic WASM app only
./runwasm.sh

# WASM app + LSP server (recommended)
./runwasm.sh --with-lsp

# Custom LSP port
./runwasm.sh --with-lsp --lsp-port=9999

# Help
./runwasm.sh --help
```

### 2. **Standalone LSP Server** (`./run-lsp-server.sh`)
Run just the LSP server in stdio mode (for editor integration):

```bash
# Start LSP server for editor integration
./run-lsp-server.sh
```

### 3. **Background LSP Server** (`./run-lsp-server-background.sh`)
Manage LSP server as a background service:

```bash
# Start background server
./run-lsp-server-background.sh start

# Check status
./run-lsp-server-background.sh status

# Stop server
./run-lsp-server-background.sh stop

# View logs
./run-lsp-server-background.sh logs

# Test connection
./run-lsp-server-background.sh test
```

## 🎨 Language Features

### **Syntax Highlighting**
- 🔵 **Keywords**: `create-cbu`, `entity`, `update-cbu`, `delete-cbu`
- 🟠 **Entity Roles**: `asset-owner`, `investment-manager`, `custodian`
- 🟡 **Strings**: Quoted string literals with escape support
- 🟢 **Comments**: LISP-style `;` comments
- 🟣 **Numbers**: Integer and floating-point literals
- **Delimiters**: Parentheses with depth tracking
- **Error Highlighting**: Invalid syntax marked in red

### **Code Completion**
- **Smart Triggers**: Auto-completion on `(`, space, and Ctrl+Space
- **Context-Aware**: Suggests appropriate completions based on cursor position
- **Rich Descriptions**: Hover documentation for all completions
- **Function Templates**: Parameter hints for DSL functions
- **Entity Roles**: All financial entity roles with descriptions

### **Error Diagnostics**
- **Real-time Validation**: Immediate syntax error feedback
- **Parentheses Matching**: Validates S-expression structure
- **String Validation**: Checks for unclosed string literals
- **Position Reporting**: Line and column error information

### **Hover Documentation**
- **Function Help**: Detailed documentation for DSL functions
- **Role Descriptions**: Entity role explanations
- **Example Usage**: Code examples for functions

## 🌐 Web UI Integration

The enhanced DSL editor in the web UI provides:

### **Editor Features**
- **Dual-Pane Layout**: Code editor with live syntax-highlighted preview
- **Theme Support**: Dark and light themes with live switching
- **Error Display**: Visual error indicators with descriptions
- **Line Numbers**: Professional editor with line number display

### **Code Completion UI**
- **Popup Interface**: Professional completion interface
- **Keyboard Navigation**: Arrow keys, Enter to apply, Escape to cancel
- **Descriptions**: Rich tooltips for all completions
- **Auto-trigger**: Intelligent completion suggestions while typing

### **Access the Enhanced Editor**
1. Start with LSP: `./runwasm.sh --with-lsp`
2. Open: http://localhost:8081
3. Navigate to: **CBU DSL IDE** tab
4. Enable: **Syntax Highlighting** checkbox
5. Try: Type `(create-cbu` and press Ctrl+Space

## 🔧 Editor Integration

### **VS Code**
Create a VS Code extension or add to settings.json:

```json
{
  "cbu-dsl-lsp.serverPath": "/path/to/data-designer/target/release/cbu-dsl-lsp-server",
  "files.associations": {
    "*.cbu": "cbu-dsl",
    "*.lisp": "cbu-dsl"
  }
}
```

### **Vim/Neovim**
With CoC or nvim-lspconfig:

```lua
require'lspconfig'.configs.cbu_dsl = {
  default_config = {
    cmd = {'/path/to/data-designer/target/release/cbu-dsl-lsp-server'},
    filetypes = {'cbu-dsl', 'lisp'},
    root_dir = lspconfig.util.root_pattern('.git', 'Cargo.toml'),
  },
}
```

### **Emacs**
With lsp-mode:

```elisp
(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "/path/to/cbu-dsl-lsp-server")
                  :major-modes '(lisp-mode)
                  :server-id 'cbu-dsl-lsp))
```

## 📝 Example DSL Files

### **Basic CBU Creation**
```lisp
;; Create a new investment fund
(create-cbu "Goldman Sachs Investment Fund" "Multi-strategy hedge fund operations"
  (entities
    (entity "GS001" "Goldman Sachs Asset Management" asset-owner)
    (entity "GS002" "Goldman Sachs Investment Advisors" investment-manager)
    (entity "BNY001" "BNY Mellon" custodian)))
```

### **CBU Update**
```lisp
;; Update existing CBU
(update-cbu "CBU001"
  (add-entities
    (entity "NEW001" "New Prime Broker" prime-broker))
  (update-metadata
    (aum 1500000000)
    (status "active")))
```

### **Query with Filters**
```lisp
;; Query CBUs with criteria
(query-cbu
  (where
    (status "active")
    (aum-range 100000000 5000000000)
    (domicile "Delaware" "Luxembourg"))
  (include
    (entities)
    (metadata)))
```

## 🎯 Benefits Over Basic Parsing

This LSP implementation goes far beyond typical parsing libraries:

### **Traditional Parsing** (nom, pest, etc.)
- ✅ Tokenization and AST generation
- ❌ No IDE integration
- ❌ No real-time feedback
- ❌ No code completion
- ❌ No hover documentation

### **Our CBU DSL LSP**
- ✅ **Full LSP Protocol** - Works with any LSP client
- ✅ **Real-time Validation** - Immediate error feedback
- ✅ **Intelligent Completion** - Context-aware suggestions
- ✅ **Rich Documentation** - Hover help and descriptions
- ✅ **Professional UI** - Syntax highlighting and themes
- ✅ **Web Integration** - Built-in editor with all features
- ✅ **Production Ready** - TCP server with process management

This provides a **professional IDE experience** for financial DSL development, making complex S-expression editing as smooth as working with mainstream programming languages!

## 🚀 Next Steps

1. **Try the Enhanced Editor**: `./runwasm.sh --with-lsp`
2. **Test Code Completion**: Press Ctrl+Space while typing
3. **Experiment with Themes**: Toggle dark/light modes
4. **Check Error Diagnostics**: Try typing invalid syntax
5. **Integrate with Your Editor**: Use the standalone LSP server

The CBU DSL now has **enterprise-grade language support** rivaling major IDEs!