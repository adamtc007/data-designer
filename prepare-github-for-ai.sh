#!/bin/bash

# GitHub Repository Preparation Script for AI Assistant Access
# Prepares the repository with AI-friendly documentation and exports

set -e

echo "📦 Preparing GitHub Repository for AI Assistant Access"
echo "====================================================="

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

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_error "Not in a git repository"
    exit 1
fi

print_success "Repository detected"

# Get repository information
REPO_URL=$(git config --get remote.origin.url 2>/dev/null || echo "No remote origin")
CURRENT_BRANCH=$(git branch --show-current)
COMMIT_COUNT=$(git rev-list --count HEAD)

print_status "Repository: $REPO_URL"
print_status "Branch: $CURRENT_BRANCH"
print_status "Commits: $COMMIT_COUNT"

# Create AI-friendly documentation structure
print_status "Creating AI documentation structure..."

# Create docs directory if it doesn't exist
mkdir -p docs/ai-integration

# Create comprehensive AI README
cat > "docs/ai-integration/AI-README.md" << 'EOF'
# Data Designer - AI Assistant Integration Guide

## 🎯 Quick Access for AI Assistants

This document provides AI assistants (ChatGPT, Claude, Gemini) with everything needed to understand and work with the Data Designer codebase.

### Project Summary
**Data Designer** is a web-first financial DSL platform built in Rust with WASM, featuring microservices architecture, Language Server Protocol implementation, and comprehensive AI integration.

### Architecture Overview
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Web UI        │    │   gRPC Server    │    │   Database      │
│   (WASM/egui)   │◄──►│   (Port 50051)   │◄──►│   (PostgreSQL)  │
│   Port 8081     │    │   HTTP Fallback  │    │   + pgvector    │
└─────────────────┘    │   Port 8080      │    └─────────────────┘
                       └──────────────────┘
```

### Key Technologies
- **Frontend**: Rust + WASM + egui (immediate mode GUI)
- **Backend**: gRPC microservices with Protocol Buffers
- **Database**: PostgreSQL with vector embeddings (pgvector)
- **DSL**: S-expression (LISP-style) financial workflow language
- **LSP**: Full Language Server Protocol with syntax highlighting
- **AI**: Multi-provider integration (OpenAI, Anthropic, Offline)

### Quick Start for AI Assistants
1. **Understand the codebase**: Read `CLAUDE.md` for comprehensive overview
2. **Explore the DSL**: Check S-expression examples in `ai-context.md`
3. **Review architecture**: See `proto/financial_taxonomy.proto` for API definitions
4. **Test locally**: Use `./runwasm.sh --with-lsp` for full environment

### File Structure (AI-Optimized)
```
data-designer/
├── CLAUDE.md                    # 📋 Complete project documentation
├── ai-context.md               # 🤖 AI assistant quick reference
├── LSP_USAGE.md               # 🎨 Language Server documentation
├── web-ui/                    # 🌐 WASM web application
│   ├── src/lib.rs            # Entry point
│   ├── src/app.rs            # Main application logic
│   └── src/cbu_dsl_ide.rs    # DSL editor with syntax highlighting
├── grpc-server/               # 🔧 Microservices backend
│   └── src/main.rs           # gRPC server implementation
├── data-designer-core/        # 🦀 Core engine and database
│   ├── src/lib.rs            # Library interface
│   ├── src/capability_engine.rs         # Capability execution
│   ├── src/capability_execution_engine.rs  # Advanced execution
│   ├── src/lisp_cbu_dsl.rs   # S-expression parser
│   └── src/transpiler.rs     # Multi-target code generation
├── proto/                     # 📡 API definitions
│   └── financial_taxonomy.proto  # Complete gRPC API (900+ lines)
├── migrations/                # 🗄️ Database schema
│   └── 011_test_data_seeding.sql  # Test data with DSL examples
├── export/                    # 📤 AI-friendly exports
│   ├── codebase-full.txt     # Complete codebase
│   ├── codebase-core.txt     # Core functionality only
│   └── codebase-summary.txt  # Executive summary
└── scripts/                   # 🚀 Utility scripts
    ├── runwasm.sh            # One-command deployment
    ├── export-for-ai.sh      # AI export utility
    └── setup-zed-ai.sh       # Zed editor integration
