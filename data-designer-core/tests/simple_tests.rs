use data_designer_core::parser::*;
use std::collections::HashMap;

#[cfg(test)]
mod simple_parser_tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        // Test that basic expressions can be parsed
        let expressions = vec![
            "42",
            "'hello world'",
            "1 + 2",
            "customer_name",
            "UPPER('test')",
        ];

        for expr in expressions {
            let result = parse_expression(expr);
            assert!(result.is_ok(), "Failed to parse: {}", expr);
        }
    }

    #[test]
    fn test_complex_expressions() {
        let complex_expressions = vec![
            "if customer_age > 18 then 'adult' else 'minor'",
            "CONCAT('Hello ', customer_name)",
            "customer_balance * 1.1",
            "(amount + fee) * tax_rate",
        ];

        for expr in complex_expressions {
            let result = parse_expression(expr);
            assert!(result.is_ok(), "Failed to parse complex expression: {}", expr);
        }
    }

    #[test]
    fn test_invalid_expressions() {
        let invalid_expressions = vec![
            "",
            "1 +",
            "(((",
            "'unclosed string",
        ];

        for expr in invalid_expressions {
            let result = parse_expression(expr);
            assert!(result.is_err(), "Should fail to parse invalid expression: {}", expr);
        }
    }
}

#[cfg(test)]
mod database_model_tests {
    use super::*;
    use data_designer_core::db::*;

    #[test]
    fn test_attribute_definition_creation() {
        let attr = AttributeDefinition {
            attribute_type: "business".to_string(),
            entity_name: "customers".to_string(),
            attribute_name: "customer_id".to_string(),
            full_path: "customers.customer_id".to_string(),
            data_type: "integer".to_string(),
            sql_type: Some("INTEGER".to_string()),
            rust_type: Some("i32".to_string()),
            description: Some("Unique customer identifier".to_string()),
        };

        assert_eq!(attr.attribute_type, "business");
        assert_eq!(attr.entity_name, "customers");
        assert!(attr.description.is_some());
    }

    #[test]
    fn test_create_derived_attribute_request() {
        let request = CreateDerivedAttributeRequest {
            name: "test_attribute".to_string(),
            data_type: "string".to_string(),
            description: Some("Test attribute".to_string()),
            rule_logic: Some("UPPER(customer_name)".to_string()),
            tags: Some(vec!["test".to_string()]),
        };

        assert_eq!(request.name, "test_attribute");
        assert_eq!(request.data_type, "string");
        assert!(request.description.is_some());
    }

    #[test]
    fn test_data_dictionary_response() {
        let response = DataDictionaryResponse {
            attributes: vec![
                serde_json::json!({
                    "attribute_name": "test_attr",
                    "data_type": "string"
                })
            ],
            total_count: 1,
            business_count: 1,
            derived_count: 0,
            system_count: 0,
        };

        assert_eq!(response.total_count, 1);
        assert_eq!(response.business_count, 1);
        assert_eq!(response.attributes.len(), 1);
    }
}