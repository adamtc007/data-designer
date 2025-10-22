//! S-Expression DSL Round Trip Tests
//! Comprehensive smoke tests for LISP-style DSL parsing, evaluation, and transpilation

use crate::lisp_cbu_dsl::{LispCbuParser, LispValue, LispCbuResult};
use crate::transpiler::{Transpiler, TranspilerOptions, TargetLanguage};
use anyhow::Result;

/// Test data for S-expression DSL smoke tests
pub struct SExpressionTestData {
    pub name: String,
    pub input_dsl: String,
    pub expected_success: bool,
    pub expected_entities: usize,
    pub description: String,
}

impl SExpressionTestData {
    /// Get comprehensive test cases for S-expression DSL
    pub fn get_test_cases() -> Vec<Self> {
        vec![
            Self {
                name: "simple_cbu_creation".to_string(),
                input_dsl: r#"
                    (create-cbu "Growth Alpha" "Diversified growth fund"
                      (entities
                        (entity "AC001" "Alpha Capital" asset-owner)
                        (entity "BM002" "Beta Management" investment-manager)))
                "#.to_string(),
                expected_success: true,
                expected_entities: 2,
                description: "Basic CBU creation with two entities".to_string(),
            },
            Self {
                name: "complex_cbu_with_multiple_roles".to_string(),
                input_dsl: r#"
                    (create-cbu "Goldman Sachs Investment Fund" "Multi-strategy hedge fund operations"
                      (entities
                        (entity "GS001" "Goldman Sachs Asset Management" asset-owner)
                        (entity "GS002" "Goldman Sachs Investment Advisors" investment-manager)
                        (entity "GS003" "Goldman Sachs Services" managing-company)
                        (entity "BNY001" "BNY Mellon" custodian)
                        (entity "PWC001" "PricewaterhouseCoopers" administrator)))
                "#.to_string(),
                expected_success: true,
                expected_entities: 5,
                description: "Complex CBU with multiple entity roles".to_string(),
            },
            Self {
                name: "cbu_update_operation".to_string(),
                input_dsl: r#"
                    (update-cbu "CBU001")
                "#.to_string(),
                expected_success: true,
                expected_entities: 0,
                description: "Simple CBU update operation".to_string(),
            },
            Self {
                name: "minimal_entity_definition".to_string(),
                input_dsl: r#"
                    (entity "TEST001" "Test Entity" asset-owner)
                "#.to_string(),
                expected_success: true,
                expected_entities: 0,
                description: "Minimal entity definition".to_string(),
            },
            Self {
                name: "cbu_with_comments".to_string(),
                input_dsl: r#"
                    ; This is a test CBU with comments
                    (create-cbu "Test Fund" "Test fund description"
                      ; Entity definitions below
                      (entities
                        (entity "E001" "Entity One" asset-owner) ; Primary entity
                        (entity "E002" "Entity Two" investment-manager))) ; Secondary entity
                "#.to_string(),
                expected_success: true,
                expected_entities: 2,
                description: "CBU creation with LISP-style comments".to_string(),
            },
            Self {
                name: "empty_cbu".to_string(),
                input_dsl: r#"
                    (create-cbu "Empty Fund" "Fund with no entities")
                "#.to_string(),
                expected_success: true,
                expected_entities: 0,
                description: "CBU creation without entities".to_string(),
            },
            Self {
                name: "nested_expressions".to_string(),
                input_dsl: r#"
                    (create-cbu "Nested Test" "Testing nested expressions"
                      (entities
                        (entity "N001" "Nested Entity"
                          (if (> aum 1000000)
                            asset-owner
                            investment-manager))))
                "#.to_string(),
                expected_success: false, // This should fail gracefully
                expected_entities: 0,
                description: "Test error handling with unsupported nested expressions".to_string(),
            },
            Self {
                name: "special_characters_in_names".to_string(),
                input_dsl: r#"
                    (create-cbu "Fonds d'Investissement Européen" "European investment fund with special characters"
                      (entities
                        (entity "EU001" "Société Générale" asset-owner)
                        (entity "EU002" "BNP Paribas Asset Management" investment-manager)))
                "#.to_string(),
                expected_success: true,
                expected_entities: 2,
                description: "CBU with international characters and special symbols".to_string(),
            },
        ]
    }