```

### DSL Examples for AI Understanding
```lisp
;; Create a Client Business Unit with entities
(create-cbu "Goldman Sachs Investment Fund" "Multi-strategy hedge fund operations"
  (entities
    (entity "GS001" "Goldman Sachs Asset Management" asset-owner)
    (entity "GS002" "Goldman Sachs Investment Advisors" investment-manager)
    (entity "BNY001" "BNY Mellon" custodian)))

;; Update CBU with new entities and metadata
(update-cbu "CBU001"
  (add-entities
    (entity "NEW001" "New Prime Broker" prime-broker))
  (update-metadata
    (aum 1500000000)
    (status "active")))

;; Query CBUs with complex filters
(query-cbu
  (where
    (status "active")
    (aum-range 100000000 5000000000)
    (domicile "Delaware" "Luxembourg"))
  (include
    (entities)
    (metadata)))
```

### AI Assistant Capabilities
1. **Code Analysis**: Understand Rust, WASM, gRPC patterns
2. **DSL Development**: Create and modify S-expression workflows
3. **Architecture Review**: Microservices, database design, API patterns
4. **Testing Support**: Unit tests, integration tests, E2E scenarios
5. **Performance Optimization**: WASM bundle size, gRPC efficiency
6. **Documentation**: Technical writing, API docs, user guides

### Common AI Tasks
- **Feature Development**: Add new DSL capabilities or UI components
- **Bug Fixes**: Resolve compilation errors, runtime issues, logic bugs
- **Code Review**: Architecture feedback, best practices, security analysis
- **Optimization**: Performance improvements, memory usage, bundle size
- **Testing**: Create comprehensive test coverage, mock data, scenarios
- **Documentation**: Update README files, API docs, usage guides

### Key Implementation Details for AI
1. **Rust Ownership**: Careful lifetime management, especially in async contexts
2. **WASM Constraints**: Limited API surface, size optimization requirements
3. **gRPC Integration**: Protocol Buffer compilation, async service traits
4. **egui Patterns**: Immediate mode GUI with retained state management
5. **PostgreSQL**: Migration management, complex queries, vector operations
6. **LSP Implementation**: Tower-LSP framework, semantic token providers
7. **Financial Domain**: Regulatory compliance, entity relationships, fund accounting

### Getting Started (AI Assistants)
1. **Read Foundation**: Start with `CLAUDE.md` for complete context
2. **Understand DSL**: Review S-expression syntax and examples
3. **Explore Codebase**: Use exports in `export/` directory for full context
4. **Local Testing**: Run `./runwasm.sh --with-lsp` for live development
5. **Make Changes**: Follow Rust best practices, test thoroughly
6. **Documentation**: Update relevant docs and examples

### Contact & Support
- **Issues**: Create GitHub issues for bugs and feature requests
- **Discussions**: Use GitHub Discussions for architecture questions
- **AI Context**: All exports automatically updated via CI/CD
- **Live Demo**: Available at localhost:8081 when running locally

---

*This README is automatically generated and updated. Last updated: $(date)*
EOF

print_success "Created comprehensive AI README"

# Create AI-specific development guide
cat > "docs/ai-integration/AI-DEVELOPMENT-GUIDE.md" << 'EOF'
# AI Development Guide for Data Designer

## 🤖 AI Assistant Development Workflow

### Understanding the Codebase
1. **Start with CLAUDE.md** - Complete project overview and architecture
2. **Review ai-context.md** - Quick reference for development context
3. **Explore LSP_USAGE.md** - Language Server Protocol implementation details
4. **Check export/ directory** - Full codebase exports for deep analysis

### Development Patterns for AI

#### Rust Patterns Used
```rust
// Async trait implementations for gRPC services
#[tonic::async_trait]
impl FinancialTaxonomyService for MyService {
    async fn get_cbu_list(&self, request: Request<GetCbuListRequest>)
        -> Result<Response<GetCbuListResponse>, Status> {
        // Implementation
    }
}

