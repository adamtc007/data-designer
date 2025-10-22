#!/bin/bash

# Test script to verify LISP parser can handle new domain DSL expressions

echo "🧪 Testing LISP Parser with Domain DSL Expressions"
echo "================================================="

# Test 1: CBU DSL (already working)
echo ""
echo "1. Testing CBU DSL (existing)..."
TEST_CBU_DSL='
; CBU creation with entities
(create-cbu "Test Fund" "Test Description"
  (entities
    (entity "US001" "Alpha Corp" investment-manager)))
'

# Test 2: Deal Record DSL
echo ""
echo "2. Testing Deal Record DSL..."
TEST_DEAL_DSL='
; Deal record creation
(create-deal "DEAL_001" "Alpha Deal" "Alpha Corp"
  (components
    (cbus "CBU_001")
    (products "CUSTODY_001")))
'

# Test 3: Onboarding Request DSL
echo ""
echo "3. Testing Onboarding Request DSL..."
TEST_ONBOARDING_DSL='
; Onboarding request creation
(create-onboarding-request "DEAL_001" "Alpha Onboarding" "ONBOARD_001"
  (onboarding-spec
    (cbu "CBU_001")
    (products "CUSTODY_001")))
'

# Write test expressions to temporary files
echo "$TEST_CBU_DSL" > /tmp/test_cbu.lisp
echo "$TEST_DEAL_DSL" > /tmp/test_deal.lisp
echo "$TEST_ONBOARDING_DSL" > /tmp/test_onboarding.lisp

echo "✅ Test expressions created:"
echo "   - /tmp/test_cbu.lisp"
echo "   - /tmp/test_deal.lisp"
echo "   - /tmp/test_onboarding.lisp"

echo ""
echo "📝 To test manually, use these expressions in the web UI at http://localhost:8081"
echo "   Navigate to Entity Management → CBU Management and paste the expressions"

echo ""
echo "🎯 Expected behavior:"
echo "   - All expressions should parse without syntax errors"
echo "   - LISP parser should be selected (not EBNF)"
echo "   - Comments (;) should be handled correctly"
echo "   - Nested S-expressions should parse properly"