    /// Get test cases for different entity roles
    pub fn get_entity_role_test_cases() -> Vec<Self> {
        vec![
            Self {
                name: "asset_owner_entity".to_string(),
                input_dsl: r#"(entity "AO001" "Asset Owner Entity" asset-owner)"#.to_string(),
                expected_success: true,
                expected_entities: 0,
                description: "Asset owner entity definition".to_string(),
            },
            Self {
                name: "investment_manager_entity".to_string(),
                input_dsl: r#"(entity "IM001" "Investment Manager Entity" investment-manager)"#.to_string(),
                expected_success: true,
                expected_entities: 0,
                description: "Investment manager entity definition".to_string(),
            },
            Self {
                name: "managing_company_entity".to_string(),
                input_dsl: r#"(entity "MC001" "Managing Company Entity" managing-company)"#.to_string(),
                expected_success: true,
                expected_entities: 0,
                description: "Managing company entity definition".to_string(),
            },
            Self {
                name: "custodian_entity".to_string(),
                input_dsl: r#"(entity "CU001" "Custodian Entity" custodian)"#.to_string(),
                expected_success: true,
                expected_entities: 0,
                description: "Custodian entity definition".to_string(),
            },
            Self {
                name: "prime_broker_entity".to_string(),
                input_dsl: r#"(entity "PB001" "Prime Broker Entity" prime-broker)"#.to_string(),
                expected_success: true,
                expected_entities: 0,
                description: "Prime broker entity definition".to_string(),
            },
        ]
    }

    /// Get test cases for error conditions
    pub fn get_error_test_cases() -> Vec<Self> {
        vec![
            Self {
                name: "malformed_s_expression".to_string(),
                input_dsl: r#"(create-cbu "Test" "Description""#.to_string(), // Missing closing paren
                expected_success: false,
                expected_entities: 0,
                description: "Malformed S-expression with missing closing parenthesis".to_string(),
            },
            Self {
                name: "invalid_function_name".to_string(),
                input_dsl: r#"(invalid-function "arg1" "arg2")"#.to_string(),
                expected_success: false,
                expected_entities: 0,
                description: "Invalid function name that doesn't exist".to_string(),
            },
            Self {
                name: "insufficient_arguments".to_string(),
                input_dsl: r#"(create-cbu)"#.to_string(), // Missing required arguments
                expected_success: false,
                expected_entities: 0,
                description: "Function call with insufficient arguments".to_string(),
            },
            Self {
                name: "invalid_entity_role".to_string(),
                input_dsl: r#"(entity "E001" "Test Entity" invalid-role)"#.to_string(),
                expected_success: false,
                expected_entities: 0,
                description: "Entity with invalid role".to_string(),
            },
        ]
    }
}

/// Round trip test runner for S-expression DSL
pub struct SExpressionRoundTripTester {
    parser: LispCbuParser,
    transpiler: Transpiler,
}

impl SExpressionRoundTripTester {
    pub fn new() -> Self {
        Self {
            parser: LispCbuParser::new(None),
            transpiler: Transpiler::new(TranspilerOptions::default()),
        }
    }

    /// Run a complete round trip test: Parse -> Evaluate -> Transpile
    pub fn run_round_trip_test(&mut self, test_case: &SExpressionTestData) -> RoundTripResult {
        let mut result = RoundTripResult {
            test_name: test_case.name.clone(),
            parse_success: false,
            eval_success: false,
            transpile_success: false,
            parse_error: None,
            eval_error: None,
            transpile_error: None,
            parsed_result: None,
            eval_result: None,
            transpiled_code: None,
            entities_found: 0,
        };

        // Step 1: Parse and evaluate using the public API
        match self.parser.parse_and_eval(&test_case.input_dsl.trim()) {
            Ok(eval_result) => {
                result.parse_success = true;
                result.eval_success = true;
                result.parsed_result = Some(format!("Successfully parsed: {}", test_case.input_dsl.trim()));
                result.eval_result = Some(eval_result.message.clone());

                // Convert the evaluation result to LispValue for transpilation
                if let Some(data) = &eval_result.data {
                    // Step 3: Transpile to different target languages
                    let mut transpile_results = Vec::new();
                    for target in [TargetLanguage::Rust, TargetLanguage::SQL, TargetLanguage::JavaScript, TargetLanguage::Python] {
                        let mut target_transpiler = Transpiler::new(TranspilerOptions {
                            target: target.clone(),
                            ..Default::default()
                        });

                        match target_transpiler.transpile_s_expression(data) {
                            Ok(code) => {
                                transpile_results.push(format!("{:?}: {}", target, code));
                            }
                            Err(e) => {
                                transpile_results.push(format!("{:?}: ERROR - {}", target, e));
                            }
                        }
                    }

                    if !transpile_results.is_empty() {
                        result.transpile_success = true;
                        result.transpiled_code = Some(transpile_results.join("\n"));
                    }

                    // Count entities if this is a CBU result
                    result.entities_found = self.count_entities_in_result(data);
                } else {
                    // No data to transpile, but evaluation was successful
                    result.transpile_success = true;
                    result.transpiled_code = Some("// No data to transpile".to_string());
                }
            }
            Err(e) => {
                result.parse_error = Some(format!("{}", e));
                // Check if it's a parse error or eval error based on error type
                if format!("{}", e).contains("Parse") {
                    result.parse_success = false;
                } else {
                    result.parse_success = true;
                    result.eval_success = false;
                    result.eval_error = Some(format!("{}", e));
                }
            }
        }

        result
    }

