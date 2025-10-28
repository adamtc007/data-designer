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
