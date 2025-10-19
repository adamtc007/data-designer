use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use data_designer_core::{
    models::{DataDictionary, DerivedAttribute, Rule, Value},
    db::{rules::CreateRuleRequest, attributes::CreateDerivedAttributeRequest}
};

/// Test fixtures for consistent test data
pub struct TestFixtures {
    pub sample_rules: Vec<CreateRuleRequest>,
    pub sample_derived_attributes: Vec<CreateDerivedAttributeRequest>,
    pub sample_dictionaries: Vec<DataDictionary>,
    pub sample_input_data: HashMap<String, HashMap<String, Value>>,
    pub error_scenarios: Vec<ErrorScenario>,
}

impl TestFixtures {
    /// Load all test fixtures
    pub fn load() -> Result<Self> {
        Ok(Self {
            sample_rules: Self::create_sample_rules(),
            sample_derived_attributes: Self::create_sample_derived_attributes(),
            sample_dictionaries: Self::create_sample_dictionaries(),
            sample_input_data: Self::create_sample_input_data(),
            error_scenarios: Self::create_error_scenarios(),
        })
    }

    /// Create sample rules for testing
    fn create_sample_rules() -> Vec<CreateRuleRequest> {
        vec![
            CreateRuleRequest {
                rule_id: "age_category_rule".to_string(),
                rule_name: "Age Category Classification".to_string(),
                description: Some("Classify customers by age category".to_string()),
                category_key: "demographic".to_string(),
                target_attribute: "age_category".to_string(),
                source_attributes: vec!["age".to_string()],
                rule_definition: "if age >= 18 then \"adult\" else \"minor\"".to_string(),
                tags: Some(vec!["demographic".to_string(), "classification".to_string()]),
            },
            CreateRuleRequest {
                rule_id: "risk_calculation_rule".to_string(),
                rule_name: "Risk Level Calculation".to_string(),
                description: Some("Calculate customer risk level".to_string()),
                category_key: "risk".to_string(),
                target_attribute: "risk_level".to_string(),
                source_attributes: vec!["age".to_string(), "income".to_string(), "country".to_string()],
                rule_definition: "if country == \"US\" && income > 50000 && age > 25 then \"LOW\" else \"HIGH\"".to_string(),
                tags: Some(vec!["risk".to_string(), "calculation".to_string()]),
            },
            CreateRuleRequest {
                rule_id: "total_compensation_rule".to_string(),
                rule_name: "Total Compensation Calculation".to_string(),
                description: Some("Calculate total compensation including bonus".to_string()),
                category_key: "financial".to_string(),
                target_attribute: "total_compensation".to_string(),
                source_attributes: vec!["base_salary".to_string(), "bonus_rate".to_string()],
                rule_definition: "base_salary + (base_salary * bonus_rate)".to_string(),
                tags: Some(vec!["financial".to_string(), "calculation".to_string()]),
            },
            CreateRuleRequest {
                rule_id: "full_name_rule".to_string(),
                rule_name: "Full Name Concatenation".to_string(),
                description: Some("Create full name from first and last name".to_string()),
                category_key: "text".to_string(),
                target_attribute: "full_name".to_string(),
                source_attributes: vec!["first_name".to_string(), "last_name".to_string()],
                rule_definition: "CONCAT(first_name, \" \", last_name)".to_string(),
                tags: Some(vec!["text".to_string(), "concatenation".to_string()]),
            },
        ]
    }

