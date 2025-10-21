#!/bin/bash

# PostgreSQL Database Metadata Exporter
# Exports complete database schema and metadata for external analysis
# Other agents can use this output to understand DB structure without direct access

set -e

# Configuration
DB_NAME="${DATABASE_NAME:-data_designer}"
DB_USER="${DATABASE_USER:-$(whoami)}"
DB_HOST="${DATABASE_HOST:-localhost}"
DB_PORT="${DATABASE_PORT:-5432}"
OUTPUT_DIR="./db-metadata-export"
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
    echo "Database: $DB_NAME"
    echo "User: $DB_USER"
    echo "Host: $DB_HOST:$DB_PORT"
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
    if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT version();" >/dev/null 2>&1; then
        print_success "Database connection successful"
        return 0
    else
        print_error "Cannot connect to database"
        echo "Please check:"
        echo "  - Database is running"
        echo "  - Credentials are correct"
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

    # Export as CSV with headers
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
         -c "\copy ($(cat "queries/$query_file")) TO '$OUTPUT_DIR/$output_file' WITH CSV HEADER;" 2>/dev/null

    if [ $? -eq 0 ]; then
        local row_count=$(tail -n +2 "$OUTPUT_DIR/$output_file" | wc -l | xargs)
        print_success "$description exported ($row_count rows)"
    else
        print_error "Failed to export $description"
        return 1
    fi
}

# Generate summary report
generate_summary() {
    print_step "Generating summary report..."

    local summary_file="$OUTPUT_DIR/database_summary.md"

    cat > "$summary_file" << EOF
# Database Metadata Export Summary

**Database:** $DB_NAME
**Export Date:** $(date)
**Export Tool:** PostgreSQL Database Metadata Exporter

## Purpose
This export provides complete database schema and metadata for external analysis.
Other agents/tools can use this information to understand the database structure
without requiring direct PostgreSQL access.

## Files Exported

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

## Database Overview

EOF

    # Get basic database stats
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "
        SELECT
            'Tables: ' || count(*)
        FROM information_schema.tables
        WHERE table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast');
    " 2>/dev/null >> "$summary_file"

    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "
        SELECT
            'Views: ' || count(*)
        FROM information_schema.views
        WHERE table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast');
    " 2>/dev/null >> "$summary_file"

    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "
        SELECT
            'Functions: ' || count(*)
        FROM information_schema.routines
        WHERE routine_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast');
    " 2>/dev/null >> "$summary_file"

    cat >> "$summary_file" << EOF

## Usage for External Agents

This metadata export can be used by other agents/tools to:

1. **Understand Schema Structure** - Table definitions, columns, data types
2. **Analyze Relationships** - Foreign keys, constraints, indexes
3. **Review Security** - Policies, grants, privileges
4. **Examine Logic** - Views, functions, procedures, triggers
5. **Plan Migrations** - Complete DDL understanding

## Key Files for Analysis

- **tables.csv** - All table structures and metadata
- **columns.csv** - Complete column definitions with types and constraints
- **constraints.csv** - Primary keys, foreign keys, check constraints
- **indexes.csv** - Index definitions and performance metadata
- **views.csv** - View definitions and dependencies
- **functions.csv** - Custom functions, procedures, and triggers
- **grants.csv** - Security permissions and privileges
- **policies.csv** - Row-level security policies

Each CSV file contains detailed metadata that external tools can parse and analyze
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

    generate_summary

    echo ""
    print_success "Database metadata export completed successfully!"
    echo ""
    echo -e "${BLUE}Output Location:${NC} $OUTPUT_DIR"
    echo -e "${BLUE}Summary Report:${NC} $OUTPUT_DIR/database_summary.md"
    echo ""
    echo "Other agents can now analyze the database structure using these exported files."
}

# Show usage if requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "PostgreSQL Database Metadata Exporter"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Environment Variables:"
    echo "  DATABASE_NAME    Database name (default: data_designer)"
    echo "  DATABASE_USER    Database user (default: current user)"
    echo "  DATABASE_HOST    Database host (default: localhost)"
    echo "  DATABASE_PORT    Database port (default: 5432)"
    echo ""
    echo "Example:"
    echo "  DATABASE_NAME=mydb DATABASE_USER=myuser $0"
    exit 0
fi

# Run main function
main "$@"