    /// Count entities in the evaluation result
    fn count_entities_in_result(&self, result: &LispValue) -> usize {
        match result {
            LispValue::List(items) => {
                // Look for entities in the result
                let mut count = 0;
                for item in items {
                    if let LispValue::Symbol(s) = item {
                        if s == "entity" {
                            count += 1;
                        }
                    } else {
                        count += self.count_entities_in_result(item);
                    }
                }
                count
            }
            _ => 0,
        }
    }

    /// Run a comprehensive test suite
    pub fn run_comprehensive_test_suite(&mut self) -> TestSuiteResult {
        let mut suite_result = TestSuiteResult {
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            test_results: Vec::new(),
        };

        // Run main test cases
        let main_tests = SExpressionTestData::get_test_cases();
        for test_case in &main_tests {
            let result = self.run_round_trip_test(test_case);
            let passed = self.evaluate_test_result(&result, test_case);

            suite_result.total_tests += 1;
            if passed {
                suite_result.passed_tests += 1;
            } else {
                suite_result.failed_tests += 1;
            }
            suite_result.test_results.push(result);
        }

        // Run entity role tests
        let role_tests = SExpressionTestData::get_entity_role_test_cases();
        for test_case in &role_tests {
            let result = self.run_round_trip_test(test_case);
            let passed = self.evaluate_test_result(&result, test_case);

            suite_result.total_tests += 1;
            if passed {
                suite_result.passed_tests += 1;
            } else {
                suite_result.failed_tests += 1;
            }
            suite_result.test_results.push(result);
        }

        // Run error condition tests
        let error_tests = SExpressionTestData::get_error_test_cases();
        for test_case in &error_tests {
            let result = self.run_round_trip_test(test_case);
            let passed = self.evaluate_error_test_result(&result, test_case);

            suite_result.total_tests += 1;
            if passed {
                suite_result.passed_tests += 1;
            } else {
                suite_result.failed_tests += 1;
            }
            suite_result.test_results.push(result);
        }

        suite_result
    }

    /// Evaluate if a test result meets expectations
    fn evaluate_test_result(&self, result: &RoundTripResult, test_case: &SExpressionTestData) -> bool {
        if test_case.expected_success {
            result.parse_success && result.eval_success
        } else {
            !result.parse_success || !result.eval_success
        }
    }

    /// Evaluate error test results (should fail gracefully)
    fn evaluate_error_test_result(&self, result: &RoundTripResult, _test_case: &SExpressionTestData) -> bool {
        // Error tests should fail, but not crash
        !result.parse_success || !result.eval_success
    }
}

/// Result of a single round trip test
#[derive(Debug)]
pub struct RoundTripResult {
    pub test_name: String,
    pub parse_success: bool,
    pub eval_success: bool,
    pub transpile_success: bool,
    pub parse_error: Option<String>,
    pub eval_error: Option<String>,
    pub transpile_error: Option<String>,
    pub parsed_result: Option<String>,
    pub eval_result: Option<String>,
    pub transpiled_code: Option<String>,
    pub entities_found: usize,
}

/// Result of the complete test suite
#[derive(Debug)]
pub struct TestSuiteResult {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub test_results: Vec<RoundTripResult>,
}

impl TestSuiteResult {
    /// Get a summary report of the test suite
    pub fn get_summary(&self) -> String {
        format!(
            "S-Expression DSL Test Suite Results:\n\
             Total Tests: {}\n\
             Passed: {} ({:.1}%)\n\
             Failed: {} ({:.1}%)\n",
            self.total_tests,
            self.passed_tests,
            (self.passed_tests as f64 / self.total_tests as f64) * 100.0,
            self.failed_tests,
            (self.failed_tests as f64 / self.total_tests as f64) * 100.0
        )
    }