    /// Create sample derived attributes
    fn create_sample_derived_attributes() -> Vec<CreateDerivedAttributeRequest> {
        vec![
            CreateDerivedAttributeRequest {
                name: "customer_segment".to_string(),
                data_type: "string".to_string(),
                description: Some("Customer segmentation based on age and income".to_string()),
                rule_logic: Some("if age < 30 && income < 40000 then \"Young Low Income\" else if age < 30 && income >= 40000 then \"Young High Income\" else if age >= 30 && income < 40000 then \"Mature Low Income\" else \"Mature High Income\"".to_string()),
                tags: Some(vec!["segmentation".to_string(), "marketing".to_string()]),
            },
            CreateDerivedAttributeRequest {
                name: "credit_score_category".to_string(),
                data_type: "string".to_string(),
                description: Some("Credit score categorization".to_string()),
                rule_logic: Some("if credit_score >= 750 then \"Excellent\" else if credit_score >= 700 then \"Good\" else if credit_score >= 650 then \"Fair\" else \"Poor\"".to_string()),
                tags: Some(vec!["credit".to_string(), "scoring".to_string()]),
            },
        ]
    }

    /// Create sample data dictionaries
    fn create_sample_dictionaries() -> Vec<DataDictionary> {
        vec![
            DataDictionary {
                datasets: vec![],
                lookup_tables: HashMap::new(),
                derived_attributes: vec![],
                canonical_models: vec![],
                solicitation_packs: vec![],
                axes: vec![],
            }
        ]
    }

    /// Create sample input data for testing
    fn create_sample_input_data() -> HashMap<String, HashMap<String, Value>> {
        let mut scenarios = HashMap::new();

        // Scenario 1: Young adult customer
        let mut young_adult = HashMap::new();
        young_adult.insert("age".to_string(), Value::Integer(25));
        young_adult.insert("income".to_string(), Value::Integer(45000));
        young_adult.insert("country".to_string(), Value::String("US".to_string()));
        young_adult.insert("first_name".to_string(), Value::String("John".to_string()));
        young_adult.insert("last_name".to_string(), Value::String("Doe".to_string()));
        young_adult.insert("credit_score".to_string(), Value::Integer(720));
        scenarios.insert("young_adult".to_string(), young_adult);

        // Scenario 2: Minor customer
        let mut minor = HashMap::new();
        minor.insert("age".to_string(), Value::Integer(16));
        minor.insert("income".to_string(), Value::Integer(0));
        minor.insert("country".to_string(), Value::String("US".to_string()));
        minor.insert("first_name".to_string(), Value::String("Jane".to_string()));
        minor.insert("last_name".to_string(), Value::String("Smith".to_string()));
        scenarios.insert("minor".to_string(), minor);

        // Scenario 3: High risk customer
        let mut high_risk = HashMap::new();
        high_risk.insert("age".to_string(), Value::Integer(22));
        high_risk.insert("income".to_string(), Value::Integer(25000));
        high_risk.insert("country".to_string(), Value::String("CA".to_string()));
        high_risk.insert("credit_score".to_string(), Value::Integer(580));
        scenarios.insert("high_risk".to_string(), high_risk);

        // Scenario 4: Employee compensation
        let mut employee = HashMap::new();
        employee.insert("base_salary".to_string(), Value::Integer(80000));
        employee.insert("bonus_rate".to_string(), Value::Float(0.15));
        scenarios.insert("employee".to_string(), employee);

        scenarios
    }

    /// Create error scenarios for negative testing
    fn create_error_scenarios() -> Vec<ErrorScenario> {
        vec![
            ErrorScenario {
                name: "invalid_dsl_syntax".to_string(),
                description: "DSL with syntax errors".to_string(),
                rule_definition: "if age >= then \"invalid\"".to_string(), // Missing value
                expected_error: "ParseError".to_string(),
            },
            ErrorScenario {
                name: "missing_dependency".to_string(),
                description: "Rule references non-existent attribute".to_string(),
                rule_definition: "if non_existent_field > 100 then \"error\"".to_string(),
                expected_error: "FieldNotFound".to_string(),
            },
            ErrorScenario {
                name: "type_mismatch".to_string(),
                description: "Comparing incompatible types".to_string(),
                rule_definition: "if age == \"twenty\" then \"error\"".to_string(),
                expected_error: "TypeMismatch".to_string(),
            },
            ErrorScenario {
                name: "division_by_zero".to_string(),
                description: "Mathematical error in calculation".to_string(),
                rule_definition: "salary / 0".to_string(),
                expected_error: "DivisionByZero".to_string(),
            },
        ]
    }

