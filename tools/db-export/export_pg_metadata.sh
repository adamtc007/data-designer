#!/bin/bash

# PostgreSQL Database Metadata Exporter
# Exports complete database schema and metadata for external analysis
# Other agents can use this output to understand DB structure without direct access

set -e

# Parse connection string if provided as first argument
CONNECTION_STRING=""
SCHEMAS_FILTER=""

if [ $# -ge 1 ]; then
    CONNECTION_STRING="$1"
fi

if [ $# -ge 2 ]; then
    SCHEMAS_FILTER="$2"
fi

# Configuration
if [ -n "$CONNECTION_STRING" ]; then
    # Use connection string
    DB_CONNECTION="$CONNECTION_STRING"
    echo "Using connection string: ${CONNECTION_STRING%%:*}://[USER]:[PASS]@[HOST]:[PORT]/[DB]"
else
    # Use individual parameters
    DB_NAME="${DATABASE_NAME:-data_designer}"
    DB_USER="${DATABASE_USER:-$(whoami)}"
    DB_HOST="${DATABASE_HOST:-localhost}"
    DB_PORT="${DATABASE_PORT:-5432}"
    DB_CONNECTION="postgresql://$DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"
fi

# Schema filtering
if [ -n "$SCHEMAS_FILTER" ]; then
    SCHEMA_WHERE_CLAUSE="AND table_schema IN ('$(echo "$SCHEMAS_FILTER" | sed "s/,/','/g")')"
    SCHEMA_LIST="Schemas: $SCHEMAS_FILTER"
else
    SCHEMA_WHERE_CLAUSE="AND table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')"
    SCHEMA_LIST="Schemas: All user schemas"
fi

OUTPUT_DIR="./db_metadata"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}"
    echo "=================================================="
    echo "  PostgreSQL Database Metadata Exporter"
    echo "=================================================="
    echo -e "${NC}"
    echo "Connection: $DB_CONNECTION"
    echo "$SCHEMA_LIST"
    echo "Output: $OUTPUT_DIR"
    echo "Timestamp: $TIMESTAMP"
    echo ""
}

