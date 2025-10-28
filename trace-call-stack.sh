#!/bin/bash
# trace-call-stack.sh
# Extracts code paths across layers for Web Claude to analyze call stacks
# Usage: ./trace-call-stack.sh [action_name]
# Example: ./trace-call-stack.sh "submit_kyc"

set -e

ACTION=${1:-"all"}

echo "🔍 Tracing call stacks for: $ACTION"
echo "================================================"

OUTPUT_FILE="docs/call-traces/trace-${ACTION}-$(date +%Y%m%d-%H%M%S).md"
mkdir -p docs/call-traces

cat > "$OUTPUT_FILE" << 'HEADER'
# Call Stack Trace Analysis

**Generated:** $(date)
**Action:** $ACTION

## Purpose
This document contains code excerpts across all layers to enable Web Claude to:
1. Trace the complete call stack from UI to database
2. Identify architectural violations
3. Spot point-to-point coupling
4. Generate specific refactor instructions

## Architecture Layers
```
Layer 1: WASM/egui UI        (onboarding-wasm/src/)
Layer 2: HTTP/REST Client     (onboarding-wasm/src/onboarding/client.rs)
Layer 3: HTTP/REST Server     (grpc-server/src/http/)
Layer 4: gRPC Client Wrapper  (grpc-server/src/grpc_client/)
Layer 5: gRPC Service         (grpc-server/src/service/)
Layer 6: Database             (grpc-server/src/db/)
```

## Expected Flow (Correct Architecture)
```
UI Button Click
  → OnboardingManager.dispatch(Action)
    → OnboardingManager.handle_action()
      → HTTP POST /api/onboarding/{action}
        → HTTP handler routes to gRPC client
          → gRPC Service method
            → Database query
```

---

HEADER

# Function to extract function/method definitions with context
extract_function() {
    local file=$1
    local pattern=$2
    local layer_name=$3
    
    if [ ! -f "$file" ]; then
        echo "⚠ File not found: $file" >> "$OUTPUT_FILE"
        return
    fi
    
    echo "" >> "$OUTPUT_FILE"
    echo "## Layer: $layer_name" >> "$OUTPUT_FILE"
    echo "**File:** \`$file\`" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    # Use rg (ripgrep) if available, otherwise grep
    if command -v rg &> /dev/null; then
        # Extract function with 5 lines of context after
        rg -A 20 "$pattern" "$file" >> "$OUTPUT_FILE" 2>/dev/null || echo "No matches found" >> "$OUTPUT_FILE"
    else
        grep -A 20 "$pattern" "$file" >> "$OUTPUT_FILE" 2>/dev/null || echo "No matches found" >> "$OUTPUT_FILE"
    fi
    
    echo "" >> "$OUTPUT_FILE"
    echo "---" >> "$OUTPUT_FILE"
}

# Layer 1: UI Layer - Button clicks and event handlers
echo "# Layer 1: UI - Event Handlers & Button Clicks" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

if [ -f "onboarding-wasm/src/app.rs" ]; then
    echo "## onboarding-wasm/src/app.rs" >> "$OUTPUT_FILE"
    echo '```rust' >> "$OUTPUT_FILE"
    cat "onboarding-wasm/src/app.rs" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
fi