    /// Generate a large number of rules for performance testing
    pub fn generate_rules(&self, count: usize) -> Vec<CreateRuleRequest> {
        let mut rules = Vec::new();
        for i in 0..count {
            rules.push(CreateRuleRequest {
                rule_id: format!("generated_rule_{}", i),
                rule_name: format!("Generated Rule {}", i),
                description: Some(format!("Auto-generated rule for testing #{}", i)),
                category_key: "generated".to_string(),
                target_attribute: format!("output_{}", i),
                source_attributes: vec![format!("input_{}", i)],
                rule_definition: format!("if input_{} > {} then \"high\" else \"low\"", i, i * 10),
                tags: Some(vec!["generated".to_string(), "performance".to_string()]),
            });
        }
        rules
    }

    /// Generate random input data for testing
    pub fn generate_random_input(&self, scenario_name: &str, size: usize) -> Vec<HashMap<String, Value>> {
        let mut inputs = Vec::new();

        for i in 0..size {
            let mut input = HashMap::new();

            match scenario_name {
                "age_income" => {
                    input.insert("age".to_string(), Value::Integer(18 + (i % 50) as i64));
                    input.insert("income".to_string(), Value::Integer(25000 + (i % 75000) as i64));
                },
                "names" => {
                    input.insert("first_name".to_string(), Value::String(format!("Person{}", i)));
                    input.insert("last_name".to_string(), Value::String(format!("Surname{}", i)));
                },
                "financial" => {
                    input.insert("base_salary".to_string(), Value::Integer(30000 + (i % 120000) as i64));
                    input.insert("bonus_rate".to_string(), Value::Float(0.05 + (i % 20) as f64 / 100.0));
                },
                _ => {
                    input.insert("value".to_string(), Value::Integer(i as i64));
                }
            }

            inputs.push(input);
        }

        inputs
    }

    /// Clean up any test data files
    pub fn cleanup(&self) -> Result<()> {
        // In a real implementation, this might clean up temporary files
        // For now, just log the cleanup
        tracing::info!("Cleaned up test fixtures");
        Ok(())
    }

    /// Get expected results for known test scenarios
    pub fn get_expected_result(&self, scenario: &str, rule_name: &str) -> Option<Value> {
        match (scenario, rule_name) {
            ("young_adult", "age_category_rule") => Some(Value::String("adult".to_string())),
            ("minor", "age_category_rule") => Some(Value::String("minor".to_string())),
            ("young_adult", "risk_calculation_rule") => Some(Value::String("LOW".to_string())),
            ("high_risk", "risk_calculation_rule") => Some(Value::String("HIGH".to_string())),
            ("employee", "total_compensation_rule") => Some(Value::Float(92000.0)), // 80000 + (80000 * 0.15)
            ("young_adult", "full_name_rule") => Some(Value::String("John Doe".to_string())),
            ("minor", "full_name_rule") => Some(Value::String("Jane Smith".to_string())),
            _ => None,
        }
    }

    /// Verify that test data is valid
    pub fn validate_fixtures(&self) -> Result<()> {
        // Validate sample rules
        for rule in &self.sample_rules {
            if rule.rule_definition.is_empty() {
                return Err(anyhow::anyhow!("Rule definition cannot be empty for rule: {}", rule.rule_id));
            }
        }

        // Validate input data scenarios
        for (scenario_name, data) in &self.sample_input_data {
            if data.is_empty() {
                return Err(anyhow::anyhow!("Input data cannot be empty for scenario: {}", scenario_name));
            }
        }

        tracing::info!("All test fixtures validated successfully");
        Ok(())
    }
}

/// Error scenario for negative testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorScenario {
    pub name: String,
    pub description: String,
    pub rule_definition: String,
    pub expected_error: String,
}