// Error handling with anyhow
use anyhow::{Result, Context};
fn parse_dsl(input: &str) -> Result<LispValue> {
    // Parsing logic with context
}

// egui immediate mode GUI patterns
impl eframe::App for DataDesignerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // UI code
        });
    }
}
```

#### Common Development Tasks

**1. Adding New DSL Functions**
```rust
// In lisp_cbu_dsl.rs
fn parse_new_function(&mut self) -> Result<LispValue> {
    // Function parsing logic
}

// In capability_execution_engine.rs
async fn execute_new_capability(&self, params: &[LispValue]) -> Result<ExecutionResult> {
    // Execution logic
}
```

**2. Creating New UI Components**
```rust
// In web-ui/src/
pub struct NewComponent {
    state: ComponentState,
}

impl NewComponent {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Component layout
        });
    }
}
```

**3. Adding gRPC Endpoints**
```protobuf
// In proto/financial_taxonomy.proto
service FinancialTaxonomyService {
    rpc NewEndpoint(NewRequest) returns (NewResponse);
}

message NewRequest {
    string parameter = 1;
}
```

#### Testing Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_functionality() {
        // Async test implementation
    }

    #[test]
    fn test_dsl_parsing() {
        let input = "(create-cbu \"test\" \"description\")";
        let result = parse_dsl(input).unwrap();
        // Assertions
    }
}
```

### AI-Specific Considerations

#### Working with Financial Domain
- **Entity Relationships**: CBU → Products → Services → Resources
- **Regulatory Compliance**: KYC, AML, custody requirements
- **Fund Accounting**: NAV calculations, position tracking, reporting
- **Workflow Orchestration**: Multi-system coordination, approvals

#### Performance Considerations
- **WASM Bundle Size**: Minimize dependencies, use `wee_alloc`
- **gRPC Efficiency**: Protocol Buffer optimization, connection pooling
- **Database Queries**: Indexed searches, pagination, caching
- **Memory Management**: Rust ownership, async context handling

#### Common Pitfalls for AI to Avoid
1. **Lifetime Issues**: Careful with references in async contexts
2. **WASM Limitations**: No threading, limited API surface
3. **Protocol Buffer Changes**: Require regeneration and compatibility
4. **egui State**: Immediate mode requires careful state management
5. **Database Migrations**: Always include rollback scripts

### Development Environment Setup

#### Prerequisites
```bash
# Rust toolchain
rustup target add wasm32-unknown-unknown

# Protocol Buffer compiler
brew install protobuf  # macOS
apt install protobuf-compiler  # Ubuntu

# Database
brew install postgresql
createdb data_designer
```

#### Quick Start
```bash
# Clone and setup
git clone <repository>
cd data-designer

# Run full environment
./runwasm.sh --with-lsp

# Access web UI
open http://localhost:8081

# Access LSP
# Configured in .zed/settings.json for Zed editor
```

### AI Assistant Workflow

#### Code Analysis Workflow
1. **Read exports** - Use `export/codebase-full.txt` for complete context
2. **Understand changes** - Review git diffs and commit messages
3. **Test locally** - Use `./runwasm.sh` for verification
4. **Follow patterns** - Match existing code style and architecture

#### Feature Development Workflow
1. **Define scope** - Clear requirements and acceptance criteria
2. **Design API** - Protocol Buffer definitions and service interfaces
3. **Implement core** - Business logic in `data-designer-core`
4. **Add gRPC** - Service implementation in `grpc-server`
5. **Build UI** - Frontend components in `web-ui`
6. **Test thoroughly** - Unit, integration, and E2E tests
7. **Update docs** - README, API docs, examples

#### Bug Fix Workflow
1. **Reproduce issue** - Local testing and debugging
2. **Identify root cause** - Code analysis and logging
3. **Implement fix** - Minimal, targeted changes
4. **Verify fix** - Comprehensive testing
5. **Prevent regression** - Add tests for the bug scenario

