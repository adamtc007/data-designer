# Database Setup Guide for Mac

## Prerequisites

Ensure PostgreSQL is installed and running on your Mac:
```bash
brew install postgresql@15
brew services start postgresql@15
```

## Quick Setup (Recommended)

```bash
# 1. Clone/Pull the repository
git checkout feat/onboarding-library-integration
git pull

# 2. Run automated setup
cd ~/Development/data-designer
./database/setup.sh

# 3. Apply CBU fixes and populate data
psql data_designer -f fix_cbu_views.sql
psql data_designer -f fix_cbu_database.sql

# 4. (Optional) Generate 100 test entities
psql data_designer -f database/generate_100_entities.sql

# 5. Verify setup
psql data_designer -c '\dt'
psql data_designer -c 'SELECT COUNT(*) FROM cbu;'
psql data_designer -c 'SELECT cbu_id, cbu_name FROM cbu;'
```

## Manual Setup (Step-by-Step)

### 1. Create Database
```bash
createdb data_designer
```

### 2. Apply Complete Schema
```bash
psql data_designer -f database/schema.sql
```

### 3. Or Apply Incremental Migrations
```bash
cd ~/Development/data-designer
psql data_designer -f database/migrations/001_compiled_code_storage.sql
psql data_designer -f database/migrations/002_config_driven_ui.sql
psql data_designer -f database/migrations/002_fix_data_dictionary_view.sql
psql data_designer -f database/migrations/003_add_cbu_system.sql
psql data_designer -f database/migrations/004_add_product_services_resources.sql
psql data_designer -f database/migrations/004_grammar_storage.sql
psql data_designer -f database/migrations/006_cleanup_unused_tables.sql
psql data_designer -f database/migrations/007_add_missing_grpc_tables.sql
psql data_designer -f database/migrations/007_onboarding_requests.sql
```

### 4. Create Views and Populate CBU Data
```bash
psql data_designer -f fix_cbu_views.sql
psql data_designer -f fix_cbu_database.sql
```

### 5. Generate Test Entities (Optional)
```bash
psql data_designer -f database/generate_100_entities.sql
```

### 6. Load Sample Data (Optional)
```bash
psql data_designer -f database/init-sample-data.sql
```

## Verification

### Check Tables
```bash
psql data_designer -c '\dt'
```

Expected tables (21 total after migration 006 cleanup):
- client_business_units
- cbu_members
- cbu_roles
- legal_entities
- onboarding_request (NEW)
- onboarding_request_dsl (NEW)
- onboarding_request_history (NEW)
- onboarding_dsl_validation_errors (NEW)
- product_options
- resource_templates
- resource_instances
- + 10 more

### Check Views
```bash
psql data_designer -c '\dv'
```

Expected views:
- cbu
- cbu_investment_mandate_structure
- cbu_member_investment_roles

### Check Data
```bash
# Should show 8 CBUs
psql data_designer -c 'SELECT COUNT(*) FROM cbu;'

# List all CBUs
psql data_designer -c 'SELECT cbu_id, cbu_name, status FROM cbu;'

# Check entities (16 base + 100 generated = 116 total)
psql data_designer -c 'SELECT COUNT(*) FROM legal_entities;'

# Check onboarding requests (should be 0 initially)
psql data_designer -c 'SELECT COUNT(*) FROM onboarding_request;'
```

## Database Configuration

### Connection String
The application uses this connection string by default:
```
postgresql:///data_designer?user=adamtc007
```

On Mac, update to your username:
```bash
export DATABASE_URL="postgresql:///data_designer?user=$USER"
```

Or add to your `~/.zshrc` or `~/.bash_profile`:
```bash
echo 'export DATABASE_URL="postgresql:///data_designer?user=$USER"' >> ~/.zshrc
source ~/.zshrc
```

## Troubleshooting

### Permission Denied
```bash
# Create the database as your user
createdb data_designer

# Or use postgres superuser
sudo -u postgres createdb data_designer
sudo -u postgres psql -c "GRANT ALL ON DATABASE data_designer TO $USER;"
```

### pgvector Extension Missing
```bash
brew install pgvector
psql data_designer -c 'CREATE EXTENSION IF NOT EXISTS vector;'
```

### Tables Already Exist
If you need to reset:
```bash
dropdb data_designer
createdb data_designer
./database/setup.sh
```

## Testing the Setup

### Start the Desktop App
```bash
./rundesk.sh
```

### Start the Onboarding UI
```bash
./runobd.sh
```

The applications will connect to PostgreSQL and should show:
- 8 CBUs in the CBU picker
- 116 legal entities in the entity picker
- Ability to create onboarding requests

## Files in Git

All schema and data files are now in git:

**Migrations:**
- database/migrations/001-007_*.sql

**Schema:**
- database/schema.sql
- database/schema-simple.sql

**Setup:**
- database/setup.sh

**Data:**
- fix_cbu_database.sql (8 CBUs + members)
- fix_cbu_views.sql (views + legal_entities table)
- database/generate_100_entities.sql (100 test entities)
- database/init-sample-data.sql (additional samples)

