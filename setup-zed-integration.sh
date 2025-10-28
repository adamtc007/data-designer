#!/bin/bash
# setup-zed-integration.sh
# Sets up Zed editor integration with Web Claude, Zed Assistant, and Claude Code
# Run this from the data-designer repository root

set -e

echo "🔧 Setting up Zed integration for data-designer..."

# Create .zed directory
echo "📁 Creating .zed directory..."
mkdir -p .zed

# Create .zed/settings.json
echo "⚙️  Creating .zed/settings.json..."
cat > .zed/settings.json << 'EOF'
{
  "assistant": {
    "version": "2",
    "default_model": {
      "provider": "anthropic",
      "model": "claude-sonnet-4-5"
    }
  },
  "context_files": [
    "REFACTOR_PLAN.md",
    ".zed/context.md",
    "docs/code-snippets/*.md"
  ],
  "terminal": {
    "env": {
      "REVIEW_MODE": "true"
    }
  }
}
EOF

# Create .zed/context.md
echo "📝 Creating .zed/context.md..."
cat > .zed/context.md << 'EOF'
# Current Development Context

## Project
Financial services onboarding platform with KYC systems for custody banks and retail broker-dealers.

## Active Work
Refactoring onboarding WASM UI to use clean state manager pattern.

## Key Architecture
```
WASM/egui UI (renders state)
    ↓ (dispatch actions)
OnboardingManager (owns state, makes backend calls)
    ↓ (HTTP/REST)
gRPC Microservices
    ↓ (SQL)
PostgreSQL + pgvector
```

## Key Files
- `REFACTOR_PLAN.md` - Master architecture plan (if exists)
- `onboarding-wasm/src/app.rs` - Main app
- `onboarding-wasm/src/onboarding/manager.rs` - State manager
- `onboarding-wasm/src/onboarding/state.rs` - DslState definition

## Architecture Rules (Critical)
1. UI NEVER calls backend directly
2. ALL state goes through OnboardingManager
3. Manager owns DslState
4. UI dispatches actions, renders state
5. No point-to-point coupling

## Current Phase
Implementation and iteration

## Review Status
- [ ] app.rs reviewed
- [ ] manager.rs reviewed
- [ ] state.rs reviewed
- [ ] Call stack verified

## Known Issues
[Add issues as they come up]

## Build Commands
```bash
./runobd.sh          # Run onboarding WASM app
./runwasm.sh         # Run main WASM app
cargo build          # Build workspace
cargo test --all     # Run tests
```

## Next Steps
[Update as you progress]
EOF

# Create docs/code-snippets directory
echo "📂 Creating docs/code-snippets directory..."
mkdir -p docs/code-snippets

# Create docs/code-snippets/README.md
echo "📄 Creating docs/code-snippets/README.md..."
cat > docs/code-snippets/README.md << 'EOF'
# Code Snippets for Review

This directory contains copies of key source files formatted as markdown for easy review by Web Claude.

## Purpose

Web Claude (browser-based) cannot directly access your local filesystem or clone git repos, but it can fetch files from GitHub. By copying key files here as markdown and pushing to GitHub, you enable Web Claude to review your code while maintaining the source of truth in the actual source directories.

## Workflow

1. **After making changes**, run `./review-prep.sh` to copy files here
2. **Commit and push** to GitHub
3. **Share the GitHub URL** with Web Claude for review
4. **Web Claude reviews** and provides feedback
5. **Apply feedback** to actual source files
6. **Repeat** as needed

## Files

Files are copied from their source locations and wrapped in markdown code blocks:
- `app.rs.md` - Main application (`onboarding-wasm/src/app.rs`)
- `manager.rs.md` - State manager (`onboarding-wasm/src/onboarding/manager.rs`)
- `state.rs.md` - State definitions (`onboarding-wasm/src/onboarding/state.rs`)
- `actions.rs.md` - Action definitions (`onboarding-wasm/src/onboarding/actions.rs`)

## Note

These are COPIES for review purposes only. The actual source of truth is in `src/` and `onboarding-wasm/src/`.
EOF

# Create review-prep.sh script
echo "🔨 Creating review-prep.sh..."
cat > review-prep.sh << 'EOF'
#!/bin/bash
# review-prep.sh
# Prepares code files for Web Claude review by copying them to docs/code-snippets/

set -e

echo "📋 Preparing code for review..."

# Ensure docs/code-snippets exists
mkdir -p docs/code-snippets

# Function to wrap file in markdown
wrap_in_markdown() {
    local src_file=$1
    local dest_file=$2
    local title=$3
    
    if [ -f "$src_file" ]; then
        echo "  ✓ Copying $src_file"
        {
            echo "# $title"
            echo ""
            echo '```rust'
            cat "$src_file"
            echo '```'
            echo ""
            echo "---"
            echo "*Source: \`$src_file\`*"
            echo ""
            echo "*Last updated: $(date)*"
        } > "$dest_file"
    else
        echo "  ⚠ Warning: $src_file not found, skipping"
    fi
}

