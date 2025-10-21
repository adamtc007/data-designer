# PostgreSQL Database Metadata Exporter

A comprehensive tool for exporting PostgreSQL database schema and metadata to CSV files, enabling external agents and tools to understand database structure without requiring direct database access.

## 🎯 Purpose

This tool is specifically designed to help **other agents/tools analyze the Data Designer database** when they don't have direct PostgreSQL connectivity. The exported metadata provides complete schema understanding for:

- 🤖 **AI Agents** - Understanding database structure for code generation
- 📊 **Analysis Tools** - Schema analysis and documentation generation
- 🔄 **Migration Tools** - Understanding current state for migrations
- 🏗️ **Architecture Review** - Complete database structure overview

## 📂 Directory Structure

```
tools/db-export/
├─ export_pg_metadata.sh      # Main one-shot exporter script
├─ queries/                   # SQL queries for metadata extraction
│  ├─ tables.sql             # Table definitions and metadata
│  ├─ columns.sql            # Column definitions, types, constraints
│  ├─ indexes.sql            # Index definitions and performance metadata
│  ├─ constraints.sql        # Primary keys, foreign keys, check constraints
│  ├─ policies.sql           # Row-level security policies
│  ├─ views.sql              # View definitions and materialized views
│  ├─ functions.sql          # Functions, procedures, triggers
│  └─ grants.sql             # Permissions and privilege grants
└─ README.md                 # This documentation
```

## 🚀 Quick Start

### Prerequisites
- PostgreSQL client (`psql`) installed
- Access to the Data Designer database
- Bash shell environment

### Basic Usage

```bash
# Navigate to the tool directory
cd tools/db-export

# Run with default settings (data_designer database)
./export_pg_metadata.sh

# Use custom database settings
DATABASE_NAME=mydb DATABASE_USER=myuser ./export_pg_metadata.sh
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_NAME` | Target database name | `data_designer` |
| `DATABASE_USER` | Database username | Current system user |
| `DATABASE_HOST` | Database hostname | `localhost` |
| `DATABASE_PORT` | Database port | `5432` |

## 📊 Output Files

The exporter creates a timestamped directory `./db-metadata-export/` containing:

### Core Metadata Files (CSV format)

| File | Description | Use Case |
|------|-------------|----------|
| **tables.csv** | Table definitions, owners, properties | Understanding table structure |
| **columns.csv** | Column metadata, types, nullability | Data type analysis and validation |
| **indexes.csv** | Index definitions and performance data | Query optimization analysis |
| **constraints.csv** | PK, FK, check constraints | Relationship mapping and validation |
| **policies.csv** | Row-level security policies | Security analysis |
| **views.csv** | View and materialized view definitions | Logic understanding |
| **functions.csv** | Functions, procedures, triggers | Business logic analysis |
| **grants.csv** | Permissions and security grants | Access control review |

### Summary Documentation

- **database_summary.md** - Human-readable overview with statistics and usage guidance

## 🤖 For External Agents

### Understanding the Data Designer Database

The Data Designer project uses a sophisticated PostgreSQL schema with:

- **Financial Entity Tables** - CBU, products, services, resources
- **DSL Templates** - Workflow definitions and capability mappings
- **Vector Embeddings** - Semantic search with pgvector
- **Investment Mandates** - Fund accounting and investment management
- **AI Integration** - Metadata for LLM understanding

### Key Tables to Analyze

Based on the Data Designer architecture, external agents should focus on:

1. **`client_business_units`** - Core business entity definitions
2. **`products`** - Financial product catalog
3. **`services`** - Service lifecycle definitions
4. **`resource_objects`** - Capability implementations
5. **`investment_mandates`** - Investment management workflows
6. **`attribute_objects`** - Enhanced metadata for AI systems

### CSV Analysis Examples

```bash
# Count total tables
tail -n +2 tables.csv | wc -l

# Find all financial entity tables
grep -E "(cbu|product|service|resource|mandate)" tables.csv

# Analyze column types distribution
cut -d',' -f8 columns.csv | sort | uniq -c | sort -nr

# Find foreign key relationships
grep "FOREIGN KEY" constraints.csv
```

## 🔧 Advanced Usage

### Custom Query Execution

You can modify the SQL queries in the `queries/` directory to extract additional metadata:

```sql
-- Example: Add custom metadata to tables.sql
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as table_size
FROM pg_tables
WHERE schemaname = 'public';
```

### Automated Exports

Set up regular exports for monitoring schema changes:

```bash
#!/bin/bash
# daily-export.sh
cd /path/to/data-designer/tools/db-export
./export_pg_metadata.sh
tar -czf "db-export-$(date +%Y%m%d).tar.gz" db-metadata-export/
```

## 🏗️ Integration with Data Designer

This tool is specifically designed for the **Data Designer Web-First Architecture**:

```
External Agents → CSV Metadata → Understanding → Code Generation
                ↗
Database → export_pg_metadata.sh → CSV Files → Analysis Tools
```

### Common Use Cases

1. **Schema Documentation** - Generate up-to-date database documentation
2. **Code Generation** - AI agents creating CRUD operations
3. **Migration Planning** - Understanding current state before changes
4. **Security Audit** - Reviewing permissions and policies
5. **Performance Analysis** - Index and constraint optimization

## 🛡️ Security Considerations

- **Read-Only Operations** - Tool only reads metadata, never modifies data
- **No Sensitive Data** - Exports schema structure only, not actual data
- **Permission Required** - Requires database read access for metadata queries
- **Local Output** - All exports stay on local filesystem

## 🔍 Troubleshooting

### Common Issues

**Connection Failed**
```bash
# Check database connectivity
psql -h localhost -U $(whoami) -d data_designer -c "SELECT version();"
```

**Permission Denied**
```bash
# Ensure user has necessary privileges
psql -d data_designer -c "GRANT USAGE ON SCHEMA information_schema TO $(whoami);"
```

**Missing Tables**
```bash
# Verify database exists and has tables
psql -d data_designer -c "\dt"
```

### Debug Mode

Add debug output to the export script:

```bash
# Enable verbose output
set -x
./export_pg_metadata.sh
```

## 📈 Output Example

```
==================================================
  PostgreSQL Database Metadata Exporter
==================================================
Database: data_designer
User: adamtc007
Host: localhost:5432
Output: ./db-metadata-export
Timestamp: 20241021_143022

[14:30:22] Testing database connection...
✅ Database connection successful
[14:30:22] Setting up output directory...
✅ Output directory created: ./db-metadata-export
[14:30:23] Exporting table definitions...
✅ Table definitions exported (23 rows)
[14:30:23] Exporting column metadata...
✅ Column metadata exported (185 rows)
...
✅ Database metadata export completed successfully!

Output Location: ./db-metadata-export
Summary Report: ./db-metadata-export/database_summary.md
```

## 🤝 Contributing

To enhance the metadata exporter:

1. **Add new queries** - Create SQL files in `queries/` directory
2. **Update export script** - Add new `run_query` calls in `export_pg_metadata.sh`
3. **Test thoroughly** - Verify exports work with different PostgreSQL versions
4. **Document changes** - Update this README with new functionality

---

**Data Designer Database Metadata Exporter** - Enabling external analysis of PostgreSQL schemas without direct database access.