# Onboarding WASM UI Refactor Plan

## Current Problem

The egui UI has become unmaintainable due to:
- **Point-to-point coupling**: UI widgets directly calling backend gRPC/HTTP endpoints
- **Scattered state management**: State logic mixed throughout UI code
- **Confusion for Claude Code**: No clear separation of concerns makes AI assistance difficult
- **Technical debt**: UI logic, business logic, and backend calls are tightly coupled

## Solution: Clean State Manager Pattern

### Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                   egui UI Layer                     │
│              (Pure Rendering Only)                  │
│                                                     │
│  • Reads DslState (immutable)                       │
│  • Renders UI based on state                        │
│  • Dispatches actions/commands to manager           │
│  • NO direct backend calls                          │
└──────────────────┬──────────────────────────────────┘
                   │
                   │ Actions/Commands (synchronous)
                   ↓
┌─────────────────────────────────────────────────────┐
│          OnboardingManager (WASM)                   │
│         (Single Source of Truth)                    │
│                                                     │
│  • Owns current DslState                            │
│  • Queues actions from UI                           │
│  • Processes actions asynchronously                 │
│  • Makes all gRPC/HTTP calls internally             │
│  • Updates DslState                                 │
│  • Returns new state for next frame                 │
└──────────────────┬──────────────────────────────────┘
                   │
                   │ gRPC/HTTP/REST
                   ↓
┌─────────────────────────────────────────────────────┐
│              Backend Services                        │
│     (Existing - DO NOT MODIFY)                      │
│                                                     │
│  • gRPC microservices (port 50051)                  │
│  • HTTP REST APIs (port 8080)                       │
│  • PostgreSQL database                              │
└─────────────────────────────────────────────────────┘
```

## Core Components

### 1. DslState - What Gets Rendered

The complete onboarding state that the UI needs to render.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslState {
    // Current workflow position
    pub current_step: OnboardingStep,
    
    // Client information
    pub client_data: Option<ClientData>,
    
    // KYC documents and status
    pub kyc_documents: Vec<Document>,
    pub kyc_status: KycStatus,
    
    // Risk assessment data
    pub risk_score: Option<u32>,
    pub risk_rating: Option<RiskRating>,
    
    // Workflow control
    pub can_proceed: bool,
    pub can_go_back: bool,
    
    // Validation and errors
    pub validation_errors: Vec<String>,
    pub warnings: Vec<String>,
    
    // UI state
    pub is_loading: bool,
    pub progress_percentage: u8,
}
```

### 2. OnboardingStep - Workflow Steps

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OnboardingStep {
    ClientInfo,
    KycDocuments,
    RiskAssessment,
    AccountSetup,
    Review,
    Complete,
}
```

### 3. OnboardingAction - UI Commands

All possible actions the UI can trigger.

```rust
#[derive(Debug, Clone)]
pub enum OnboardingAction {
    // Workflow control
    StartOnboarding { client_id: String },
    AdvanceStep,
    GoBackStep,
    JumpToStep(OnboardingStep),
    Reset,
    
    // Data updates
    UpdateClientInfo { data: ClientData },
    UploadDocument { doc: Document },
    RemoveDocument { doc_id: String },
    SubmitKyc,
    
    // Risk assessment
    CalculateRisk,
    OverrideRiskScore { score: u32, reason: String },
    
    // Account setup
    ConfigureAccount { config: AccountConfig },
    FinalizeOnboarding,
}
```

### 4. OnboardingManager - Single Source of Truth

The brain of the application. Handles all state transitions.

```rust
pub struct OnboardingManager {
    // Current state
    state: DslState,
    
    // Backend client
    grpc_client: OnboardingClient,
    
    // Action queue
    pending_actions: VecDeque<OnboardingAction>,
}

impl OnboardingManager {
    pub fn new(grpc_client: OnboardingClient) -> Self {
        Self {
            state: DslState::default(),
            grpc_client,
            pending_actions: VecDeque::new(),
        }
    }
    
    /// UI calls this - synchronous, just queues the action
    pub fn dispatch(&mut self, action: OnboardingAction) {
        self.pending_actions.push_back(action);
    }
    
