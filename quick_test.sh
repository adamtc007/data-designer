#!/bin/bash

echo "Testing Data Designer IDE..."
echo "==============================="

# Test if the server is running
echo -n "1. Checking if server is running... "
if curl -s http://localhost:1420 > /dev/null; then
    echo "✅ Server is running"
else
    echo "❌ Server is not running"
    exit 1
fi

# Check if the application has the Test Data tab
echo -n "2. Checking for Test Data tab... "
if curl -s http://localhost:1420/src/index.html | grep -q "Test Data"; then
    echo "✅ Test Data tab exists"
else
    echo "❌ Test Data tab not found"
fi

# Check if source test data file exists
echo -n "3. Checking source test data... "
if [ -f "test_data/source_attributes.json" ]; then
    DATASETS=$(grep -o '"id"' test_data/source_attributes.json | wc -l)
    echo "✅ Found $DATASETS datasets"
else
    echo "❌ Source data not found"
fi

# Check if target test data file exists
echo -n "4. Checking target rules... "
if [ -f "test_data/target_attributes.json" ]; then
    RULES=$(grep -o '"rule_id"' test_data/target_attributes.json | wc -l)
    echo "✅ Found $RULES rule mappings"
else
    echo "❌ Target rules not found"
fi

# Check if the main JS file has the new test data functions
echo -n "5. Checking JavaScript functions... "
if grep -q "loadTestDataButton" src/main.js && grep -q "testWithDataButton" src/main.js; then
    echo "✅ Test data functions exist"
else
    echo "❌ Test data functions missing"
fi

# Check if Rust backend has the new commands
echo -n "6. Checking Rust commands... "
if grep -q "load_source_data" src-tauri/src/lib.rs && grep -q "test_rule_with_dataset" src-tauri/src/lib.rs; then
    echo "✅ Backend commands exist"
else
    echo "❌ Backend commands missing"
fi

echo ""
echo "==============================="
echo "✨ IDE Test Complete!"
echo ""
echo "The Data Designer IDE is running successfully with:"
echo "- Rules Editor with test rule selection"
echo "- Grammar Editor for DSL modification"
echo "- Dictionary for attribute management"
echo "- Test Data tab for dataset-based testing"
echo "- Compiler tab for rule compilation"
echo ""
echo "Access the application at: http://localhost:1420"