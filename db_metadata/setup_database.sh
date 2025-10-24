#!/bin/bash

# Data Designer Database Setup Script
# Complete database recreation from DDL, migrations, and seed data

set -e

# Configuration
DB_NAME="${1:-data_designer}"
DB_USER="${2:-$(whoami)}"
DB_HOST="${3:-localhost}"
DB_PORT="${4:-5432}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}"
    echo "=================================================="
    echo "  Data Designer Database Setup"
    echo "=================================================="
    echo -e "${NC}"
    echo "Database: $DB_NAME"
    echo "User: $DB_USER"
    echo "Host: $DB_HOST:$DB_PORT"
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
    print_step "Testing PostgreSQL connection..."
    if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "SELECT version();" >/dev/null 2>&1; then
        print_success "PostgreSQL connection successful"
        return 0
    else
        print_error "Cannot connect to PostgreSQL"
        echo "Please ensure PostgreSQL is running and credentials are correct"
        exit 1
    fi
}

# Create database
create_database() {
    print_step "Creating database '$DB_NAME'..."

    # Drop database if exists (optional - comment out for safety)
    # psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "DROP DATABASE IF EXISTS $DB_NAME;" 2>/dev/null

    # Create database
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "CREATE DATABASE $DB_NAME;" 2>/dev/null || true

    # Test connection to new database
    if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT version();" >/dev/null 2>&1; then
        print_success "Database '$DB_NAME' ready"
    else
        print_error "Failed to create or connect to database '$DB_NAME'"
        exit 1
    fi
}

# Install extensions
install_extensions() {
    print_step "Installing required PostgreSQL extensions..."

    # Install pgvector for vector embeddings
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "CREATE EXTENSION IF NOT EXISTS vector;" 2>/dev/null || print_error "Failed to install pgvector extension"

    # Install other commonly used extensions
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";" 2>/dev/null || true
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "CREATE EXTENSION IF NOT EXISTS pg_trgm;" 2>/dev/null || true

    print_success "Extensions installed"
}

# Apply schema DDL
apply_schema() {
    print_step "Applying schema DDL..."

    if [ -f "schema_ddl.sql" ]; then
        psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "schema_ddl.sql" >/dev/null 2>&1
        if [ $? -eq 0 ]; then
            print_success "Schema DDL applied successfully"
        else
            print_error "Failed to apply schema DDL"
            exit 1
        fi
    else
        print_error "schema_ddl.sql not found"
        exit 1
    fi
}

# Apply migrations
apply_migrations() {
    print_step "Applying database migrations..."

    if [ -d "migrations" ]; then
        for migration in migrations/*.sql; do
            if [ -f "$migration" ]; then
                filename=$(basename "$migration")
                print_step "  Applying migration: $filename"
                psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$migration" >/dev/null 2>&1
                if [ $? -eq 0 ]; then
                    print_success "  Migration $filename applied"
                else
                    print_error "  Failed to apply migration $filename"
                    # Continue with other migrations
                fi
            fi
        done
        print_success "All migrations processed"
    else
        print_error "migrations/ directory not found"
    fi
}

# Load seed data
load_seed_data() {
    print_step "Loading seed data..."

    if [ -d "seed_data" ]; then
        for seed_file in seed_data/*.sql; do
            if [ -f "$seed_file" ]; then
                filename=$(basename "$seed_file")
                print_step "  Loading seed data: $filename"
                psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$seed_file" >/dev/null 2>&1
                if [ $? -eq 0 ]; then
                    print_success "  Seed data $filename loaded"
                else
                    print_error "  Failed to load seed data $filename"
                    # Continue with other seed files
                fi
            fi
        done
        print_success "All seed data processed"
    else
        print_error "seed_data/ directory not found"
    fi
}

# Verify database setup
verify_setup() {
    print_step "Verifying database setup..."

    # Count tables
    table_count=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT count(*) FROM information_schema.tables WHERE table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast');" 2>/dev/null | xargs)

    # Count rows in some key tables
    cbu_count=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT count(*) FROM client_business_units;" 2>/dev/null | xargs || echo "0")
    product_count=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT count(*) FROM products;" 2>/dev/null | xargs || echo "0")

    echo ""
    print_success "Database verification complete:"
    echo "  - Tables created: $table_count"
    echo "  - CBU records: $cbu_count"
    echo "  - Product records: $product_count"
    echo ""
}

# Main execution
main() {
    print_header

    # Check if we're in the right directory
    if [ ! -f "schema_ddl.sql" ]; then
        print_error "Please run this script from the db_metadata directory"
        exit 1
    fi

    test_connection
    create_database
    install_extensions
    apply_schema
    apply_migrations
    load_seed_data
    verify_setup

    echo ""
    print_success "Data Designer database setup completed successfully!"
    echo ""
    echo -e "${BLUE}Database Details:${NC}"
    echo "  Name: $DB_NAME"
    echo "  Host: $DB_HOST:$DB_PORT"
    echo "  User: $DB_USER"
    echo ""
    echo -e "${BLUE}Connection String:${NC}"
    echo "  postgresql://$DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"
    echo ""
    echo "The database is now ready for the Data Designer application!"
}

# Show usage if requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Data Designer Database Setup Script"
    echo ""
    echo "Usage: $0 [db_name] [db_user] [db_host] [db_port]"
    echo ""
    echo "Arguments:"
    echo "  db_name    Database name (default: data_designer)"
    echo "  db_user    Database user (default: current user)"
    echo "  db_host    Database host (default: localhost)"
    echo "  db_port    Database port (default: 5432)"
    echo ""
    echo "Example:"
    echo "  $0 my_data_designer myuser localhost 5432"
    echo ""
    echo "Prerequisites:"
    echo "  - PostgreSQL server running"
    echo "  - pgvector extension available"
    echo "  - User has CREATE DATABASE privileges"
    exit 0
fi

# Run main function
main "$@"