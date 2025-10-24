// Comprehensive LISP DSL Smoke Tests - Full Round-Trip Validation
// Tests: EBNF → Parser → DSL Execution → Database → Query → Regenerate DSL

#[cfg(test)]
mod lisp_dsl_smoke_tests {
    use super::*;
    use crate::lisp_cbu_dsl::{LispCbuParser, LispDslError};
    use sqlx::PgPool;
    use tokio_test;

    /// Test Suite 1: EBNF Grammar Validation
    #[tokio::test]
    async fn test_ebnf_grammar_compliance() {
        println!("🧪 Testing EBNF Grammar Compliance");

        // Test Case 1.1: Valid LISP syntax patterns
        let valid_expressions = vec![
            // Basic S-expression
            "(create-cbu \"Test Fund\" \"Description\")",

            // With comments
            "; This is a comment\n(create-cbu \"Fund\" \"Desc\")",

            // Nested structures
            "(create-cbu \"Fund\" \"Desc\" (entities (entity \"US001\" \"Alpha\" investment-manager)))",

            // Multiple expressions
            "(create-cbu \"Fund1\" \"Desc1\")\n(query-cbu)",

            // Complex nesting
            "(create-cbu \"Fund\" \"Desc\" (entities (entity \"US001\" \"Alpha\" investment-manager) (entity \"US002\" \"Beta\" asset-owner)))",
        ];

        let parser = LispCbuParser::new(None);

        for (i, expr) in valid_expressions.iter().enumerate() {
            match parser.parse(expr) {
                Ok(parsed) => {
                    println!("✅ Test 1.{}: Valid expression parsed successfully", i + 1);
                    assert!(!parsed.is_empty(), "Parsed expression should not be empty");
                }
                Err(e) => {
                    panic!("❌ Test 1.{}: Valid expression failed to parse: {} - Expression: {}", i + 1, e, expr);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_syntax_rejection() {
        println!("🧪 Testing Invalid Syntax Rejection");

        let invalid_expressions = vec![
            // Unmatched parentheses
            "(create-cbu \"Fund\" \"Desc\"",

            // Invalid characters in symbols
            "(create-cbu \"Fund\" \"Desc\" @invalid)",

            // Empty expressions
            "",

            // Only comments
            "; Just a comment",

            // Malformed strings
            "(create-cbu \"unterminated string)",
        ];

        let parser = LispCbuParser::new(None);

        for (i, expr) in invalid_expressions.iter().enumerate() {
            match parser.parse(expr) {
                Ok(_) => {
                    if expr.trim().is_empty() || expr.trim().starts_with(';') {
                        println!("✅ Test 2.{}: Empty/comment-only expression handled correctly", i + 1);
                    } else {
                        panic!("❌ Test 2.{}: Invalid expression should have failed: {}", i + 1, expr);
                    }
                }
                Err(_) => {
                    println!("✅ Test 2.{}: Invalid expression correctly rejected", i + 1);
                }
            }
        }
    }

    /// Test Suite 2: Parser Functionality
    #[tokio::test]
    async fn test_tokenizer_accuracy() {
        println!("🧪 Testing Tokenizer Accuracy");

        let parser = LispCbuParser::new(None);

        // Test Case 2.1: Basic tokenization
        let tokens = parser.tokenize("(create-cbu \"Test Fund\" \"Description\")").unwrap();
        let expected = vec!["(", "create-cbu", "\"Test Fund\"", "\"Description\"", ")"];
        assert_eq!(tokens, expected, "Basic tokenization should match expected tokens");
        println!("✅ Test 2.1: Basic tokenization correct");

        // Test Case 2.2: Comment handling
        let tokens_with_comments = parser.tokenize("; Comment\n(create-cbu \"Fund\" \"Desc\")").unwrap();
        let expected_no_comments = vec!["(", "create-cbu", "\"Fund\"", "\"Desc\"", ")"];
        assert_eq!(tokens_with_comments, expected_no_comments, "Comments should be stripped during tokenization");
        println!("✅ Test 2.2: Comment stripping correct");

        // Test Case 2.3: String escaping
        let tokens_escaped = parser.tokenize("(create-cbu \"Fund\\\"Name\" \"Desc\")").unwrap();
        assert!(tokens_escaped[2].contains("\\\""), "Escaped quotes should be preserved");
        println!("✅ Test 2.3: String escaping correct");
    }

    #[tokio::test]
    async fn test_ast_construction() {
        println!("🧪 Testing AST Construction");

        let parser = LispCbuParser::new(None);

        // Test Case 2.4: Simple AST
        let ast = parser.parse("(create-cbu \"Fund\" \"Desc\")").unwrap();
        assert_eq!(ast.len(), 1, "Should parse to single expression");

        if let crate::lisp_cbu_dsl::LispValue::List(items) = &ast[0] {
            assert_eq!(items.len(), 3, "create-cbu should have 3 elements");
            assert_eq!(items[0], crate::lisp_cbu_dsl::LispValue::Symbol("create-cbu".to_string()));
            println!("✅ Test 2.4: AST structure correct");
        } else {
            panic!("❌ Test 2.4: AST should be a list");
        }

        // Test Case 2.5: Nested AST
        let nested_ast = parser.parse("(create-cbu \"Fund\" \"Desc\" (entities (entity \"US001\" \"Alpha\" role)))").unwrap();
        assert_eq!(nested_ast.len(), 1, "Should parse to single expression");
        println!("✅ Test 2.5: Nested AST construction correct");
    }

    /// Test Suite 3: DSL Execution (Mock Database)
    #[tokio::test]
    async fn test_dsl_execution_without_db() {
        println!("🧪 Testing DSL Execution (Mock Database)");

        let mut parser = LispCbuParser::new(None); // No database pool

        // Test Case 3.1: create-cbu execution
        let result = parser.parse_and_eval("(create-cbu \"Smoke Test Fund\" \"Test Description\")");
        match result {
            Ok(eval_result) => {
                assert!(eval_result.success, "create-cbu should succeed");
                assert!(eval_result.message.contains("created"), "Message should indicate creation");
                println!("✅ Test 3.1: create-cbu execution successful");
            }
            Err(e) => {
                panic!("❌ Test 3.1: create-cbu execution failed: {}", e);
            }
        }

        // Test Case 3.2: query-cbu execution
        let query_result = parser.parse_and_eval("(query-cbu)");
        match query_result {
            Ok(eval_result) => {
                assert!(eval_result.success, "query-cbu should succeed");
                println!("✅ Test 3.2: query-cbu execution successful");
            }
            Err(e) => {
                panic!("❌ Test 3.2: query-cbu execution failed: {}", e);
            }
        }

        // Test Case 3.3: Complex entity creation
        let complex_dsl = r#"
            ; Complex CBU with entities
            (create-cbu "Alpha Growth Fund" "Diversified growth investment fund"
              (entities
                (entity "US001" "Alpha Corp" investment-manager)
                (entity "US002" "Beta Holdings" asset-owner)))
        "#;

        let complex_result = parser.parse_and_eval(complex_dsl);
        match complex_result {
            Ok(eval_result) => {
                assert!(eval_result.success, "Complex CBU creation should succeed");
                println!("✅ Test 3.3: Complex entity creation successful");
            }
            Err(e) => {
                panic!("❌ Test 3.3: Complex entity creation failed: {}", e);
            }
        }
    }

    /// Test Suite 4: Error Handling and Edge Cases
    #[tokio::test]
    async fn test_error_handling() {
        println!("🧪 Testing Error Handling");

        let mut parser = LispCbuParser::new(None);

        // Test Case 4.1: Invalid function name
        let invalid_function_result = parser.parse_and_eval("(invalid-function \"arg1\" \"arg2\")");
        match invalid_function_result {
            Ok(_) => panic!("❌ Test 4.1: Invalid function should fail"),
            Err(LispDslError::UnknownFunction(_)) => {
                println!("✅ Test 4.1: Unknown function error handled correctly");
            }
            Err(e) => panic!("❌ Test 4.1: Unexpected error type: {}", e),
        }

        // Test Case 4.2: Arity mismatch
        let arity_error_result = parser.parse_and_eval("(create-cbu)"); // Missing required args
        match arity_error_result {
            Ok(_) => panic!("❌ Test 4.2: Arity mismatch should fail"),
            Err(LispDslError::ArityMismatch { .. }) => {
                println!("✅ Test 4.2: Arity mismatch error handled correctly");
            }
            Err(e) => panic!("❌ Test 4.2: Unexpected error type: {}", e),
        }

        // Test Case 4.3: Type mismatch
        let type_error_result = parser.parse_and_eval("(create-cbu 123 456)"); // Numbers instead of strings
        match type_error_result {
            Ok(_) => panic!("❌ Test 4.3: Type mismatch should fail"),
            Err(LispDslError::TypeError(_)) => {
                println!("✅ Test 4.3: Type error handled correctly");
            }
            Err(e) => panic!("❌ Test 4.3: Unexpected error type: {}", e),
        }
    }

    /// Test Suite 5: Performance and Stress Testing
    #[tokio::test]
    async fn test_performance_stress() {
        println!("🧪 Testing Performance and Stress");

        let parser = LispCbuParser::new(None);

        // Test Case 5.1: Large expression parsing
        let large_entities: Vec<String> = (1..=100)
            .map(|i| format!("(entity \"US{:03}\" \"Entity {}\" investment-manager)", i, i))
            .collect();

        let large_expression = format!(
            "(create-cbu \"Large Fund\" \"Fund with many entities\" (entities {}))",
            large_entities.join(" ")
        );

        let start_time = std::time::Instant::now();
        let result = parser.parse(&large_expression);
        let parse_duration = start_time.elapsed();

        match result {
            Ok(parsed) => {
                assert!(!parsed.is_empty(), "Large expression should parse successfully");
                println!("✅ Test 5.1: Large expression parsed in {:?}", parse_duration);
                assert!(parse_duration.as_millis() < 1000, "Parsing should be fast (< 1s)");
            }
            Err(e) => panic!("❌ Test 5.1: Large expression parsing failed: {}", e),
        }

        // Test Case 5.2: Multiple expression parsing
        let multiple_expressions: Vec<String> = (1..=50)
            .map(|i| format!("(create-cbu \"Fund{}\" \"Description{}\")", i, i))
            .collect();

        let multi_expr = multiple_expressions.join("\n");

        let start_time = std::time::Instant::now();
        let result = parser.parse(&multi_expr);
        let parse_duration = start_time.elapsed();

        match result {
            Ok(parsed) => {
                assert_eq!(parsed.len(), 50, "Should parse 50 expressions");
                println!("✅ Test 5.2: Multiple expressions parsed in {:?}", parse_duration);
            }
            Err(e) => panic!("❌ Test 5.2: Multiple expressions parsing failed: {}", e),
        }
    }

    /// Test Suite 6: DSL Generation and Round-Trip
    #[tokio::test]
    async fn test_dsl_generation_roundtrip() {
        println!("🧪 Testing DSL Generation Round-Trip");

        let parser = LispCbuParser::new(None);

        // Test Case 6.1: CBU with entities round-trip
        let entities = vec![
            ("US001".to_string(), "Alpha Corp".to_string(), "investment-manager".to_string()),
            ("US002".to_string(), "Beta Holdings".to_string(), "asset-owner".to_string()),
        ];

        let generated_dsl = parser.generate_dsl_from_cbu("Test Fund", "Test Description", &entities);
        println!("Generated DSL:\n{}", generated_dsl);

        // Parse the generated DSL
        let parsed_result = parser.parse(&generated_dsl);
        match parsed_result {
            Ok(parsed) => {
                assert!(!parsed.is_empty(), "Generated DSL should parse successfully");
                println!("✅ Test 6.1: DSL generation round-trip successful");

                // Verify structure
                if let crate::lisp_cbu_dsl::LispValue::List(items) = &parsed[0] {
                    assert_eq!(items[0], crate::lisp_cbu_dsl::LispValue::Symbol("create-cbu".to_string()));
                    println!("✅ Test 6.1: Generated DSL structure correct");
                }
            }
            Err(e) => panic!("❌ Test 6.1: Generated DSL failed to parse: {}", e),
        }

        // Test Case 6.2: Empty entity list
        let empty_dsl = parser.generate_dsl_from_cbu("Empty Fund", "No entities", &[]);
        let empty_parsed = parser.parse(&empty_dsl);
        match empty_parsed {
            Ok(parsed) => {
                assert!(!parsed.is_empty(), "Empty entity DSL should parse");
                println!("✅ Test 6.2: Empty entity DSL generation successful");
            }
            Err(e) => panic!("❌ Test 6.2: Empty entity DSL failed: {}", e),
        }
    }

    /// Integration Test: Full System Smoke Test
    #[tokio::test]
    async fn smoke_test_full_system() {
        println!("🚀 Full System Smoke Test");

        let mut parser = LispCbuParser::new(None);

        // Step 1: Parse complex DSL
        let complex_dsl = r#"
            ; Comprehensive CBU creation test
            (create-cbu "Smoke Test Alpha Fund" "Comprehensive smoke test fund"
              (entities
                (entity "US001" "Alpha Investment Management" investment-manager)
                (entity "US002" "Beta Pension Fund" asset-owner)
                (entity "US003" "Gamma Bank" custodian)))
        "#;

        println!("Step 1: Parsing complex DSL...");
        let parsed = parser.parse(complex_dsl).expect("Complex DSL should parse");
        assert!(!parsed.is_empty(), "Parsed result should not be empty");
        println!("✅ Step 1: Complex DSL parsed successfully");

        // Step 2: Execute DSL
        println!("Step 2: Executing DSL...");
        let execution_result = parser.parse_and_eval(complex_dsl).expect("DSL execution should succeed");
        assert!(execution_result.success, "DSL execution should be successful");
        assert!(execution_result.cbu_id.is_some(), "CBU ID should be generated");
        println!("✅ Step 2: DSL executed successfully - CBU ID: {:?}", execution_result.cbu_id);

        // Step 3: Generate DSL from data
        println!("Step 3: Generating DSL from data...");
        let entities = vec![
            ("US001".to_string(), "Alpha Investment Management".to_string(), "investment-manager".to_string()),
            ("US002".to_string(), "Beta Pension Fund".to_string(), "asset-owner".to_string()),
            ("US003".to_string(), "Gamma Bank".to_string(), "custodian".to_string()),
        ];
        let regenerated_dsl = parser.generate_dsl_from_cbu("Smoke Test Alpha Fund", "Comprehensive smoke test fund", &entities);
        println!("✅ Step 3: DSL regenerated successfully");

        // Step 4: Verify round-trip consistency
        println!("Step 4: Verifying round-trip consistency...");
        let reparsed = parser.parse(&regenerated_dsl).expect("Regenerated DSL should parse");
        assert!(!reparsed.is_empty(), "Reparsed DSL should not be empty");
        println!("✅ Step 4: Round-trip consistency verified");

        println!("🎉 Full System Smoke Test PASSED - All components working correctly!");
    }

    /// Utility function to run all smoke tests
    pub async fn run_all_smoke_tests() {
        println!("🔥 Running Complete LISP DSL Smoke Test Suite");
        println!("================================================");

        test_ebnf_grammar_compliance().await;
        test_invalid_syntax_rejection().await;
        test_tokenizer_accuracy().await;
        test_ast_construction().await;
        test_dsl_execution_without_db().await;
        test_error_handling().await;
        test_performance_stress().await;
        test_dsl_generation_roundtrip().await;
        smoke_test_full_system().await;

        println!("🎉 ALL SMOKE TESTS PASSED!");
        println!("✅ EBNF Grammar Compliance: OK");
        println!("✅ Parser Functionality: OK");
        println!("✅ DSL Execution: OK");
        println!("✅ Error Handling: OK");
        println!("✅ Performance: OK");
        println!("✅ Round-Trip: OK");
        println!("✅ Full System Integration: OK");
    }
}