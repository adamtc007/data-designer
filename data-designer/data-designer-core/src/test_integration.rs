use crate::models::{DataDictionary, DerivedAttribute, Rule, Value};
use crate::engine::RulesEngine;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_ast_runtime_integration() {
        // Create a test data dictionary with a derived attribute
        let mut facts = HashMap::new();
        facts.insert("age".to_string(), Value::Integer(25));
        facts.insert("income".to_string(), Value::Integer(50000));
        facts.insert("name".to_string(), Value::String("John".to_string()));

        let rule = Rule {
            description: "Age category determination".to_string(),
            condition: Some("age >= 18".to_string()),
            value: Some("\"adult\"".to_string()),
            otherwise_value: Some("\"minor\"".to_string()),
        };

        let derived_attr = DerivedAttribute {
            name: "age_category".to_string(),
            attribute_type: "string".to_string(),
            visibility: "public".to_string(),
            description: "Determines if person is adult or minor".to_string(),
            embedding: None,
            dependencies: vec!["age".to_string()],
            rules: vec![rule],
            governance: Default::default(),
        };

        let dictionary = DataDictionary {
            canonical_models: vec![],
            derived_attributes: vec![derived_attr],
            solicitation_packs: vec![],
            axes: vec![],
        };

        // Create rules engine and evaluate
        let engine = RulesEngine::new(&dictionary).unwrap();
        let result = engine.evaluate_chain(&["age_category".to_string()], &facts).unwrap();

        // Check the result
        assert_eq!(result.get("age_category"), Some(&Value::String("adult".to_string())));
    }

    #[test]
    fn test_complex_arithmetic_expression() {
        let mut facts = HashMap::new();
        facts.insert("base_salary".to_string(), Value::Integer(50000));
        facts.insert("bonus_rate".to_string(), Value::Float(0.1));

        let rule = Rule {
            description: "Calculate total compensation".to_string(),
            condition: None,
            value: Some("base_salary + (base_salary * bonus_rate)".to_string()),
            otherwise_value: None,
        };

        let derived_attr = DerivedAttribute {
            name: "total_compensation".to_string(),
            attribute_type: "number".to_string(),
            visibility: "public".to_string(),
            description: "Total compensation including bonus".to_string(),
            embedding: None,
            dependencies: vec!["base_salary".to_string(), "bonus_rate".to_string()],
            rules: vec![rule],
            governance: Default::default(),
        };

        let dictionary = DataDictionary {
            canonical_models: vec![],
            derived_attributes: vec![derived_attr],
            solicitation_packs: vec![],
            axes: vec![],
        };

        let engine = RulesEngine::new(&dictionary).unwrap();
        let result = engine.evaluate_chain(&["total_compensation".to_string()], &facts).unwrap();

        // 50000 + (50000 * 0.1) = 55000
        assert_eq!(result.get("total_compensation"), Some(&Value::Float(55000.0)));
    }

    #[test]
    fn test_string_functions() {
        let mut facts = HashMap::new();
        facts.insert("first_name".to_string(), Value::String("John".to_string()));
        facts.insert("last_name".to_string(), Value::String("Doe".to_string()));

        let rule = Rule {
            description: "Create full name".to_string(),
            condition: None,
            value: Some("CONCAT(first_name, \" \", last_name)".to_string()),
            otherwise_value: None,
        };

        let derived_attr = DerivedAttribute {
            name: "full_name".to_string(),
            attribute_type: "string".to_string(),
            visibility: "public".to_string(),
            description: "Full name concatenation".to_string(),
            embedding: None,
            dependencies: vec!["first_name".to_string(), "last_name".to_string()],
            rules: vec![rule],
            governance: Default::default(),
        };

        let dictionary = DataDictionary {
            canonical_models: vec![],
            derived_attributes: vec![derived_attr],
            solicitation_packs: vec![],
            axes: vec![],
        };

        let engine = RulesEngine::new(&dictionary).unwrap();
        let result = engine.evaluate_chain(&["full_name".to_string()], &facts).unwrap();

        assert_eq!(result.get("full_name"), Some(&Value::String("John Doe".to_string())));
    }
}