    /// Call this each frame - processes one action from queue
    pub async fn update(&mut self) {
        if let Some(action) = self.pending_actions.pop_front() {
            self.handle_action(action).await;
        }
    }
    
    /// UI reads this - immutable reference
    pub fn state(&self) -> &DslState {
        &self.state
    }
    
    /// Internal - processes actions and updates state
    async fn handle_action(&mut self, action: OnboardingAction) {
        self.state.is_loading = true;
        
        match action {
            OnboardingAction::StartOnboarding { client_id } => {
                match self.grpc_client.start_onboarding(&client_id).await {
                    Ok(response) => {
                        self.state = response.into_dsl_state();
                    }
                    Err(e) => {
                        self.state.validation_errors.push(e.to_string());
                    }
                }
            }
            OnboardingAction::SubmitKyc => {
                match self.grpc_client.submit_kyc(&self.state).await {
                    Ok(response) => {
                        self.state.kyc_status = response.status;
                        self.state.current_step = OnboardingStep::RiskAssessment;
                    }
                    Err(e) => {
                        self.state.validation_errors.push(e.to_string());
                    }
                }
            }
            // ... handle all other actions
            _ => {}
        }
        
        self.state.is_loading = false;
    }
}
```

## egui Application Structure

### Simple Top-Level App

```rust
// src/app.rs

pub struct OnboardingApp {
    manager: OnboardingManager,
}

impl OnboardingApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let grpc_client = OnboardingClient::new("http://localhost:50051");
        
        Self {
            manager: OnboardingManager::new(grpc_client),
        }
    }
}

impl eframe::App for OnboardingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process pending actions asynchronously
        let manager = &mut self.manager;
        ctx.spawn(async move {
            manager.update().await;
        });
        
        // Render UI based on current state
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render(ui);
        });
        
        // Request repaint if actions are pending
        if self.manager.has_pending_actions() {
            ctx.request_repaint();
        }
    }
}

impl OnboardingApp {
    fn render(&mut self, ui: &mut egui::Ui) {
        let state = self.manager.state();
        
        // Show loading indicator if processing
        if state.is_loading {
            ui.spinner();
        }
        
        // Render based on current step
        match state.current_step {
            OnboardingStep::ClientInfo => {
                ui::client_info::render(ui, state, &mut self.manager);
            }
            OnboardingStep::KycDocuments => {
                ui::kyc_docs::render(ui, state, &mut self.manager);
            }
            OnboardingStep::RiskAssessment => {
                ui::risk::render(ui, state, &mut self.manager);
            }
            OnboardingStep::AccountSetup => {
                ui::account_setup::render(ui, state, &mut self.manager);
            }
            OnboardingStep::Review => {
                ui::review::render(ui, state, &mut self.manager);
            }
            OnboardingStep::Complete => {
                ui::complete::render(ui, state, &mut self.manager);
            }
        }
        
        // Show errors at bottom
        if !state.validation_errors.is_empty() {
            ui.separator();
            ui.colored_label(egui::Color32::RED, "Errors:");
            for error in &state.validation_errors {
                ui.label(error);
            }
        }
    }
}
```

### Pure UI Components

```rust
// src/ui/client_info.rs

pub fn render(ui: &mut egui::Ui, state: &DslState, manager: &mut OnboardingManager) {
    ui.heading("Client Information");
    
    // Pure rendering - just read state and dispatch actions
    if let Some(client) = &state.client_data {
        ui.label(format!("Name: {}", client.name));
        ui.label(format!("Email: {}", client.email));
        ui.label(format!("LEI: {}", client.lei_code));
    } else {
        ui.label("No client data available");
    }
    
    ui.add_space(10.0);
    
    // Navigation
    ui.horizontal(|ui| {
        if ui.button("Back").clicked() {
            manager.dispatch(OnboardingAction::GoBackStep);
        }
        
        if state.can_proceed {
            if ui.button("Next →").clicked() {
                manager.dispatch(OnboardingAction::AdvanceStep);
            }
        }
    });
}
```

## Directory Structure

```
onboarding-wasm/
├── Cargo.toml
├── index.html
└── src/
    ├── lib.rs                  # WASM entry point
    ├── app.rs                  # OnboardingApp - top level
    │
    ├── onboarding/
    │   ├── mod.rs              # Re-exports
    │   ├── manager.rs          # OnboardingManager implementation
    │   ├── state.rs            # DslState, OnboardingStep
    │   ├── actions.rs          # OnboardingAction enum
    │   └── client.rs           # gRPC client wrapper
    │
    └── ui/
        ├── mod.rs              # Re-exports
        ├── client_info.rs      # Client info step UI
        ├── kyc_docs.rs         # KYC documents step UI
        ├── risk.rs             # Risk assessment step UI
        ├── account_setup.rs    # Account setup step UI
        ├── review.rs           # Review step UI
        └── complete.rs         # Completion step UI