# Copy onboarding-wasm files if they exist
if [ -d "onboarding-wasm/src" ]; then
    echo "📦 Copying onboarding-wasm files..."
    wrap_in_markdown "onboarding-wasm/src/lib.rs" "docs/code-snippets/lib.rs.md" "lib.rs - WASM Entry Point"
    wrap_in_markdown "onboarding-wasm/src/app.rs" "docs/code-snippets/app.rs.md" "app.rs - Main Application"
    
    # Onboarding module files
    if [ -d "onboarding-wasm/src/onboarding" ]; then
        wrap_in_markdown "onboarding-wasm/src/onboarding/mod.rs" "docs/code-snippets/onboarding-mod.rs.md" "onboarding/mod.rs - Module Root"
        wrap_in_markdown "onboarding-wasm/src/onboarding/manager.rs" "docs/code-snippets/manager.rs.md" "manager.rs - State Manager"
        wrap_in_markdown "onboarding-wasm/src/onboarding/state.rs" "docs/code-snippets/state.rs.md" "state.rs - State Definitions"
        wrap_in_markdown "onboarding-wasm/src/onboarding/actions.rs" "docs/code-snippets/actions.rs.md" "actions.rs - Action Definitions"
        wrap_in_markdown "onboarding-wasm/src/onboarding/client.rs" "docs/code-snippets/client.rs.md" "client.rs - gRPC Client Wrapper"
    fi
    
    # UI module files
    if [ -d "onboarding-wasm/src/ui" ]; then
        wrap_in_markdown "onboarding-wasm/src/ui/mod.rs" "docs/code-snippets/ui-mod.rs.md" "ui/mod.rs - UI Module Root"
        wrap_in_markdown "onboarding-wasm/src/ui/client_info.rs" "docs/code-snippets/ui-client-info.rs.md" "ui/client_info.rs - Client Info UI"
        wrap_in_markdown "onboarding-wasm/src/ui/kyc_docs.rs" "docs/code-snippets/ui-kyc-docs.rs.md" "ui/kyc_docs.rs - KYC Documents UI"
    fi
else
    echo "⚠ Warning: onboarding-wasm/src not found"
fi

# Copy proto file if it exists
if [ -f "proto/onboarding.proto" ]; then
    echo "📡 Copying proto file..."
    {
        echo "# onboarding.proto - Protocol Buffer Definitions"
        echo ""
        echo '```protobuf'
        cat "proto/onboarding.proto"
        echo '```'
    } > "docs/code-snippets/onboarding.proto.md"
fi

echo ""
echo "✅ Review preparation complete!"
echo ""
echo "Files copied to docs/code-snippets/"
echo ""
echo "Next steps:"
echo "  1. Review the files locally"
echo "  2. git add docs/code-snippets/"
echo "  3. git commit -m 'Prepare code for review'"
echo "  4. git push"
echo "  5. Share this URL with Web Claude:"
echo "     https://github.com/adamtc007/data-designer/tree/main/docs/code-snippets"
echo ""
EOF

chmod +x review-prep.sh

# Create .gitignore entry if needed
if [ -f .gitignore ]; then
    if ! grep -q "docs/code-snippets/\*.md" .gitignore; then
        echo ""
        echo "# Code snippets for review (generated)" >> .gitignore
        echo "# Uncomment next line if you don't want to commit these" >> .gitignore
        echo "# docs/code-snippets/*.md" >> .gitignore
        echo "📝 Updated .gitignore"
    fi
fi

echo ""
echo "✅ Zed integration setup complete!"
echo ""
echo "Created:"
echo "  ✓ .zed/settings.json - Zed configuration"
echo "  ✓ .zed/context.md - Development context for Zed assistant"
echo "  ✓ docs/code-snippets/README.md - Review workflow documentation"
echo "  ✓ review-prep.sh - Script to prepare code for review"
echo ""
echo "Usage:"
echo ""
echo "1. In Zed editor, open assistant with cmd+? (or ctrl+?)"
echo "   It will automatically use context from .zed/context.md and REFACTOR_PLAN.md"
echo ""
echo "2. When ready for Web Claude review, run:"
echo "   ./review-prep.sh"
echo "   git add docs/code-snippets/ .zed/"
echo "   git commit -m 'Add code for review'"
echo "   git push"
echo ""
echo "3. Share with Web Claude:"
echo "   'Review https://github.com/adamtc007/data-designer/tree/main/docs/code-snippets'"
echo ""
echo "4. Zed Assistant can help with quick checks:"
echo "   'Does app.rs follow the architecture in REFACTOR_PLAN.md?'"
echo "   'Review manager.rs for direct backend calls'"
echo ""
echo "Happy coding! 🚀"
