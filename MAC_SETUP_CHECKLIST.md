# Mac Setup Checklist for Claude Code

## Step 1: Verify Git Repository
```bash
cd ~/Development/data-designer
git fetch origin
git checkout feat/onboarding-library-integration
git pull origin feat/onboarding-library-integration
```

## Step 2: Check PostgreSQL
```bash
# Verify PostgreSQL is running
psql --version
pg_isready

# If not installed:
# brew install postgresql@15
# brew services start postgresql@15
```

## Step 3: Set Up Database
```bash
# Check if database exists
psql -lqt | cut -d \| -f 1 | grep -qw data_designer && echo "Database exists" || echo "Need to create database"

# Create database if needed
createdb data_designer

# Apply complete schema
psql data_designer -f database/schema.sql 2>&1 | tail -20

# Apply CBU views and data
psql data_designer -f fix_cbu_views.sql
psql data_designer -f fix_cbu_database.sql

# Generate test entities (optional)
psql data_designer -f database/generate_100_entities.sql
```

## Step 4: Verify Database Setup
```bash
# Check tables (should be ~21 tables)
psql data_designer -c '\dt' | wc -l

# Check views
psql data_designer -c '\dv'

# Verify CBU data (should show 8)
psql data_designer -c 'SELECT COUNT(*) FROM cbu;'

# List CBUs
psql data_designer -c 'SELECT cbu_id, cbu_name, status FROM cbu;'

# Check entities (should show 116: 16 base + 100 generated)
psql data_designer -c 'SELECT COUNT(*) FROM legal_entities;'

# Verify onboarding tables exist
psql data_designer -c '\dt onboarding*'
```

## Step 5: Set Environment Variable
```bash
# Set DATABASE_URL for your Mac username
export DATABASE_URL="postgresql:///data_designer?user=$USER"

# Add to shell profile for persistence
echo 'export DATABASE_URL="postgresql:///data_designer?user=$USER"' >> ~/.zshrc
```

## Step 6: Build and Test Desktop App (wgpu - Memory Leak Fix)
```bash
# Clean build
cargo clean

# Build
cargo build --release

# Run desktop with wgpu renderer
./rundesk.sh
```

**TEST FOR MEMORY LEAKS:**
- Open Activity Monitor (Cmd+Space, type "Activity Monitor")
- Find "data-designer-desktop" process
- Watch memory usage while using the app
- Create/edit CBUs, switch tabs, load entities
- Memory should stay stable (not climbing continuously)

## Step 7: Build and Test WASM/Browser Version
```bash
# Install wasm-pack if not present
which wasm-pack || cargo install wasm-pack

# Build and serve WASM
./runwasm.sh

# Opens browser to http://localhost:8000
# Test same operations as desktop
```

## Expected Results

### Desktop (wgpu renderer):
- ✅ No memory leaks (fixed from glow → wgpu migration)
- ✅ 60fps rendering
- ✅ Stable memory usage
- ✅ All CBU operations work
- ✅ Entity picker loads 116 entities fast

### WASM/Browser:
- ✅ Identical functionality to desktop
- ✅ Fast startup (~2 seconds)
- ✅ Smooth 60fps
- ✅ Full CBU DSL IDE features

## Common Issues

### Database Connection Failed
```bash
# Check connection
psql data_designer -c 'SELECT 1;'

# Fix permissions
sudo -u postgres psql -c "GRANT ALL ON DATABASE data_designer TO $USER;"
```

### Compilation Errors
```bash
# Update Rust
rustup update

# Check dependencies
cargo tree | grep -E "egui|wgpu|sqlx"
```

### Memory Still Leaking
```bash
# Verify wgpu is being used
grep -r "wgpu" web-ui/Cargo.toml

# Check for glow references (should be removed)
grep -r "glow" web-ui/Cargo.toml || echo "✓ No glow dependencies"
```

## Performance Comparison

**Mac (M1/M2/M3) Expected:**
- Cargo build: 5-10x faster than HP laptop
- Desktop app startup: < 1 second
- WASM build: 2-3x faster
- Incremental builds: sub-second

**vs HP Laptop:**
- HP: cargo build ~2 minutes
- Mac: cargo build ~10-20 seconds (estimated)

## Files to Check in Git

```bash
# Verify all database files are present
ls -la database/migrations/
ls -la database/schema.sql
ls -la fix_cbu_database.sql
ls -la fix_cbu_views.sql
ls -la database/generate_100_entities.sql

# Verify build scripts
ls -la rundesk.sh
ls -la runwasm.sh
ls -la web-ui/build-web.sh

# Check wgpu migration
git log --oneline | grep wgpu
# Should show: d5fbbf9 feat: migrate desktop renderer from glow to wgpu
```

## Success Criteria

✅ Database: 21 tables, 3 views, 8 CBUs, 116 entities
✅ Desktop: Launches without errors, no memory leaks
✅ WASM: Builds and runs in browser
✅ Performance: Fast compilation on Mac M-series chip
✅ Memory: Stable usage during extended use (wgpu fix verified)

---

**Ready to switch to Mac for development! 🚀**
