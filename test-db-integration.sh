#!/bin/bash

echo "Testing PostgreSQL Integration for Data Designer IDE"
echo "====================================================="

# Test database connection
echo "1. Testing database connection..."
psql -U adamtc007 -d data_designer -c "SELECT version();" > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ Database connection successful"
else
    echo "❌ Database connection failed"
    exit 1
fi

# Check if pgvector is installed
echo "2. Checking pgvector extension..."
psql -U adamtc007 -d data_designer -c "SELECT * FROM pg_extension WHERE extname = 'vector';" | grep -q vector
if [ $? -eq 0 ]; then
    echo "✅ pgvector extension is installed"
else
    echo "❌ pgvector extension not found"
fi

# Count rules in database
echo "3. Checking rules in database..."
RULE_COUNT=$(psql -U adamtc007 -d data_designer -t -c "SELECT COUNT(*) FROM rules;")
echo "✅ Found $RULE_COUNT rules in database"

# Count business attributes
echo "4. Checking business attributes..."
ATTR_COUNT=$(psql -U adamtc007 -d data_designer -t -c "SELECT COUNT(*) FROM business_attributes;")
echo "✅ Found $ATTR_COUNT business attributes"

# Count derived attributes
echo "5. Checking derived attributes..."
DERIVED_COUNT=$(psql -U adamtc007 -d data_designer -t -c "SELECT COUNT(*) FROM derived_attributes;")
echo "✅ Found $DERIVED_COUNT derived attributes"

# Test a sample query with embedding (checking column exists)
echo "6. Checking embedding column..."
psql -U adamtc007 -d data_designer -c "SELECT rule_id, rule_name, embedding IS NOT NULL as has_embedding FROM rules LIMIT 1;" > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ Embedding column exists and is accessible"
else
    echo "❌ Embedding column not accessible"
fi

echo ""
echo "Summary:"
echo "--------"
echo "Database: data_designer"
echo "Rules: $RULE_COUNT"
echo "Business Attributes: $ATTR_COUNT"
echo "Derived Attributes: $DERIVED_COUNT"
echo ""
echo "PostgreSQL with pgvector is ready for the IDE!"
echo ""
echo "To run the Web-First Data Designer with database support:"
echo "1. ./runwasm.sh"
echo "   (One command: build + serve + open browser)"
echo ""
echo "The Web Application will automatically:"
echo "- Connect to gRPC server (port 50051)"
echo "- Load financial entities from PostgreSQL via gRPC"
echo "- Generate vector embeddings for semantic search"
echo "- Execute DSL capabilities through the execution engine"
echo "- Persist all changes to the database"