# Data Designer Database Recreation Guide

Complete guide to recreate the Data Designer PostgreSQL database from scratch using the exported metadata, DDL, migrations, and seed data.

## 📁 Directory Contents

```
db_metadata/
├── 📊 SCHEMA METADATA
│   ├── schema_ddl.sql          # Complete DDL export (11,932 lines)
│   ├── tables.csv              # 95 tables metadata
│   ├── columns.csv             # 1,742 columns with types
│   ├── indexes.csv             # 373 indexes
│   ├── constraints.csv         # 614 constraints (PK/FK/CHECK)
│   ├── views.csv               # 23 views
│   ├── functions.csv           # 155 functions/procedures
│   ├── metadata.json           # Machine-readable statistics
│   └── database_summary.md     # Human-readable overview
│
├── 🔄 MIGRATIONS
│   ├── 001_compiled_code_storage.sql
│   ├── 002_config_driven_ui.sql
│   ├── 003_add_cbu_system.sql
│   ├── 004_add_product_services_resources.sql
│   ├── 009_resource_capabilities_system.sql
│   ├── 010_enhanced_onboarding_workflows.sql
│   └── 011_test_data_seeding.sql
│
├── 🌱 SEED DATA
│   ├── init-sample-data.sql
│   ├── create_sample_investment_mandates.sql
│   ├── populate_actual_attributes.sql
│   ├── populate_corrected_attributes.sql
│   ├── populate_extended_attributes.sql
│   ├── populate_financial_services_taxonomy.sql
│   └── populate_investment_mandates_with_roles.sql
│
└── 🚀 SETUP TOOLS
    ├── setup_database.sh       # Automated setup script
    └── DATABASE_RECREATION.md  # This guide
```

## 🚀 Quick Setup (Automated)

### Prerequisites
- PostgreSQL 17+ server running
- pgvector extension available
- User with CREATE DATABASE privileges

### One-Command Setup
```bash
cd db_metadata
./setup_database.sh [db_name] [db_user] [db_host] [db_port]

# Examples:
./setup_database.sh                          # Uses defaults
./setup_database.sh my_data_designer         # Custom DB name
./setup_database.sh data_designer myuser     # Custom DB and user
```

### Default Configuration
- **Database**: `data_designer`
- **User**: Current system user
- **Host**: `localhost`
- **Port**: `5432`

## 🔧 Manual Setup (Step by Step)

### Step 1: Create Database
```bash
# Connect to PostgreSQL
psql -d postgres

# Create database
CREATE DATABASE data_designer;

# Connect to new database
\c data_designer

# Install required extensions
CREATE EXTENSION IF NOT EXISTS vector;        -- For vector embeddings
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";   -- For UUID generation
CREATE EXTENSION IF NOT EXISTS pg_trgm;       -- For text search
```

### Step 2: Apply Schema DDL
```bash
# Apply complete schema
psql -d data_designer -f schema_ddl.sql
```

### Step 3: Apply Migrations (In Order)
```bash
# Apply migrations in sequence
psql -d data_designer -f migrations/001_compiled_code_storage.sql
psql -d data_designer -f migrations/002_config_driven_ui.sql
psql -d data_designer -f migrations/003_add_cbu_system.sql
psql -d data_designer -f migrations/004_add_product_services_resources.sql
psql -d data_designer -f migrations/009_resource_capabilities_system.sql
psql -d data_designer -f migrations/010_enhanced_onboarding_workflows.sql
psql -d data_designer -f migrations/011_test_data_seeding.sql
```

### Step 4: Load Seed Data
```bash
# Load all seed data files
psql -d data_designer -f seed_data/init-sample-data.sql
psql -d data_designer -f seed_data/populate_financial_services_taxonomy.sql
psql -d data_designer -f seed_data/populate_extended_attributes.sql
psql -d data_designer -f seed_data/populate_actual_attributes.sql
psql -d data_designer -f seed_data/create_sample_investment_mandates.sql
psql -d data_designer -f seed_data/populate_investment_mandates_with_roles.sql
```

### Step 5: Verify Setup
```bash
# Check table count
psql -d data_designer -c "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public';"

# Check sample data
psql -d data_designer -c "SELECT count(*) FROM client_business_units;"
psql -d data_designer -c "SELECT count(*) FROM products;"
psql -d data_designer -c "SELECT count(*) FROM services;"
```

## 🏗️ Database Architecture Overview

### Core Financial Entities
- **`client_business_units`** - CBU organization and member roles
- **`products`** - Financial product catalog with line of business
- **`services`** - Service lifecycle descriptions
- **`resource_objects`** - Capability implementations
- **`investment_mandates`** - Investment management workflows