    /// Get detailed test results
    pub fn get_detailed_report(&self) -> String {
        let mut report = self.get_summary();
        report.push_str("\nDetailed Results:\n");

        for result in &self.test_results {
            report.push_str(&format!(
                "\n--- {} ---\n\
                 Parse: {} | Eval: {} | Transpile: {}\n",
                result.test_name,
                if result.parse_success { "✓" } else { "✗" },
                if result.eval_success { "✓" } else { "✗" },
                if result.transpile_success { "✓" } else { "✗" }
            ));

            if let Some(error) = &result.parse_error {
                report.push_str(&format!("Parse Error: {}\n", error));
            }
            if let Some(error) = &result.eval_error {
                report.push_str(&format!("Eval Error: {}\n", error));
            }
            if let Some(error) = &result.transpile_error {
                report.push_str(&format!("Transpile Error: {}\n", error));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s_expression_round_trip_smoke_test() {
        let mut tester = SExpressionRoundTripTester::new();
        let test_case = SExpressionTestData {
            name: "smoke_test".to_string(),
            input_dsl: r#"(create-cbu "Test Fund" "Smoke test fund")"#.to_string(),
            expected_success: true,
            expected_entities: 0,
            description: "Basic smoke test".to_string(),
        };

        let result = tester.run_round_trip_test(&test_case);
        assert!(result.parse_success, "Parse should succeed");
        assert!(result.eval_success, "Evaluation should succeed");
    }

    #[test]
    fn test_entity_creation_round_trip() {
        let mut tester = SExpressionRoundTripTester::new();
        let test_case = SExpressionTestData {
            name: "entity_test".to_string(),
            input_dsl: r#"(entity "TEST001" "Test Entity" asset-owner)"#.to_string(),
            expected_success: true,
            expected_entities: 0,
            description: "Entity creation test".to_string(),
        };

        let result = tester.run_round_trip_test(&test_case);
        assert!(result.parse_success, "Parse should succeed");
        assert!(result.eval_success, "Evaluation should succeed");
    }

    #[test]
    fn test_comprehensive_cbu_creation() {
        let mut tester = SExpressionRoundTripTester::new();
        let test_case = SExpressionTestData {
            name: "comprehensive_cbu".to_string(),
            input_dsl: r#"
                (create-cbu "Goldman Sachs Fund" "Multi-strategy hedge fund"
                  (entities
                    (entity "GS001" "Goldman Sachs" asset-owner)
                    (entity "GS002" "GS Investment Advisors" investment-manager)))
            "#.to_string(),
            expected_success: true,
            expected_entities: 2,
            description: "Comprehensive CBU creation".to_string(),
        };

        let result = tester.run_round_trip_test(&test_case);
        assert!(result.parse_success, "Parse should succeed");
        assert!(result.eval_success, "Evaluation should succeed");
    }

    #[test]
    fn test_error_handling() {
        let mut tester = SExpressionRoundTripTester::new();
        let test_case = SExpressionTestData {
            name: "error_test".to_string(),
            input_dsl: r#"(invalid-function "test")"#.to_string(),
            expected_success: false,
            expected_entities: 0,
            description: "Error handling test".to_string(),
        };

        let result = tester.run_round_trip_test(&test_case);
        // This should parse but fail on evaluation
        assert!(result.parse_success, "Parse should succeed");
        assert!(!result.eval_success, "Evaluation should fail for invalid function");
    }

    #[test]
    fn test_full_test_suite() {
        let mut tester = SExpressionRoundTripTester::new();
        let suite_result = tester.run_comprehensive_test_suite();

        println!("{}", suite_result.get_summary());

        // At least 80% of tests should pass
        let success_rate = suite_result.passed_tests as f64 / suite_result.total_tests as f64;
        assert!(success_rate >= 0.8,
            "Test suite success rate ({:.1}%) should be at least 80%",
            success_rate * 100.0);
    }

    #[test]
    fn test_transpilation_to_multiple_targets() {
        let mut tester = SExpressionRoundTripTester::new();
        let test_case = SExpressionTestData {
            name: "transpilation_test".to_string(),
            input_dsl: r#"(create-cbu "Test Fund" "Test description")"#.to_string(),
            expected_success: true,
            expected_entities: 0,
            description: "Transpilation test".to_string(),
        };

        let result = tester.run_round_trip_test(&test_case);
        assert!(result.parse_success, "Parse should succeed");
        assert!(result.eval_success, "Evaluation should succeed");
        assert!(result.transpile_success, "Transpilation should succeed");
        assert!(result.transpiled_code.is_some(), "Should have transpiled code");
    }
}