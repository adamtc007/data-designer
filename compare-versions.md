# Data Designer - Web-First Architecture

> **Note**: This project has migrated from Tauri desktop to a **Web-First WASM Architecture**

## 🌐 Current Web-First Implementation

### Architecture Overview
```
Browser (Any Device)
    ↓ gRPC (Port 50051)
gRPC Microservices Server
    ↓ PostgreSQL
Database & Core Engine
```

### ✅ Current Features:
- **Pure Rust WASM Web Client**: Browser-native egui GUI (60fps)
- **gRPC Microservices**: Protocol Buffers API (900+ lines)
- **PostgreSQL Integration**: Vector embeddings with pgvector
- **Complete AI Assistant**: All 7 AI features with gRPC integration
- **Secure Key Management**: System keychain integration via gRPC
- **Financial Entity Management**: CBU, Product, Service, Resource, Workflow CRUD
- **White Truffle Architecture**: Advanced execution engine with orchestration
- **Hybrid Reliability**: gRPC-first with automatic database fallback

### 🚀 Quick Start
```bash
# One-command deployment
./runwasm.sh                   # Build + serve + open browser

# Manual steps
cd grpc-server && cargo run   # Start gRPC server (port 50051)
cd web-ui && ./build-web.sh   # Build WASM package
cd web-ui && ./serve-web.sh   # Serve on localhost:8080
```

### 📊 Benefits of Web-First Architecture
| Aspect | Web-First WASM | Previous Tauri |
|--------|----------------|----------------|
| **Universal Access** | ✅ Any device with browser | ❌ Platform-specific builds |
| **Zero Installation** | ✅ Instant access | ❌ Download & install required |
| **Performance** | ✅ 60fps, 12MB bundle | ✅ Native speed |
| **Deployment** | ✅ Static file serving | ❌ App store distribution |
| **Cross-Platform** | ✅ Universal compatibility | ⚠️ Multiple build targets |
| **Database Integration** | ✅ gRPC microservices | ✅ Direct connection |
| **Maintenance** | ✅ Single codebase | ❌ Desktop + web versions |

## 🎯 Why We Migrated

1. **Universal Accessibility** - Works on any device with a modern browser
2. **Simplified Deployment** - Single WASM bundle vs multiple platform builds
3. **Better Scalability** - Microservices architecture for enterprise use
4. **Easier Maintenance** - One UI codebase instead of desktop + web versions
5. **Enterprise Ready** - gRPC APIs suitable for financial services integration

## 🏗️ Architecture Components

### Web UI (`web-ui/`)
- Pure Rust WASM with egui framework
- Enhanced font rendering and professional styling
- Entity management and capability interfaces
- Professional code editor with syntax highlighting

### gRPC Server (`grpc-server/`)
- Financial taxonomy service with Protocol Buffers
- Complete CRUD operations for all entities
- AI key management and suggestion APIs
- Workflow orchestration endpoints

### Core Library (`data-designer-core/`)
- Capability execution engine with 10+ built-in capabilities
- PostgreSQL integration with vector embeddings
- Onboarding orchestration with complex dependency management
- DSL parser and evaluation engine

**Current Status**: ✅ **COMPLETED SYSTEM** - Production-ready web-first financial DSL platform