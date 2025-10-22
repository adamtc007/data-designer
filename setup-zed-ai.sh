#!/bin/bash

# Setup script for Zed AI integration with Data Designer
# Configures Zed editor for optimal AI assistant usage

set -e

echo "🎯 Setting up Zed AI Integration for Data Designer"
echo "=================================================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Zed is installed
if ! command -v zed &> /dev/null; then
    print_warning "Zed editor not found. Install from: https://zed.dev"
    echo ""
    echo "Installation options:"
    echo "  • Download from https://zed.dev"
    echo "  • Homebrew: brew install --cask zed"
    echo ""
    read -p "Continue setup anyway? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    print_success "Zed editor found"
fi

# Create Zed configuration directory if it doesn't exist
ZED_CONFIG_DIR="$HOME/.config/zed"
if [ ! -d "$ZED_CONFIG_DIR" ]; then
    print_status "Creating Zed configuration directory..."
    mkdir -p "$ZED_CONFIG_DIR"
    print_success "Created $ZED_CONFIG_DIR"
fi

# Copy project-specific Zed configuration
print_status "Setting up project-specific Zed configuration..."
if [ -f ".zed/settings.json" ]; then
    cp ".zed/settings.json" "$ZED_CONFIG_DIR/settings.json.data-designer-backup"
    print_success "Backed up Zed settings to settings.json.data-designer-backup"
fi

if [ -f ".zed/keymap.json" ]; then
    cp ".zed/keymap.json" "$ZED_CONFIG_DIR/keymap.json.data-designer-backup"
    print_success "Backed up Zed keymap to keymap.json.data-designer-backup"
fi

# Check for API keys
print_status "Checking for AI provider API keys..."

api_key_found=false

# Check for OpenAI key
if security find-generic-password -a "$USER" -s "OPENAI_API_KEY" >/dev/null 2>&1; then
    print_success "OpenAI API key found in keychain"
    api_key_found=true
else
    print_warning "OpenAI API key not found in keychain"
fi

# Check for Anthropic key
if security find-generic-password -a "$USER" -s "ANTHROPIC_API_KEY" >/dev/null 2>&1; then
    print_success "Anthropic API key found in keychain"
    api_key_found=true
else
    print_warning "Anthropic API key not found in keychain"
fi

if [ "$api_key_found" = false ]; then
    print_warning "No AI provider API keys found"
    echo ""
    echo "To add API keys to keychain:"
    echo "  OpenAI:    security add-generic-password -a \"\$USER\" -s \"OPENAI_API_KEY\" -w \"your-key\""
    echo "  Anthropic: security add-generic-password -a \"\$USER\" -s \"ANTHROPIC_API_KEY\" -w \"your-key\""
fi

# Build LSP server if not already built
print_status "Checking LSP server..."
if [ ! -f "target/release/cbu-dsl-lsp-server" ]; then
    print_status "Building CBU DSL Language Server..."
    if cargo build --release --bin cbu-dsl-lsp-server; then
        print_success "LSP server built successfully"
    else
        print_error "Failed to build LSP server"
        exit 1
    fi
else
    print_success "LSP server already built"
fi

# Create AI context export
print_status "Creating AI context export..."
if [ -f "export-for-ai.sh" ]; then
    chmod +x export-for-ai.sh
    if ./export-for-ai.sh --summary; then
        print_success "AI context export ready"
    else
        print_warning "AI context export had issues"
    fi
else
    print_warning "export-for-ai.sh not found"
fi

# Create workspace-specific Zed settings
print_status "Creating workspace-specific settings..."
cat > ".zed/settings.json" << 'EOF'
{
  "assistant": {
    "default_model": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022"
    },
    "providers": {
      "openai": {
        "api_url": "https://api.openai.com/v1",
        "low_speed_timeout_in_seconds": 30
      },
      "anthropic": {
        "api_url": "https://api.anthropic.com",
        "low_speed_timeout_in_seconds": 30
      }
    }
  },
  "language_servers": {
    "cbu-dsl-lsp": {
      "binary": {
        "path": "./target/release/cbu-dsl-lsp-server",
        "arguments": []
      }
    }
  },
  "languages": {
    "LISP": {
      "language_servers": ["cbu-dsl-lsp"],
      "file_types": ["lisp", "cbu"],
      "hard_tabs": false,
      "tab_size": 2
    }
  },
  "file_types": {
    "*.cbu": "LISP",
    "*.lisp": "LISP"
  },
  "theme": {
    "mode": "system",
    "light": "One Light",
    "dark": "One Dark"
  },
  "vim_mode": true,
  "format_on_save": "on",
  "git": {
    "inline_blame": {
      "enabled": true
    }
  },
  "inlay_hints": {
    "enabled": true
  }
}
EOF

print_success "Workspace Zed settings created"

# Instructions for usage
echo ""
print_success "Zed AI Integration Setup Complete!"
echo ""
echo "🚀 Next Steps:"
echo ""
echo "1. Open Data Designer in Zed:"
echo "   zed ."
echo ""
echo "2. AI Assistant Usage:"
echo "   • Cmd+Shift+A: Toggle AI assistant panel"
echo "   • Cmd+K Cmd+D: Inline AI assistance"
echo "   • Cmd+Enter: Ask AI about selection"
echo ""
echo "3. Start Development Environment:"
echo "   ./runwasm.sh --with-lsp"
echo ""
echo "4. LSP Features Available:"
echo "   • Syntax highlighting for .lisp and .cbu files"
echo "   • Code completion (Ctrl+Space)"
echo "   • Error diagnostics"
echo "   • Hover documentation"
echo ""
echo "5. AI Context Files:"
echo "   • ai-context.md - Quick project overview"
echo "   • export/ - Full codebase exports"
echo ""
echo "💡 Pro Tips:"
echo "   • Use 'Show AI Context' in assistant to load project info"
echo "   • Ask about specific DSL syntax or financial concepts"
echo "   • Request code reviews and optimization suggestions"
echo "   • Get help with Rust, WASM, or gRPC implementation"
echo ""
print_success "Happy coding with AI assistance! 🤖"