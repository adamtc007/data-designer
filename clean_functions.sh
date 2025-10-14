#!/bin/bash

# Remove all product, resource, and service functions from index.html
sed -i '' '/async function refresh.*\(Product\|Resource\|Service\)/,/^[[:space:]]*}$/d' src/index.html
sed -i '' '/function show.*\(Product\|Resource\|Service\)Form/,/^[[:space:]]*}$/d' src/index.html
sed -i '' '/async function create.*\(Product\|Resource\|Service\)/,/^[[:space:]]*}$/d' src/index.html
sed -i '' '/async function update.*\(Product\|Resource\|Service\)/,/^[[:space:]]*}$/d' src/index.html
sed -i '' '/async function edit.*\(Product\|Resource\|Service\)/,/^[[:space:]]*}$/d' src/index.html
sed -i '' '/async function delete.*\(Product\|Resource\|Service\)/,/^[[:space:]]*}$/d' src/index.html
sed -i '' '/\/\/ .*\(Product\|Resource\|Service\) CRUD functions/d' src/index.html

echo "Function cleanup complete"