```

## Critical Rules

### ✅ DO

1. **UI components are pure render functions** - They only read state and dispatch actions
2. **All backend calls go through OnboardingManager** - UI never touches gRPC/HTTP directly
3. **State is immutable from UI perspective** - UI gets `&DslState`, never `&mut`
4. **Actions are simple data** - No closures, no complex logic in action types
5. **Manager handles all async** - UI stays synchronous and simple
6. **One action per frame** - Process actions sequentially for predictable behavior

### ❌ DON'T

1. **Never call backend from UI** - No gRPC, HTTP, or database calls in render code
2. **Never mutate state directly** - All changes through manager.dispatch()
3. **Never put business logic in UI** - Validation, calculations, state transitions belong in manager
4. **Never create complex action chains** - Keep actions atomic
5. **Never block the UI thread** - All I/O happens in manager.update()

## Implementation Steps

### Phase 1: Core Types (Start Here)
1. Create `src/onboarding/state.rs` with `DslState` and `OnboardingStep`
2. Create `src/onboarding/actions.rs` with `OnboardingAction` enum
3. Define the data structures (ClientData, Document, etc.)

### Phase 2: Manager Implementation
1. Create `src/onboarding/manager.rs` with `OnboardingManager`
2. Implement `dispatch()`, `update()`, `state()` methods
3. Stub out action handlers (return mock data initially)

### Phase 3: gRPC Client Wrapper
1. Create `src/onboarding/client.rs`
2. Wrap existing gRPC client with clean async interface
3. Convert gRPC responses to DslState

### Phase 4: Simple App Shell
1. Create `src/app.rs` with `OnboardingApp`
2. Wire up manager
3. Create basic render loop

### Phase 5: UI Components (One at a time)
1. Start with `src/ui/client_info.rs`
2. Create pure render function
3. Test with mock data
4. Repeat for each step

### Phase 6: Integration
1. Connect real gRPC client
2. Test end-to-end flow
3. Add error handling
4. Polish UI

## Testing Strategy

### Unit Tests
- Test OnboardingManager state transitions with mock client
- Test action handling logic
- Test state validation

### Integration Tests
- Test UI components with mock manager
- Test manager with mock gRPC client
- Test full flow with test database

### Manual Testing
- Use `./runobd.sh` to run WASM app
- Test each onboarding step
- Verify backend integration

## Migration Notes

### What to Keep
- All existing backend services (gRPC, HTTP, DB)
- Protocol Buffers definitions
- Business logic on backend
- Database schemas

### What to Replace
- Entire egui UI implementation
- State management code
- Direct backend calls from UI
- Scattered action handlers

### What to Reuse
- gRPC client connection code (wrap it)
- Data type definitions (adapt to DslState)
- Backend API contracts (keep unchanged)

## Success Criteria

- [ ] UI has ZERO direct backend calls
- [ ] All state changes flow through OnboardingManager
- [ ] UI components are pure render functions
- [ ] Claude Code can understand the architecture
- [ ] Adding new features requires changes in only 1-2 files
- [ ] State transitions are predictable and testable
- [ ] No more spaghetti code

## Next Steps

1. **Create this file in repo root**: `REFACTOR_PLAN.md`
2. **Tell Claude Code**: "Read REFACTOR_PLAN.md and implement Phase 1"
3. **Iterate**: Build one phase at a time
4. **Test continuously**: Use `./runobd.sh` to verify
5. **Keep backend unchanged**: This is UI-only refactor

---

**Remember**: The goal is SEPARATION OF CONCERNS. UI renders, Manager manages, Backend serves.Good question! Here are your best options for communicating this architecture to Claude Code (terminal):

## Option 1: Create a Design Document (Recommended)

Create a file in your repo that Claude Code can read:

```bash
# In your repo root
touch REFACTOR_PLAN.md
```

Then paste this content:

```markdown
# Onboarding WASM UI Refactor Plan

