use data_designer::parser::{parse_rule, ASTNode, BinaryOperator};
use std::collections::HashMap;

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parse_numbers() {
        let (_, ast) = parse_rule("42").unwrap();
        assert_eq!(ast, ASTNode::Number(42.0));

        let (_, ast) = parse_rule("-3.14").unwrap();
        assert_eq!(ast, ASTNode::Number(-3.14));
    }

    #[test]
    fn test_parse_strings() {
        let (_, ast) = parse_rule("\"hello world\"").unwrap();
        assert_eq!(ast, ASTNode::String("hello world".to_string()));

        let (_, ast) = parse_rule("'single quotes'").unwrap();
        assert_eq!(ast, ASTNode::String("single quotes".to_string()));
    }

    #[test]
    fn test_parse_arithmetic() {
        let (_, ast) = parse_rule("2 + 3 * 4").unwrap();
        // Should parse as 2 + (3 * 4) due to precedence
        match ast {
            ASTNode::BinaryOp { op: BinaryOperator::Add, .. } => {},
            _ => panic!("Expected binary add operation"),
        }
    }

    #[test]
    fn test_parse_assignment() {
        let (_, ast) = parse_rule("result = 10 + 20").unwrap();
        match ast {
            ASTNode::Assignment { target, .. } => {
                assert_eq!(target, "result");
            },
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let (_, ast) = parse_rule("CONCAT(\"hello\", \" \", \"world\")").unwrap();
        match ast {
            ASTNode::FunctionCall { name, args } => {
                assert_eq!(name, "CONCAT");
                assert_eq!(args.len(), 3);
            },
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_evaluation() {
        let context = HashMap::from([
            ("x".to_string(), serde_json::json!(10)),
            ("y".to_string(), serde_json::json!(20)),
        ]);

        let (_, ast) = parse_rule("x + y * 2").unwrap();
        let result = data_designer::parser::evaluate_ast(&ast, &context).unwrap();
        assert_eq!(result.as_f64().unwrap(), 50.0);
    }
}