### DSL & Capability System
- **`workflow_templates`** - DSL workflow definitions
- **`capability_definitions`** - Execution capabilities
- **`resource_templates`** - Template-driven workflows
- **`onboarding_workflows`** - Complex workflow orchestration

### AI & Vector System
- **`embeddings`** - Vector embeddings for semantic search
- **`attribute_objects`** - Enhanced metadata for AI systems
- **`rules`** - DSL rules and expressions
- **`grammar_rules`** - Dynamic grammar definitions

### Configuration & UI
- **`ui_configurations`** - Multi-layered Resource Dictionary
- **`attribute_sets`** - Configuration-driven UI components
- **`data_dictionary`** - Enhanced type system with metadata

## 🔍 Verification Queries

### Database Health Check
```sql
-- Table count by schema
SELECT schemaname, count(*) as table_count
FROM pg_tables
WHERE schemaname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
GROUP BY schemaname;

-- Extension status
SELECT extname, extversion FROM pg_extension;

-- Sample data verification
SELECT 'CBUs' as entity, count(*) as records FROM client_business_units
UNION ALL
SELECT 'Products', count(*) FROM products
UNION ALL
SELECT 'Services', count(*) FROM services
UNION ALL
SELECT 'Resources', count(*) FROM resource_objects
UNION ALL
SELECT 'Mandates', count(*) FROM investment_mandates;
```

### Data Completeness Check
```sql
-- Investment mandate relationships
SELECT
    im.mandate_name,
    cbu.cbu_name,
    COUNT(mi.instrument_id) as instrument_count
FROM investment_mandates im
JOIN client_business_units cbu ON im.cbu_id = cbu.cbu_id
LEFT JOIN mandate_instruments mi ON im.mandate_id = mi.mandate_id
GROUP BY im.mandate_name, cbu.cbu_name
ORDER BY instrument_count DESC;

-- Product-Service-Resource chain
SELECT
    p.product_name,
    COUNT(DISTINCT ps.service_id) as service_count,
    COUNT(DISTINCT sr.resource_id) as resource_count
FROM products p
LEFT JOIN product_services ps ON p.product_id = ps.product_id
LEFT JOIN service_resources sr ON ps.service_id = sr.service_id
GROUP BY p.product_name
ORDER BY service_count DESC, resource_count DESC;
```

## 🚀 Application Integration

### Environment Variables
```bash
# Set database connection for Data Designer application
export DATABASE_URL="postgresql://username@localhost:5432/data_designer"

# For gRPC server
cd grpc-server && cargo run

# For WASM web UI
cd web-ui && ./build-web.sh
```

### Connection String Format
```
postgresql://[user]:[password]@[host]:[port]/[database]
postgresql://adamtc007@localhost:5432/data_designer
```

## 🔒 Security Considerations

### Database Permissions
```sql
-- Create application user (optional)
CREATE USER data_designer_app WITH PASSWORD 'secure_password';

-- Grant necessary permissions
GRANT CONNECT ON DATABASE data_designer TO data_designer_app;
GRANT USAGE ON SCHEMA public TO data_designer_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO data_designer_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO data_designer_app;
```

### Row-Level Security (If Applicable)
The database includes RLS policies for certain tables. Check `policies.csv` for details.

## 🛠️ Troubleshooting

### Common Issues

**Extension Missing**
```bash
# Install pgvector on macOS
cd /tmp
git clone --branch v0.8.1 https://github.com/pgvector/pgvector.git
cd pgvector
PG_CONFIG=/opt/homebrew/opt/postgresql@17/bin/pg_config make
PG_CONFIG=/opt/homebrew/opt/postgresql@17/bin/pg_config make install
```

**Permission Denied**
```sql
-- Grant superuser temporarily (if needed)
ALTER USER your_user SUPERUSER;
-- Remember to revoke after setup
ALTER USER your_user NOSUPERUSER;
```

**Migration Conflicts**
- Run migrations in the exact order specified
- Check for any custom modifications in your environment
- Some migrations may be cumulative and safe to re-run

### Verification Commands
```bash
# Check PostgreSQL is running
pg_isready -h localhost -p 5432

# Check database exists
psql -l | grep data_designer

# Check table structure matches
psql -d data_designer -c "\dt"
```

## 📊 Expected Results

After successful setup, you should have:
- **95+ tables** with complete schema
- **Sample CBUs** - 5 representative business units
- **Product catalog** - Financial products across major business lines
- **Investment mandates** - Fund accounting workflows
- **Vector embeddings** - Semantic search capability
- **Complete DSL system** - Templates and capabilities

The database will be ready for the **Data Designer Web-First application** with full functionality including AI assistance, gRPC microservices, and WASM web client.

---

**Data Designer Database Recreation** - Everything needed to rebuild the complete financial DSL platform database.