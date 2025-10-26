# 🚀 Onboarding Workflow Platform

**End-to-end onboarding workflow compiler and execution platform**

A standalone Rust/WASM application for designing, compiling, and executing client onboarding workflows using metadata-driven configuration and IR-based task orchestration.

---

## 🎯 Overview

This is the **runtime/execution platform** for the onboarding system, complementing the editor/designer tools in `web-ui`.

### Platform Separation

- **web-ui** → **Design Time** - Edit CBU DSL, Resource DSL, design capabilities
- **onboarding-ui** → **Runtime** - Compile & execute onboarding workflows (this app)

### Architecture

```
┌─────────────────┐      ┌──────────────────┐      ┌─────────────────┐
│  YAML Metadata  │  →   │  Compiler (IR)   │  →   │  Async Executor │
│                 │      │                  │      │                 │
│ • Products      │      │  • Plan          │      │  • Task Scheduler│
│ • CBU Templates │      │  • IDD (gaps)    │      │  • Adapters     │
│ • Resources     │      │  • Bindings      │      │  • Logging      │
└─────────────────┘      └──────────────────┘      └─────────────────┘
```

### Key Features

✅ **100% Rust/WASM** - Zero JavaScript frameworks
✅ **Cross-Platform** - Native desktop + web browser
✅ **Three-Panel UI** - YAML editor | Intent form | Compiled output
✅ **Live Editing** - Edit metadata in-app with auto-save
✅ **Workflow Compilation** - Generate execution plans from configs
✅ **Async Execution** - Task orchestration with dependency resolution
✅ **60fps Rendering** - Native performance with egui

---

## 🚀 Quick Start

### Desktop (Native)

```bash
# From project root
./run-onboarding-desk.sh
```

**Features:**
- Full IDE integration & debugging
- Native file system access
- System-level performance
- No browser required

### Web (WASM)

```bash
# From project root
./run-onboarding-wasm.sh
```

**Features:**
- Runs in any modern browser
- No installation required
- Identical UI to desktop
- Automatic backend startup

**Then open:** http://localhost:8000

---

## 📖 How to Use

### 1. YAML Configuration Panel (Left)

**Edit workflow metadata:**

- **Product Catalog** - Products and services (e.g., GlobalCustody@v3)
- **CBU Templates** - Client Business Unit templates
- **Resource Dictionaries** - Resource type definitions

**Actions:**
- Switch between files using tabs
- Edit YAML directly in code editor
- Click **💾 Save** to persist changes

### 2. Intent Editor Panel (Middle)

**Configure workflow parameters:**

```yaml
Instance ID: OR-2025-00042
CBU ID: CBU-12345
Products: GlobalCustody@v3
```

**Team Users (JSON Array):**
```json
[
  {"email": "ops.admin@client.com", "role": "Administrator"},
  {"email": "ops.approver@client.com", "role": "Approver"}
]
```

**CBU Profile (JSON Object):**
```json
{"region": "EU"}
```

### 3. Output Viewer Panel (Right)

**View compiled results:**

1. Click **⚙ Compile Workflow** to generate:
   - **Execution Plan** - Task DAG with dependencies
   - **IDD** - Information Dependency Diagram (data gaps)
   - **Bindings** - Variable bindings

2. Click **▶ Execute Workflow** to run the plan:
   - Real-time execution log
   - Task completion status
   - Error reporting

---

## 🏗️ Architecture

### Frontend (Rust/WASM)

**Entry Points:**
- `src/main.rs` - Desktop application (native)
- `src/lib.rs` - Web application (WASM)

**Core Components:**
- `src/app.rs` - Main UI application (300+ lines)
- `src/state_manager.rs` - Async state management with Arc<Mutex<>>
- `src/http_client.rs` - HTTP REST client (reqwest)
- `src/wasm_utils.rs` - Cross-platform utilities

### Backend (Rust)

**Server:** `grpc-server` (port 8080)

**Endpoints:**
- `GET  /api/onboarding/get-metadata` - Fetch YAML configs
- `POST /api/onboarding/update-metadata` - Save YAML changes
- `POST /api/onboarding/compile` - Compile workflow → Plan/IDD
- `POST /api/onboarding/execute` - Execute compiled plan

**Data Source:**
- `onboarding/metadata/` - YAML configuration files
  - `product_catalog.yaml`
  - `cbu_templates.yaml`
  - `resource_dicts/*.yaml`

---

## 🔧 Development

### Building

```bash
# Desktop build
cargo build --bin onboarding-desktop

# WASM build
cd onboarding-ui
./build-web.sh

# Full workspace build
cargo build --package onboarding-ui
```

### Running