## Problem
Current egui UI has point-to-point coupling between UI and backend calls.
Claude Code gets confused due to spaghetti architecture.

## Solution: Clean State Manager Pattern

### Architecture
```
egui UI (dumb renderer)
    ↓ (dispatch actions)
OnboardingManager (smart state machine in WASM)
    ↓ (gRPC/HTTP calls)
Backend Services (existing - DO NOT TOUCH)
```

### Core Components

1. **DslState** - What gets rendered
   - current_step: OnboardingStep
   - client_data, kyc_docs, risk_score, etc.
   - validation_errors
   - can_proceed flag

2. **OnboardingAction** - UI commands
   - StartOnboarding, UpdateClientInfo, SubmitKyc, etc.

3. **OnboardingManager** - Single source of truth
   - Owns DslState
   - dispatch(action) - UI calls this
   - update() - Processes actions async
   - state() - UI reads this

### Rules
- UI NEVER calls backend directly
- ALL state changes go through OnboardingManager
- Manager returns new DslState each frame
- UI is pure rendering based on DslState

### Directory Structure
```
onboarding-wasm/
├── src/
│   ├── lib.rs              # WASM entry
│   ├── app.rs              # OnboardingApp
│   ├── onboarding/
│   │   ├── mod.rs          # Manager, State, Actions
│   │   ├── client.rs       # gRPC wrapper
│   │   └── state.rs        # DslState
│   └── ui/
│       ├── client_info.rs
│       ├── kyc_docs.rs
│       └── risk.rs
```

## Implementation Steps
1. Define DslState, OnboardingAction, OnboardingStep enums
2. Implement OnboardingManager with dispatch/update/state methods
3. Create simple OnboardingApp that only calls manager
4. Build UI components as pure render functions
```

Then in terminal:
```bash
claude-code "Read REFACTOR_PLAN.md and help me implement this clean architecture for the onboarding WASM UI. Start by creating the core types in src/onboarding/mod.rs"
```

## Option 2: Paste in Terminal Directly

Just paste the key architecture into the terminal chat:

```bash
claude-code "I'm refactoring the onboarding egui UI. New architecture:

1. OnboardingManager in WASM owns all state
2. UI dispatches actions to manager
3. Manager makes backend calls, returns DslState
4. UI renders based on DslState only

Create src/onboarding/mod.rs with:
- DslState struct (current_step, client_data, errors)
- OnboardingAction enum (StartOnboarding, SubmitKyc, etc)
- OnboardingManager with dispatch(), update(), state() methods

Keep all existing gRPC/DB/REST unchanged."
```

## Option 3: Use Conversation Memory

Since you've already discussed this architecture with me, you can reference it:

```bash
claude-code "Check my recent web chat where we designed a clean state manager pattern for the onboarding UI refactor. Implement that architecture starting with the core types."
```

Claude Code can access your recent conversations and will see this discussion.

## Option 4: Create Example Code File

Create a reference implementation file:

```bash
# Create a template/reference file
mkdir -p docs/refactor
touch docs/refactor/architecture_reference.rs
```

Put the example code structures in there, then:

```bash
claude-code "Follow the architecture pattern in docs/refactor/architecture_reference.rs to rebuild the onboarding WASM UI"
```

## My Recommendation

**Use Option 1 (Design Doc) + Option 3 (Reference this chat)**

1. Create `REFACTOR_PLAN.md` with the high-level architecture
2. In terminal: `claude-code "Reference my recent web chat about onboarding UI refactor. Read REFACTOR_PLAN.md and implement step 1: create the core types"`

This gives Claude Code both the written specification AND the context from our discussion.

Want me to generate the complete `REFACTOR_PLAN.md` content ready to paste?
