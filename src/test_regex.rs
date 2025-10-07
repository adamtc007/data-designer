#[cfg(test)]
mod regex_tests {
    use crate::BusinessRule;
    use std::collections::HashMap;
    use serde_json::json;

    #[test]
    fn test_regex_literal_parsing() {
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Regex Literal Test".to_string(),
            "Test regex literal parsing".to_string(),
            r#"pattern = /^[A-Z]{3}_\d{4}$/"#.to_string()
        );

        assert!(rule.parse().is_ok());
    }

    #[test]
    fn test_matches_operator() {
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Matches Test".to_string(),
            "Test MATCHES operator".to_string(),
            r#""INST_2024_00156" MATCHES /^INST_\d{4}_\d{5}$/"#.to_string()
        );

        rule.parse().unwrap();
        let context = HashMap::new();
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_email_function() {
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Email Validation".to_string(),
            "Test IS_EMAIL function".to_string(),
            r#"IS_EMAIL("john.doe@example.com")"#.to_string()
        );

        rule.parse().unwrap();
        let context = HashMap::new();
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_lei_function() {
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "LEI Validation".to_string(),
            "Test IS_LEI function".to_string(),
            r#"IS_LEI("549300VFXB3LH3JW7N94")"#.to_string()
        );

        rule.parse().unwrap();
        let context = HashMap::new();
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_swift_function() {
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "SWIFT Validation".to_string(),
            "Test IS_SWIFT function".to_string(),
            r#"IS_SWIFT("APXCUS33XXX")"#.to_string()
        );

        rule.parse().unwrap();
        let context = HashMap::new();
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_extract_function() {
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Extract Test".to_string(),
            "Test EXTRACT function".to_string(),
            r#"EXTRACT("INST_2024_00156", "\d{4}")"#.to_string()
        );

        rule.parse().unwrap();
        let context = HashMap::new();
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result, json!("2024"));
    }

    #[test]
    fn test_validate_function() {
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Validate Test".to_string(),
            "Test VALIDATE function".to_string(),
            r#"VALIDATE("ABC_1234", "^[A-Z]{3}_\d{4}$")"#.to_string()
        );

        rule.parse().unwrap();
        let context = HashMap::new();
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_regex_with_context() {
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Context Regex Test".to_string(),
            "Test regex with context variables".to_string(),
            r#"email MATCHES /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/"#.to_string()
        );

        rule.parse().unwrap();
        let mut context = HashMap::new();
        context.insert("email".to_string(), json!("alice@company.com"));
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_invalid_email() {
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Invalid Email Test".to_string(),
            "Test IS_EMAIL with invalid email".to_string(),
            r#"IS_EMAIL("not-an-email")"#.to_string()
        );

        rule.parse().unwrap();
        let context = HashMap::new();
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_regex_assignment() {
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Regex Assignment".to_string(),
            "Test regex with assignment".to_string(),
            r#"email_valid = IS_EMAIL("test@example.com")"#.to_string()
        );

        rule.parse().unwrap();
        let context = HashMap::new();
        let result = rule.evaluate(&context).unwrap();
        // For assignments, the result is a JSON object with the assignment
        assert_eq!(result, json!({"email_valid": true}));
    }
}