```bash
# Development mode (desktop)
cargo run --bin onboarding-desktop --features tokio

# Development mode (web)
cd onboarding-ui
./build-web.sh && ./serve-web.sh
```

### Testing

```bash
# Run all tests
cargo test --package onboarding-ui

# Run with backend
DATABASE_URL="postgresql:///data_designer?user=adamtc007" \
  cargo run --bin grpc-server
```

---

## 📊 Workflow Compilation

### Input: YAML + Intent

```yaml
# Product Catalog
products:
  - id: GlobalCustody@v3
    services:
      - serviceId: Safekeeping
      - serviceId: TradeCapture
        options:
          - id: instructionMethod
            type: select
            choices: ["SWIFT", "API", "ManualPlatform"]
```

### Output: Execution Plan (IR)

```json
{
  "instance_id": "OR-2025-00042",
  "cbu_id": "CBU-12345",
  "products": ["GlobalCustody@v3"],
  "steps": [
    {
      "id": "d1",
      "kind": {
        "type": "SolicitData",
        "options": ["instructionMethod", "enabledMarkets"],
        "audience": "Client"
      },
      "needs": [],
      "after": []
    },
    {
      "id": "cfg:custody-core:v1",
      "kind": {
        "type": "ResourceOp",
        "resource": "custody-core:v1",
        "op": "Configure"
      },
      "needs": [],
      "after": ["d1"]
    }
  ]
}
```

### IDD (Information Dependency Diagram)

```json
{
  "schema": {
    "instructionMethod": {
      "type": "select",
      "required": true,
      "provenance": ["option:TradeCapture"]
    },
    "reporting-gaap": {
      "type": "string",
      "required": true,
      "provenance": ["choose-gaap"]
    }
  },
  "values": {
    "reporting-gaap": "IFRS"
  },
  "gaps": ["instructionMethod", "enabledMarkets"]
}
```

---

## 🎨 UI Features

### Panel Toggles

Use checkboxes in top-right to show/hide panels:
- ☑ YAML - Configuration editor
- ☑ Intent - Workflow parameters
- ☑ Output - Compilation results

### Status Indicators

- 🔄 **Spinner** - Loading/compiling/executing
- ✓ **Green** - Success
- ❌ **Red** - Errors

### Keyboard Navigation

- `Tab` - Navigate between fields
- `Ctrl+S` - Save current file (when modified)
- `Ctrl+Enter` - Compile workflow

---

## 🔐 Security

**Network:**
- CORS enabled on backend
- HTTP-only (localhost development)
- Production: Use HTTPS + authentication

**Data:**
- No secrets in YAML files
- Database credentials via environment
- Client-side validation

---

## 📝 Configuration Files

### Product Catalog Structure

```yaml
products:
  - id: <product-id>
    services:
      - serviceId: <service-name>
        options:
          - id: <option-id>
            prompt: <user-facing question>
            type: select | multiselect | text
            choices: [<option1>, <option2>]
            requiredForResources: [<resource-id>]
    resources:
      - type: <resource-type-id>
        implements: [<service-id>]
```

### CBU Template Structure

```yaml
cbuTemplates:
  - id: <template-id>
    description: <description>
    requiredRoles:
      - role: <role-name>
        entityTypeConstraint: [<entity-type>]
```

### Resource Dictionary Structure

```yaml
id: <resource-type-id>
version: <semver>
description: <description>
dictionary:
  attrs:
    - key: <attribute-name>
      type: <data-type>
      required: true | false
      default: <default-value>
```

---

## 🚧 Roadmap

- [ ] Real-time validation of YAML syntax
- [ ] Workflow execution visualization (task DAG)
- [ ] Historical execution logs
- [ ] Multi-user collaboration
- [ ] Workflow versioning & rollback
- [ ] Integration with external systems (Kafka, gRPC)

---

## 📚 Related Packages

- **onboarding** - Core workflow compiler library
- **onboarding-cli** - Command-line demo tool
- **grpc-server** - Backend HTTP/gRPC server
- **web-ui** - Editor/designer platform (CBU DSL, Resource DSL)

---

## 🤝 Contributing

This is part of the Data Designer workspace. See main project README.

**Development:**
1. Make changes to source files
2. Test with `./run-onboarding-desk.sh`
3. Verify WASM build with `./run-onboarding-wasm.sh`
4. Run tests: `cargo test --package onboarding-ui`

---

## 📄 License

See main project license.

---

## 🙏 Acknowledgments

Built with:
- **Rust** - Systems programming language
- **egui** - Immediate mode GUI framework
- **eframe** - egui application framework
- **wasm-bindgen** - Rust/WASM interop
- **reqwest** - HTTP client
- **serde** - Serialization framework

---

**🚀 Ready to onboard clients at scale!**
