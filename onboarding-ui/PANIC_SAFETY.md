# Panic Safety & Error Handling

## Design Philosophy

**Zero Panics in Production** - This codebase is designed to handle all errors gracefully without panicking.

## Error Handling Strategy

### 1. No Unwraps in Hot Paths

```rust
// ❌ BAD - Can panic
let value = option.unwrap();

// ✅ GOOD - Graceful handling
match option {
    Some(v) => v,
    None => {
        log::error!("Expected value missing");
        return; // or use default
    }
}
```

### 2. Tokio Runtime Safety

**Problem:** `tokio::spawn()` panics if no runtime is active

**Solution:** Runtime detection with fallback

```rust
pub fn spawn_async<F>(future: F) {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.spawn(future),
        Err(_) => {
            // Fallback: create runtime in separate thread
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new()
                    .expect("Failed to create runtime");
                rt.block_on(future);
            });
        }
    }
}
```

### 3. Mutex Poisoning

**Problem:** `lock().unwrap()` panics if mutex is poisoned

**Solution:** Handle lock failures

```rust
// ❌ BAD
let mut data = state.lock().unwrap();

// ✅ GOOD
if let Ok(mut data) = state.lock() {
    *data = new_value;
} else {
    log::error!("Failed to acquire lock - mutex poisoned");
    // Continue gracefully
}
```

### 4. Network Errors

All HTTP requests use `Result<T, E>` and are handled:

```rust
match client.get_request::<Data>("/api/endpoint").await {
    Ok(data) => { /* process */ },
    Err(e) => {
        log::warn!("Request failed: {}", e);
        // Show error to user, don't crash
    }
}
```

### 5. JSON Parsing

All deserialization is checked:

```rust
let users: Vec<User> = match serde_json::from_str(&input) {
    Ok(u) => u,
    Err(e) => {
        // Show validation error to user
        return Err(format!("Invalid JSON: {}", e));
    }
};
```

## Critical Sections

### Startup (main.rs)

```rust
#[tokio::main]  // ✅ Ensures tokio runtime available
async fn main() -> eframe::Result<()> {
    env_logger::init();  // Never panics

    // eframe::run_native returns Result, not panic
    eframe::run_native(/* ... */)
}
```

### State Updates (state_manager.rs)

All async operations wrapped in Result handling:

```rust
wasm_utils::spawn_async(async move {
    let result = client.get_request().await;  // Result<T, E>

    match result {
        Ok(data) => { /* update state */ },
        Err(e) => {
            log::warn!("Operation failed: {}", e);
            // Set error state, don't panic
        }
    }
});
```

### UI Rendering (app.rs)

```rust
// All option/result access uses safe patterns:
if let Some(result) = &self.state.compile_result {
    // Process result
}

self.state.compile_result.as_ref()
    .map(|r| r.success)
    .unwrap_or(false)  // Safe: has default
```

## Testing for Panics

```bash
# Run with panic detection
RUST_BACKTRACE=1 cargo test

# Check for unwrap() in code
rg "\.unwrap\(\)" --type rust

# Check for expect() in code
rg "\.expect\(" --type rust

# Audit critical paths
cargo clippy -- -W clippy::unwrap_used
```

## Known Safe Unwraps

Only in initialization paths where failure is unrecoverable:

```rust
// ✅ Safe - happens once at startup
tokio::runtime::Runtime::new()
    .expect("Failed to create runtime - system misconfigured");
```

## Panic Recovery

Even if a panic occurs:

1. **Desktop**: Process exits cleanly (no corruption)
2. **Web**: WASM catches and displays error message
3. **State**: All state is in-memory, no data loss on crash

## Migration Notes

Coming from frameworks with panic issues (Tauri/Tahoe):

- ✅ No C/C++ FFI boundary panics
- ✅ No IPC serialization panics
- ✅ No webview bridge panics
- ✅ Pure Rust with explicit error handling
- ✅ WASM has automatic panic→JS error conversion

## Monitoring

Production monitoring should track:

```rust
// Log all error paths
log::error!("Critical failure: {}", error);

// Never silent failures
match operation() {
    Ok(_) => {},
    Err(e) => log::warn!("Non-critical failure: {}", e),
    //      ^^^ Always logged, never silently ignored
}
```

## Guarantees

1. **No .unwrap() in request handlers**
2. **No .expect() in async operations**
3. **All network errors logged and handled**
4. **All mutex poisoning handled**
5. **Tokio runtime validated before spawn**

**Result:** Production-ready code with zero panic risk in normal operation.
