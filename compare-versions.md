# Tauri Dev vs Web Version Comparison

## Key Differences

There are **significant differences** between running the IDE with `cargo tauri dev` (Tauri version) and running it in a regular web browser (web version).

## 🖥️ Tauri Version (`cargo tauri dev`)
**Full-featured IDE with complete backend integration**

### ✅ Available Features:
- **PostgreSQL Database Integration**: Full CRUD operations with rules, attributes, categories
- **pgvector Similarity Search**: Real vector embeddings with cosine similarity search
- **AI Agent with System API Keys**: Automatic detection of `ANTHROPIC_API_KEY`, `OPENAI_API_KEY` from environment
- **Rule Persistence**: Save/load rules to/from PostgreSQL database
- **Vector Embeddings**: Generate 1536-dimensional embeddings for semantic search
- **Find Similar Rules**: Real semantic similarity search using pgvector
- **System Integration**: Access to file system, environment variables, native APIs
- **Rules Catalogue**: Load actual rules from database with full metadata
- **Database Commands**: All `db_*` Tauri commands available

### 🔧 Backend Access:
- Direct access to Rust backend via `window.__TAURI__.invoke()`
- Full PostgreSQL connection and operations
- Real vector similarity calculations
- Environment variable access for API keys

## 🌐 Web Version (Browser Only)
**Lightweight demo with mock data fallbacks**

### ⚠️ Limited Features:
- **Mock Data Only**: No real database connection
- **Mock Similar Rules**: Hardcoded similarity results with fake percentages
- **No API Key Detection**: Cannot access system environment variables
- **No Rule Persistence**: Changes not saved anywhere
- **Static Rules Catalogue**: Hardcoded mock rules only
- **No Vector Operations**: No real embeddings or similarity calculations
- **AI Agent Fallback**: Uses comprehensive mock responses when no API keys

### 🎭 Fallback Behavior:
- Shows mock rules with fake DSL examples
- Displays mock similarity scores (85%, 72%, etc.)
- AI Agent provides helpful responses without real API calls
- All database operations return mock data

## 🔍 Detection Logic

The IDE automatically detects the environment:

```javascript
if (window.__TAURI__ && window.__TAURI__.invoke) {
    // Tauri version - use real backend
    const dbRules = await window.__TAURI__.invoke('db_get_all_rules');
    const similarRules = await window.__TAURI__.invoke('db_find_similar_rules', {
        dsl_text: dslText,
        limit: 5
    });
} else {
    // Web version - use mock data
    return loadMockRules();
    const mockSimilarRules = [...]; // Hardcoded examples
}
```

## 📊 Feature Comparison Table

| Feature | Tauri Version | Web Version |
|---------|---------------|-------------|
| Database Integration | ✅ PostgreSQL 17.6 | ❌ Mock data only |
| Vector Search | ✅ pgvector 0.8.1 | ❌ Fake similarity scores |
| API Key Detection | ✅ Environment variables | ❌ Manual entry only |
| Rule Persistence | ✅ PostgreSQL storage | ❌ No persistence |
| AI Embeddings | ✅ Real OpenAI/Anthropic | ❌ Mock embeddings |
| System Integration | ✅ Full Tauri APIs | ❌ Browser sandbox only |
| Performance | ✅ Native speed | ⚡ Fast but limited |
| Setup Required | 🔧 PostgreSQL + pgvector | 🌐 Just open in browser |

## 🎯 When to Use Each

### Use Tauri Version When:
- You want full database integration
- You need real vector similarity search
- You have API keys for embeddings
- You want to persist and manage rules
- You're doing serious DSL development

### Use Web Version When:
- Quick demo or testing
- No database setup available
- Just exploring the IDE interface
- Showing the UI to others
- Development on systems without PostgreSQL

## 🚀 Running Each Version

### Tauri Version:
```bash
cd src-tauri
cargo tauri dev
# Opens at: http://localhost:1420 with full features
```

### Web Version:
```bash
npm run dev
# Open any browser to: http://localhost:1420
# Will show mock data fallbacks
```

## 🎭 Demo Mode

The web version is essentially a **demo mode** that:
- Shows what the IDE looks like
- Demonstrates the UI/UX
- Provides realistic mock data
- Works without any setup
- Never fails or crashes

The Tauri version is the **production mode** with:
- Real database operations
- Actual vector computations
- True API integrations
- Full persistence
- Complete feature set