print_step() {
    echo -e "${YELLOW}[$(date +'%H:%M:%S')] $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Test database connection
test_connection() {
    print_step "Testing database connection..."
    if psql "$DB_CONNECTION" -c "SELECT version();" >/dev/null 2>&1; then
        print_success "Database connection successful"
        return 0
    else
        print_error "Cannot connect to database"
        echo "Please check:"
        echo "  - Database is running"
        echo "  - Connection string is correct"
        echo "  - Network connectivity"
        exit 1
    fi
}

# Create output directory
setup_output_dir() {
    print_step "Setting up output directory..."
    rm -rf "$OUTPUT_DIR"
    mkdir -p "$OUTPUT_DIR"
    print_success "Output directory created: $OUTPUT_DIR"
}

# Execute SQL query and save results
run_query() {
    local query_file="$1"
    local output_file="$2"
    local description="$3"

    if [ ! -f "queries/$query_file" ]; then
        print_error "Query file not found: queries/$query_file"
        return 1
    fi

    print_step "Exporting $description..."

    # Modify query to include schema filter
    local modified_query
    modified_query=$(cat "queries/$query_file" | sed "s/WHERE table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')/WHERE 1=1 $SCHEMA_WHERE_CLAUSE/g")
    modified_query=$(echo "$modified_query" | sed "s/WHERE schemaname NOT IN ('information_schema', 'pg_catalog', 'pg_toast')/WHERE 1=1 $(echo "$SCHEMA_WHERE_CLAUSE" | sed 's/table_schema/schemaname/g')/g")

    # Export as CSV with headers
    echo "$modified_query" | psql "$DB_CONNECTION" -c "\copy ($(cat)) TO '$OUTPUT_DIR/$output_file' WITH CSV HEADER;" 2>/dev/null

    if [ $? -eq 0 ]; then
        local row_count=$(tail -n +2 "$OUTPUT_DIR/$output_file" | wc -l | xargs)
        print_success "$description exported ($row_count rows)"
    else
        print_error "Failed to export $description"
        return 1
    fi
}

# Generate DDL export
export_ddl() {
    print_step "Exporting DDL (Data Definition Language)..."

    local ddl_file="$OUTPUT_DIR/schema_ddl.sql"

    if [ -n "$SCHEMAS_FILTER" ]; then
        local schema_list="'$(echo "$SCHEMAS_FILTER" | sed "s/,/','/g")'"
        pg_dump "$DB_CONNECTION" --schema-only --no-owner --no-privileges \
            $(echo "$SCHEMAS_FILTER" | sed 's/,/ --schema=/g' | sed 's/^/--schema=/') \
            > "$ddl_file" 2>/dev/null
    else
        pg_dump "$DB_CONNECTION" --schema-only --no-owner --no-privileges \
            --exclude-schema=information_schema --exclude-schema=pg_catalog \
            --exclude-schema=pg_toast > "$ddl_file" 2>/dev/null
    fi

    if [ $? -eq 0 ]; then
        local line_count=$(wc -l < "$ddl_file" | xargs)
        print_success "DDL exported ($line_count lines)"
    else
        print_error "Failed to export DDL"
    fi
}

# Generate JSON metadata
export_json_metadata() {
    print_step "Generating JSON metadata..."

    local json_file="$OUTPUT_DIR/metadata.json"

    cat > "$json_file" << EOF
{
  "export_info": {
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "connection": "$(echo "$DB_CONNECTION" | sed 's/:\/\/[^@]*@/:\/\/***:***@/')",
    "schemas": $(if [ -n "$SCHEMAS_FILTER" ]; then echo "\"$SCHEMAS_FILTER\""; else echo "\"all_user_schemas\""; fi),
    "exporter_version": "2.0"
  },
  "database_stats": {
EOF

    # Get database statistics
    psql "$DB_CONNECTION" -t -c "
        SELECT
            '    \"tables\": ' || count(*) || ','
        FROM information_schema.tables
        WHERE 1=1 $SCHEMA_WHERE_CLAUSE;
    " 2>/dev/null >> "$json_file"

    psql "$DB_CONNECTION" -t -c "
        SELECT
            '    \"views\": ' || count(*) || ','
        FROM information_schema.views
        WHERE 1=1 $SCHEMA_WHERE_CLAUSE;
    " 2>/dev/null >> "$json_file"

    psql "$DB_CONNECTION" -t -c "
        SELECT
            '    \"functions\": ' || count(*) || ','
        FROM information_schema.routines
        WHERE 1=1 $(echo "$SCHEMA_WHERE_CLAUSE" | sed 's/table_schema/routine_schema/g');
    " 2>/dev/null >> "$json_file"

    psql "$DB_CONNECTION" -t -c "
        SELECT
            '    \"columns\": ' || count(*) || ''
        FROM information_schema.columns
        WHERE 1=1 $SCHEMA_WHERE_CLAUSE;
    " 2>/dev/null >> "$json_file"

    cat >> "$json_file" << EOF
  },
  "files_exported": [
    "tables.csv",
    "columns.csv",
    "indexes.csv",
    "constraints.csv",
    "policies.csv",
    "views.csv",
    "functions.csv",
    "grants.csv",
    "schema_ddl.sql",
    "database_summary.md"
  ]
}
EOF

    print_success "JSON metadata generated"
}

# Generate summary report
generate_summary() {
    print_step "Generating summary report..."

    local summary_file="$OUTPUT_DIR/database_summary.md"

    cat > "$summary_file" << EOF
# Database Metadata Export Summary

**Connection:** $(echo "$DB_CONNECTION" | sed 's/:\/\/[^@]*@/:\/\/***:***@/')
**Schemas:** $(if [ -n "$SCHEMAS_FILTER" ]; then echo "$SCHEMAS_FILTER"; else echo "All user schemas"; fi)
**Export Date:** $(date)
**Export Tool:** PostgreSQL Database Metadata Exporter v2.0

## Purpose
This export provides complete database schema and metadata for external analysis.
Other agents/tools can use this information to understand the database structure
without requiring direct PostgreSQL access.

## Files Exported

### CSV Metadata Files
EOF

    # Add file descriptions
    for file in "$OUTPUT_DIR"/*.csv; do
        if [ -f "$file" ]; then
            local filename=$(basename "$file")
            local row_count=$(tail -n +2 "$file" | wc -l | xargs)
            echo "- **$filename** - $row_count records" >> "$summary_file"
        fi
    done

    cat >> "$summary_file" << EOF

### DDL and Documentation
- **schema_ddl.sql** - Complete Data Definition Language export
- **metadata.json** - Machine-readable metadata and statistics
- **database_summary.md** - This human-readable summary

## Database Overview

EOF

    # Get basic database stats
    psql "$DB_CONNECTION" -t -c "
        SELECT
            'Tables: ' || count(*)
        FROM information_schema.tables
        WHERE 1=1 $SCHEMA_WHERE_CLAUSE;
    " 2>/dev/null >> "$summary_file"

    psql "$DB_CONNECTION" -t -c "
        SELECT
            'Views: ' || count(*)
        FROM information_schema.views
        WHERE 1=1 $SCHEMA_WHERE_CLAUSE;
    " 2>/dev/null >> "$summary_file"

    psql "$DB_CONNECTION" -t -c "
        SELECT
            'Functions: ' || count(*)
        FROM information_schema.routines
        WHERE 1=1 $(echo "$SCHEMA_WHERE_CLAUSE" | sed 's/table_schema/routine_schema/g');
    " 2>/dev/null >> "$summary_file"

    cat >> "$summary_file" << EOF

## Usage for External Agents

This metadata export can be used by other agents/tools to:

1. **Understand Schema Structure** - Table definitions, columns, data types
2. **Analyze Relationships** - Foreign keys, constraints, indexes
3. **Review Security** - Policies, grants, privileges
4. **Examine Logic** - Views, functions, procedures, triggers
5. **Plan Migrations** - Complete DDL understanding
6. **Generate Code** - Use schema metadata for CRUD operations

## Key Files for Analysis

- **schema_ddl.sql** - Complete DDL for schema recreation
- **metadata.json** - Machine-readable statistics and metadata
- **tables.csv** - All table structures and metadata
- **columns.csv** - Complete column definitions with types and constraints
- **constraints.csv** - Primary keys, foreign keys, check constraints
- **indexes.csv** - Index definitions and performance metadata
- **views.csv** - View definitions and dependencies
- **functions.csv** - Custom functions, procedures, and triggers
- **grants.csv** - Security permissions and privileges
- **policies.csv** - Row-level security policies

Each file contains detailed metadata that external tools can parse and analyze
without requiring direct database connectivity.

EOF

    print_success "Summary report generated: $summary_file"
}

# Main execution
main() {
    print_header

    # Check if we're in the right directory
    if [ ! -d "queries" ]; then
        print_error "Please run this script from the tools/db-export directory"
        exit 1
    fi

    test_connection
    setup_output_dir

    # Export all metadata components
    run_query "tables.sql" "tables.csv" "table definitions"
    run_query "columns.sql" "columns.csv" "column metadata"
    run_query "indexes.sql" "indexes.csv" "index definitions"
    run_query "constraints.sql" "constraints.csv" "constraint metadata"
    run_query "policies.sql" "policies.csv" "security policies"
    run_query "views.sql" "views.csv" "view definitions"
    run_query "functions.sql" "functions.csv" "functions and procedures"
    run_query "grants.sql" "grants.csv" "permissions and grants"

    # Export DDL and generate additional metadata
    export_ddl
    export_json_metadata
    generate_summary

    echo ""
    print_success "Database metadata export completed successfully!"
    echo ""
    echo -e "${BLUE}Output Location:${NC} $OUTPUT_DIR"
    echo -e "${BLUE}Summary Report:${NC} $OUTPUT_DIR/database_summary.md"
    echo -e "${BLUE}DDL Export:${NC} $OUTPUT_DIR/schema_ddl.sql"
    echo -e "${BLUE}JSON Metadata:${NC} $OUTPUT_DIR/metadata.json"
    echo ""
    echo "Ready for git commit:"
    echo "  git add $OUTPUT_DIR"
    echo "  git commit -m \"chore(db): snapshot schema metadata (DDL + JSON)\""
}

# Show usage if requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "PostgreSQL Database Metadata Exporter v2.0"
    echo ""
    echo "Usage: $0 [connection_string] [schemas]"
    echo ""
    echo "Arguments:"
    echo "  connection_string  PostgreSQL connection string (optional)"
    echo "                     Format: postgres://USER:PASS@HOST:5432/DB"
    echo "  schemas           Comma-separated list of schemas to export (optional)"
    echo "                     Example: public,client_core,deal_flow"
    echo ""
    echo "Environment Variables (used if no connection string provided):"
    echo "  DATABASE_NAME    Database name (default: data_designer)"
    echo "  DATABASE_USER    Database user (default: current user)"
    echo "  DATABASE_HOST    Database host (default: localhost)"
    echo "  DATABASE_PORT    Database port (default: 5432)"
    echo ""
    echo "Examples:"
    echo "  # Use environment variables"
    echo "  $0"
    echo ""
    echo "  # Use connection string"
    echo "  $0 \"postgres://user:pass@host:5432/db\""
    echo ""
    echo "  # Use connection string with specific schemas"
    echo "  $0 \"postgres://user:pass@host:5432/db\" \"public,client_core,deal_flow\""
    echo ""
    echo "Output:"
    echo "  - CSV files with metadata"
    echo "  - schema_ddl.sql with complete DDL"
    echo "  - metadata.json with statistics"
    echo "  - database_summary.md with documentation"
    exit 0
fi

# Run main function
main "$@"