if [ -d "onboarding-wasm/src/ui" ]; then
    for ui_file in onboarding-wasm/src/ui/*.rs; do
        if [ -f "$ui_file" ]; then
            echo "" >> "$OUTPUT_FILE"
            echo "## $(basename $ui_file)" >> "$OUTPUT_FILE"
            echo '```rust' >> "$OUTPUT_FILE"
            cat "$ui_file" >> "$OUTPUT_FILE"
            echo '```' >> "$OUTPUT_FILE"
        fi
    done
fi

# Layer 2: OnboardingManager - State management
echo "" >> "$OUTPUT_FILE"
echo "# Layer 2: State Manager" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

if [ -f "onboarding-wasm/src/onboarding/manager.rs" ]; then
    echo "## onboarding-wasm/src/onboarding/manager.rs" >> "$OUTPUT_FILE"
    echo '```rust' >> "$OUTPUT_FILE"
    cat "onboarding-wasm/src/onboarding/manager.rs" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
fi

if [ -f "onboarding-wasm/src/onboarding/state.rs" ]; then
    echo "" >> "$OUTPUT_FILE"
    echo "## onboarding-wasm/src/onboarding/state.rs" >> "$OUTPUT_FILE"
    echo '```rust' >> "$OUTPUT_FILE"
    cat "onboarding-wasm/src/onboarding/state.rs" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
fi

if [ -f "onboarding-wasm/src/onboarding/actions.rs" ]; then
    echo "" >> "$OUTPUT_FILE"
    echo "## onboarding-wasm/src/onboarding/actions.rs" >> "$OUTPUT_FILE"
    echo '```rust' >> "$OUTPUT_FILE"
    cat "onboarding-wasm/src/onboarding/actions.rs" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
fi

# Layer 3: HTTP/gRPC Client (WASM side)
echo "" >> "$OUTPUT_FILE"
echo "# Layer 3: Backend Client (WASM → Backend)" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

if [ -f "onboarding-wasm/src/onboarding/client.rs" ]; then
    echo "## onboarding-wasm/src/onboarding/client.rs" >> "$OUTPUT_FILE"
    echo '```rust' >> "$OUTPUT_FILE"
    cat "onboarding-wasm/src/onboarding/client.rs" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
fi

# Layer 4: gRPC Server side
echo "" >> "$OUTPUT_FILE"
echo "# Layer 4: gRPC Server" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

if [ -f "grpc-server/src/service/onboarding_service.rs" ]; then
    echo "## grpc-server/src/service/onboarding_service.rs" >> "$OUTPUT_FILE"
    echo '```rust' >> "$OUTPUT_FILE"
    cat "grpc-server/src/service/onboarding_service.rs" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
fi

# Layer 5: Database layer
echo "" >> "$OUTPUT_FILE"
echo "# Layer 5: Database Layer" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

if [ -f "grpc-server/src/db/onboarding_repo.rs" ]; then
    echo "## grpc-server/src/db/onboarding_repo.rs" >> "$OUTPUT_FILE"
    echo '```rust' >> "$OUTPUT_FILE"
    cat "grpc-server/src/db/onboarding_repo.rs" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
fi

# Protocol Buffers definition
echo "" >> "$OUTPUT_FILE"
echo "# Protocol Buffers Definition" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

if [ -f "proto/onboarding.proto" ]; then
    echo "## proto/onboarding.proto" >> "$OUTPUT_FILE"
    echo '```protobuf' >> "$OUTPUT_FILE"
    cat "proto/onboarding.proto" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
fi

# Add analysis request
cat >> "$OUTPUT_FILE" << 'FOOTER'

---

# Analysis Request for Web Claude

## Tasks

### 1. Trace Complete Call Stacks
For each major UI action (StartOnboarding, SubmitKyc, AdvanceStep, etc.):
- Map the complete call path from UI button → Database
- List each function/method called
- Identify which layer each call is in
- Show the data flow

### 2. Identify Architectural Violations
Look for:
- ❌ UI calling backend directly (skipping OnboardingManager)
- ❌ UI calling gRPC directly (skipping HTTP layer)
- ❌ State mutations outside OnboardingManager
- ❌ Business logic in UI components
- ❌ Point-to-point coupling

### 3. Verify Correct Patterns
Check for:
- ✅ UI only dispatches actions to OnboardingManager
- ✅ OnboardingManager owns all state
- ✅ Manager makes HTTP calls (not gRPC directly from WASM)
- ✅ Clean layer boundaries
- ✅ No backend logic in UI

### 4. Generate Refactoring Instructions
For each violation found:
1. Show the current code (quote the specific lines)
2. Explain why it's wrong (which layer boundary it violates)
3. Provide the corrected code
4. Show where it should go

### 5. Create Call Stack Diagrams
Generate markdown diagrams showing:
- Current (actual) call flow
- Expected (correct) call flow
- Highlight differences

## Output Format

Please structure your response as:

```markdown
# Call Stack Analysis Report

## Executive Summary
[High-level findings]

## Detailed Call Stacks
[For each action, map the flow]

## Violations Found
[List each violation with code quotes]

## Refactoring Required
[Specific changes needed]

## Updated REFACTOR_PLAN.md
[Generate updated plan if needed]
```

FOOTER

echo "" >> "$OUTPUT_FILE"
echo "✅ Trace complete!"
echo ""
echo "Generated: $OUTPUT_FILE"
echo ""
echo "Next steps:"
echo "1. Review the trace locally: cat $OUTPUT_FILE"
echo "2. git add docs/call-traces/"
echo "3. git commit -m 'Add call stack trace for analysis'"
echo "4. git push"
echo "5. Share with Web Claude:"
echo "   'Analyze call stacks in https://github.com/adamtc007/data-designer/blob/main/$OUTPUT_FILE'"
echo ""
echo "Web Claude will:"
echo "  • Trace complete call paths across all layers"
echo "  • Identify architectural violations"
echo "  • Show exactly where coupling exists"
echo "  • Generate specific refactor instructions"
echo ""
