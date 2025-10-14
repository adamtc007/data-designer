#!/bin/bash

# Create clean version by removing all product/resource/service related functions
grep -n "function.*\(Product\|Resource\|Service\)" index.html | while read line; do
    func_line=$(echo $line | cut -d: -f1)
    echo "Found function at line $func_line"
done

# For now, let's just remove them manually with sed
cp index.html index-working.html

# Remove specific function blocks (this is a simplified approach)
sed -i '' '/async function refresh.*\(Product\|Resource\|Service\)/,/^[ ]*}$/d' index-working.html
sed -i '' '/function show.*\(Product\|Resource\|Service\)/,/^[ ]*}$/d' index-working.html
sed -i '' '/async function create.*\(Product\|Resource\|Service\)/,/^[ ]*}$/d' index-working.html
sed -i '' '/async function .*\(Product\|Resource\|Service\)/,/^[ ]*}$/d' index-working.html
sed -i '' '/function show.*Hierarchy/,/^[ ]*}$/d' index-working.html

echo "Cleaning complete"