### Resources for AI Assistants
- **Rust Documentation**: https://doc.rust-lang.org/
- **egui Documentation**: https://docs.rs/egui/
- **tonic gRPC**: https://docs.rs/tonic/
- **Protocol Buffers**: https://developers.google.com/protocol-buffers
- **PostgreSQL**: https://www.postgresql.org/docs/
- **WASM**: https://webassembly.org/

### AI Success Metrics
- **Code Quality**: Passes clippy lints, follows Rust best practices
- **Test Coverage**: Comprehensive unit and integration tests
- **Performance**: WASM bundle under 15MB, sub-second response times
- **Documentation**: Clear, accurate, and up-to-date
- **User Experience**: Intuitive UI, helpful error messages

---

*Remember: This is a financial services platform. Security, reliability, and compliance are paramount.*
EOF

print_success "Created AI development guide"

# Create GitHub workflow for automatic AI exports
print_status "Creating GitHub Actions workflow for AI exports..."
mkdir -p .github/workflows

cat > ".github/workflows/ai-export.yml" << 'EOF'
name: AI Assistant Export Generation

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  schedule:
    # Run daily at 2 AM UTC
    - cron: '0 2 * * *'

jobs:
  generate-ai-exports:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0  # Full history for better context

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        targets: wasm32-unknown-unknown

    - name: Cache Cargo dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Install system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y protobuf-compiler

    - name: Generate AI exports
      run: |
        chmod +x export-for-ai.sh
        ./export-for-ai.sh --all

    - name: Update AI documentation
      run: |
        # Update timestamps in documentation
        sed -i "s/Last updated:.*/Last updated: $(date)/" docs/ai-integration/AI-README.md

        # Generate file statistics
        echo "## Codebase Statistics" >> docs/ai-integration/STATS.md
        echo "- Total files: $(find . -type f -name '*.rs' | wc -l)" >> docs/ai-integration/STATS.md
        echo "- Lines of code: $(find . -name '*.rs' -exec cat {} \; | wc -l)" >> docs/ai-integration/STATS.md
        echo "- Last updated: $(date)" >> docs/ai-integration/STATS.md

    - name: Commit and push AI exports
      run: |
        git config --local user.email "action@github.com"
        git config --local user.name "GitHub Action"
        git add export/ docs/ai-integration/
        git diff --staged --quiet || git commit -m "🤖 Auto-update AI exports and documentation"
        git push
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

    - name: Create AI context release
      if: github.ref == 'refs/heads/main'
      run: |
        # Create compressed archive for easy download
        tar -czf data-designer-ai-context.tar.gz export/ docs/ai-integration/ ai-context.md CLAUDE.md LSP_USAGE.md

        # Upload as release asset if this is a tagged release
        if [[ $GITHUB_REF == refs/tags/* ]]; then
          gh release upload ${{ github.ref_name }} data-designer-ai-context.tar.gz
        fi
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
EOF

print_success "Created GitHub Actions workflow"

# Create issue templates for AI-assisted development
print_status "Creating GitHub issue templates..."
mkdir -p .github/ISSUE_TEMPLATE

cat > ".github/ISSUE_TEMPLATE/ai-feature-request.md" << 'EOF'
---
name: AI-Assisted Feature Request
about: Request a new feature with AI assistant development support
title: '[AI-FEATURE] '
labels: ['enhancement', 'ai-assisted']
assignees: ''
---

## Feature Description
**Brief description of the feature**
A clear and concise description of what you want to happen.

## AI Assistant Context
**Which AI assistant will work on this?**
- [ ] ChatGPT
- [ ] Claude
- [ ] Gemini
- [ ] Other: ___________

**Relevant codebase areas**
- [ ] Web UI (web-ui/)
- [ ] gRPC Server (grpc-server/)
- [ ] Core Engine (data-designer-core/)
- [ ] Database (migrations/)
- [ ] DSL Parser (lisp_cbu_dsl.rs)
- [ ] LSP Server (cbu-dsl-lsp/)

## Technical Requirements
**Implementation details**
- Expected API changes (if any)
- Database schema changes (if any)
- UI/UX requirements
- Performance considerations

## AI Development Resources
**Helpful context for AI assistant**
- [ ] Feature requires new DSL syntax
- [ ] Feature needs new gRPC endpoints
- [ ] Feature involves database changes
- [ ] Feature affects WASM bundle size
- [ ] Feature needs comprehensive testing

## Acceptance Criteria
**Definition of done**
- [ ] Feature implemented and tested
- [ ] Documentation updated
- [ ] AI exports regenerated
- [ ] No breaking changes (unless intentional)

## Additional Context
Add any other context, screenshots, or examples about the feature request here.

---
**For AI Assistants:**
- Review `export/codebase-full.txt` for complete context
- Follow patterns in `docs/ai-integration/AI-DEVELOPMENT-GUIDE.md`
- Test with `./runwasm.sh --with-lsp`
- Update relevant documentation
EOF

cat > ".github/ISSUE_TEMPLATE/ai-bug-report.md" << 'EOF'
---
name: AI-Assisted Bug Report
about: Report a bug with AI assistant debugging support
title: '[AI-BUG] '
labels: ['bug', 'ai-assisted']
assignees: ''
---

## Bug Description
**Brief description of the bug**
A clear and concise description of what the bug is.

## Reproduction Steps
**Steps to reproduce the behavior:**
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

## Expected Behavior
A clear and concise description of what you expected to happen.

## Actual Behavior
A clear and concise description of what actually happened.

## Environment
**Development environment:**
- OS: [e.g., macOS 14.0, Ubuntu 22.04, Windows 11]
- Rust version: [e.g., 1.70.0]
- Browser: [e.g., Chrome 118, Firefox 119, Safari 17]
- WASM target: [e.g., wasm32-unknown-unknown]

## Error Details
**Console output, stack traces, or error messages:**
```
Paste error messages here
```

## AI Assistant Context
**Which AI assistant will debug this?**
- [ ] ChatGPT
- [ ] Claude
- [ ] Gemini
- [ ] Other: ___________

**Suspected areas (check all that apply)**
- [ ] Frontend (egui/WASM)
- [ ] Backend (gRPC server)
- [ ] Database queries
- [ ] DSL parsing
- [ ] LSP server
- [ ] Build system
- [ ] Protocol Buffers

## AI Debugging Resources
**Helpful context for AI assistant**
- [ ] Bug is reproducible locally
- [ ] Bug affects specific browsers
- [ ] Bug involves async operations
- [ ] Bug is related to memory management
- [ ] Bug affects performance
- [ ] Bug involves financial calculations

## Additional Context
Add any other context about the problem here.

**Screenshots**
If applicable, add screenshots to help explain your problem.

---
**For AI Assistants:**
- Start debugging with `./runwasm.sh --with-lsp`
- Review error logs in browser console and terminal
- Use `export/codebase-full.txt` for context
- Follow debugging patterns in `docs/ai-integration/AI-DEVELOPMENT-GUIDE.md`
- Test fix thoroughly before submitting
EOF

print_success "Created GitHub issue templates"

# Create pull request template
cat > ".github/pull_request_template.md" << 'EOF'
## Pull Request Summary
**Brief description of changes**

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] AI-assisted development

## AI Assistant Information
**If this PR was developed with AI assistance:**
- [ ] ChatGPT
- [ ] Claude
- [ ] Gemini
- [ ] Other: ___________

**AI development approach:**
- [ ] Full implementation by AI
- [ ] AI-assisted debugging
- [ ] AI code review and optimization
- [ ] AI-generated tests
- [ ] AI documentation updates

## Testing
**How has this been tested?**
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Integration tests pass
- [ ] Manual testing in browser
- [ ] LSP server functionality verified
- [ ] WASM build successful
- [ ] gRPC endpoints tested

## Changes Made
**Detailed list of changes:**
- [ ] Modified existing files (list below)
- [ ] Added new files (list below)
- [ ] Updated documentation
- [ ] Updated AI exports
- [ ] Database schema changes

**Files modified:**
- `path/to/file.rs` - Brief description of changes
- `path/to/other.rs` - Brief description of changes

## Documentation
- [ ] Updated CLAUDE.md if architecture changed
- [ ] Updated ai-context.md if AI context changed
- [ ] Updated LSP_USAGE.md if LSP features changed
- [ ] Updated relevant code comments
- [ ] Regenerated AI exports with `./export-for-ai.sh`

## Performance Impact
- [ ] No performance impact
- [ ] Improves performance
- [ ] May impact performance (explain below)

**Performance notes:**

## Breaking Changes
- [ ] No breaking changes
- [ ] Contains breaking changes (explain below)

**Breaking change details:**

## Additional Notes
Add any additional notes, context, or concerns here.

## AI Assistant Review
**For AI assistants reviewing this PR:**
- Review `export/codebase-full.txt` for full context
- Verify changes follow patterns in `docs/ai-integration/AI-DEVELOPMENT-GUIDE.md`
- Test locally with `./runwasm.sh --with-lsp`
- Check for potential security, performance, or maintainability issues
EOF

print_success "Created pull request template"

# Create AI assistant onboarding script
cat > "docs/ai-integration/AI-ONBOARDING.md" << 'EOF'
# AI Assistant Onboarding for Data Designer

## 🚀 Quick Start for New AI Assistants

### 1. Essential Reading (5 minutes)
1. **CLAUDE.md** - Complete project overview and architecture
2. **ai-context.md** - Quick reference for immediate context
3. **docs/ai-integration/AI-README.md** - This document

### 2. Understanding the Codebase (10 minutes)
1. **Export files** - Download `export/codebase-full.txt` for complete context
2. **Architecture** - Review the 3-tier microservices structure
3. **DSL Examples** - Understand S-expression financial workflow syntax

### 3. Local Development Setup (15 minutes)
```bash
# Prerequisites (if working locally)
rustup target add wasm32-unknown-unknown
brew install protobuf postgresql

# Quick start
./runwasm.sh --with-lsp

# Access points
open http://localhost:8081  # Web UI
# LSP server on port 9257
```

### 4. Key Concepts for AI Assistants

#### Financial Domain Knowledge
- **CBU (Client Business Unit)**: Organizational structure for financial entities
- **Entity Roles**: asset-owner, investment-manager, custodian, prime-broker, etc.
- **Fund Accounting**: NAV calculations, position tracking, regulatory reporting
- **Workflow Orchestration**: Multi-system coordination with approvals

#### Technical Architecture
- **Web-First**: WASM application with egui immediate mode GUI
- **Microservices**: gRPC-first with HTTP fallback for reliability
- **Language Server**: Full LSP implementation with syntax highlighting
- **AI Integration**: Multi-provider system with secure keychain storage

#### Development Patterns
- **Rust Ownership**: Careful lifetime management in async contexts
- **Error Handling**: Comprehensive Result types with anyhow context
- **Testing**: Unit, integration, and property-based testing
- **Documentation**: AI-friendly exports and comprehensive inline docs

### 5. Common AI Tasks

#### Code Development
```rust
// Typical patterns you'll encounter
#[tonic::async_trait]
impl ServiceTrait for Implementation {
    async fn method(&self, req: Request<T>) -> Result<Response<U>, Status> {
        // gRPC service implementation
    }
}

// egui UI patterns
impl Component {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Immediate mode GUI code
        });
    }
}
```

#### DSL Development
```lisp
;; S-expression patterns for financial workflows
(create-cbu "Fund Name" "Description"
  (entities
    (entity "ID" "Name" role)))

(configure-capability "capability-name"
  (parameters
    (param "key" "value")))
```

#### Testing Patterns
```rust
#[tokio::test]
async fn test_integration() {
    // Async testing with comprehensive scenarios
}
```

### 6. AI Assistant Workflow

#### Before Starting Any Task
1. **Read current context** - Always check latest exports and documentation
2. **Understand requirements** - Clear acceptance criteria and scope
3. **Plan implementation** - Consider architecture, patterns, and testing

#### During Development
1. **Follow patterns** - Match existing code style and architecture
2. **Test thoroughly** - Unit tests, integration tests, manual verification
3. **Document changes** - Update relevant docs and examples
4. **Consider impact** - Performance, security, maintainability

#### After Completion
1. **Verify functionality** - Full system testing with `./runwasm.sh --with-lsp`
2. **Update exports** - Run `./export-for-ai.sh` for future AI assistants
3. **Document thoroughly** - Clear commit messages and PR descriptions

### 7. Quality Standards

#### Code Quality
- **Rust Best Practices**: Ownership, error handling, async patterns
- **Performance**: WASM bundle size, gRPC efficiency, database optimization
- **Security**: Input validation, secure key storage, SQL injection prevention
- **Maintainability**: Clear code structure, comprehensive documentation

#### Testing Requirements
- **Unit Tests**: Core business logic, DSL parsing, data transformations
- **Integration Tests**: gRPC services, database operations, UI components
- **End-to-End**: Complete user workflows, error scenarios, edge cases

#### Documentation Standards
- **Code Comments**: Clear explanations of complex logic
- **API Documentation**: Comprehensive service and method descriptions
- **User Guides**: Clear examples and usage patterns
- **AI Context**: Keep exports and context files updated

### 8. Troubleshooting Guide

#### Common Issues
1. **Compilation Errors**: Check Rust ownership, lifetime annotations
2. **WASM Issues**: Verify target configuration, dependency compatibility
3. **gRPC Problems**: Protocol Buffer compilation, service registration
4. **Database Issues**: Migration status, connection configuration
5. **LSP Problems**: Server build status, client configuration

#### Debugging Approach
1. **Local Reproduction**: Always test locally first
2. **Log Analysis**: Check browser console, server logs, database logs
3. **Incremental Testing**: Isolate issues with unit tests
4. **Context Review**: Use exports to understand full system state

### 9. Resources and References

#### Documentation
- **Rust**: https://doc.rust-lang.org/
- **egui**: https://docs.rs/egui/
- **tonic**: https://docs.rs/tonic/
- **Protocol Buffers**: https://developers.google.com/protocol-buffers

#### Project-Specific
- **Complete Context**: `export/codebase-full.txt`
- **Development Guide**: `docs/ai-integration/AI-DEVELOPMENT-GUIDE.md`
- **API Definitions**: `proto/financial_taxonomy.proto`
- **Test Examples**: `migrations/011_test_data_seeding.sql`

### 10. Success Metrics

#### Development Quality
- [ ] Code compiles without warnings
- [ ] All tests pass (`cargo test --all`)
- [ ] Clippy lints pass (`cargo clippy --all`)
- [ ] WASM builds successfully
- [ ] LSP server functions correctly

#### User Experience
- [ ] Intuitive UI with clear error messages
- [ ] Responsive performance (< 2s for operations)
- [ ] Comprehensive help and documentation
- [ ] Robust error handling and recovery

#### AI Integration
- [ ] Context files updated and accurate
- [ ] Documentation clear and comprehensive
- [ ] Examples working and tested
- [ ] Future AI assistants can understand changes

---

**Welcome to the Data Designer AI assistant team! 🤖**

Your contributions help build a world-class financial DSL platform with enterprise-grade reliability and user experience.
EOF

print_success "Created AI onboarding guide"

# Update main README to include AI integration information
print_status "Updating main README with AI integration section..."

# Check if README exists and add AI section
if [ -f "README.md" ]; then
    # Add AI integration section if not already present
    if ! grep -q "AI Assistant Integration" README.md; then
        cat >> "README.md" << 'EOF'

## 🤖 AI Assistant Integration

Data Designer includes comprehensive AI assistant integration for development support.

### Quick AI Access
```bash
# Export codebase for AI assistants
./export-for-ai.sh

# Setup Zed editor with AI integration
./setup-zed-ai.sh

# Prepare GitHub repository for AI access
./prepare-github-for-ai.sh
```

### AI-Friendly Resources
- **Complete Context**: `export/codebase-full.txt`
- **Quick Reference**: `ai-context.md`
- **Development Guide**: `docs/ai-integration/AI-DEVELOPMENT-GUIDE.md`
- **Onboarding**: `docs/ai-integration/AI-ONBOARDING.md`

### Supported AI Assistants
- **ChatGPT** - Full codebase understanding and development
- **Claude** - Architecture review and optimization
- **Gemini** - Testing and documentation
- **Zed Integration** - Real-time AI assistance during development

The AI integration includes automatic export generation, comprehensive documentation, and development workflow optimization for seamless AI-assisted development.
EOF
        print_success "Updated README with AI integration section"
    else
        print_warning "README already contains AI integration section"
    fi
else
    print_warning "README.md not found - skipping update"
fi

# Generate repository statistics
print_status "Generating repository statistics..."
cat > "docs/ai-integration/REPOSITORY-STATS.md" << EOF
# Data Designer Repository Statistics

Generated: $(date)

## Codebase Overview
- **Total Rust files**: $(find . -name '*.rs' -not -path './target/*' | wc -l)
- **Lines of Rust code**: $(find . -name '*.rs' -not -path './target/*' -exec cat {} \; | wc -l)
- **Total files**: $(find . -type f -not -path './target/*' -not -path './.git/*' | wc -l)
- **Protocol Buffer files**: $(find . -name '*.proto' | wc -l)
- **SQL migration files**: $(find . -name '*.sql' | wc -l)
- **Shell scripts**: $(find . -name '*.sh' | wc -l)

## Repository Information
- **Current branch**: $(git branch --show-current)
- **Total commits**: $(git rev-list --count HEAD)
- **Contributors**: $(git shortlog -sn | wc -l)
- **Last commit**: $(git log -1 --format='%h - %s (%cr)')

## Directory Structure
\`\`\`
$(tree -I 'target|.git|node_modules' -L 3 2>/dev/null || find . -type d -not -path './target/*' -not -path './.git/*' | head -20)
\`\`\`

## Recent Activity
\`\`\`
$(git log --oneline -10)
\`\`\`

## Package Information
\`\`\`toml
$(cat Cargo.toml | head -20)
\`\`\`
EOF

print_success "Generated repository statistics"

# Final summary and instructions
echo ""
print_success "GitHub Repository Preparation Complete!"
echo ""
echo "🎯 AI Assistant Integration Ready"
echo ""
echo "📁 Created Files:"
echo "   • docs/ai-integration/AI-README.md - Comprehensive AI guide"
echo "   • docs/ai-integration/AI-DEVELOPMENT-GUIDE.md - Development patterns"
echo "   • docs/ai-integration/AI-ONBOARDING.md - Quick start for new AI assistants"
echo "   • docs/ai-integration/REPOSITORY-STATS.md - Repository statistics"
echo "   • .github/workflows/ai-export.yml - Automatic export generation"
echo "   • .github/ISSUE_TEMPLATE/ - AI-assisted issue templates"
echo "   • .github/pull_request_template.md - PR template with AI context"
echo ""
echo "🔗 AI Assistant Access Methods:"
echo "   1. **Direct Access**: Share export/codebase-full.txt with AI assistants"
echo "   2. **GitHub Integration**: AI assistants can browse repository documentation"
echo "   3. **Zed Editor**: Real-time AI assistance during development"
echo "   4. **Automated Updates**: GitHub Actions keep exports current"
echo ""
echo "🚀 Next Steps:"
echo "   1. Commit and push these changes to GitHub"
echo "   2. Enable GitHub Actions for automatic AI export generation"
echo "   3. Share repository URL with AI assistants"
echo "   4. Use ./setup-zed-ai.sh for local AI integration"
echo ""
echo "💡 AI Assistant Instructions:"
echo "   • Start with docs/ai-integration/AI-README.md"
echo "   • Use export/codebase-full.txt for complete context"
echo "   • Follow patterns in AI-DEVELOPMENT-GUIDE.md"
echo "   • Test locally with ./runwasm.sh --with-lsp"
echo ""
print_success "Repository is now AI-assistant ready! 🤖"