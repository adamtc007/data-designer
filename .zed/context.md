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
