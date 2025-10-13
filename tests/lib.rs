use data_designer::{BusinessRule, RulesEngine, generate_test_context, get_sample_rules};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_simple_arithmetic() {
        let context = generate_test_context();
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Test".to_string(),
            "Test arithmetic".to_string(),
            "10 + 20 * 3".to_string(),
        );

        assert!(rule.parse().is_ok());
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result.as_f64().unwrap(), 70.0);
    }

    #[test]
    fn test_string_concatenation() {
        let context = generate_test_context();
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Test".to_string(),
            "Test string concat".to_string(),
            r#""Hello " & "World""#.to_string(),
        );

        assert!(rule.parse().is_ok());
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result.as_str().unwrap(), "Hello World");
    }

    #[test]
    fn test_variable_reference() {
        let context = generate_test_context();
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Test".to_string(),
            "Test variable reference".to_string(),
            "price * quantity".to_string(),
        );

        assert!(rule.parse().is_ok());
        let result = rule.evaluate(&context).unwrap();
        // 29.99 * 5 = 149.95
        assert_eq!(result.as_f64().unwrap(), 149.95);
    }

    #[test]
    fn test_function_call() {
        let context = generate_test_context();
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Test".to_string(),
            "Test function call".to_string(),
            r#"CONCAT("Hello ", name, "!")"#.to_string(),
        );

        assert!(rule.parse().is_ok());
        let result = rule.evaluate(&context).unwrap();
        // Context has name = "Alice"
        assert_eq!(result.as_str().unwrap(), "Hello Alice!");
    }

    #[test]
    fn test_rules_engine() {
        let context = generate_test_context();
        let mut engine = RulesEngine::new();

        for rule in get_sample_rules().into_iter().take(3) {
            assert!(engine.add_rule(rule).is_ok());
        }

        let results = engine.evaluate_all(&context);
        assert_eq!(results.len(), 3);

        for (_, result) in results {
            assert!(result.is_ok());
        }
    }
}