#!/bin/bash

# Database setup script for Data Designer
echo "Setting up PostgreSQL database for Data Designer..."

# Try to connect as the current user first, fallback to postgres
if psql -lqt 2>/dev/null | cut -d \| -f 1 | grep -qw data_designer; then
    echo "Database 'data_designer' already exists"
    DB_EXISTS=1
else
    echo "Creating database 'data_designer'..."
    createdb data_designer 2>/dev/null || {
        echo "Trying with postgres user..."
        psql postgres -c "CREATE DATABASE data_designer;" 2>/dev/null || echo "Could not create database"
    }
fi

# Apply schema
echo "Applying database schema..."
psql data_designer -f database/schema.sql 2>&1 | grep -v "already exists" | grep -v "NOTICE"

echo "Database setup complete!"
echo ""
echo "To verify, run: psql data_designer -c '\dt'"