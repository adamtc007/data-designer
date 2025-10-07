use data_designer::{BusinessRule, RulesEngine, generate_test_context, get_sample_rules};
use std::collections::HashMap;

fn main() {
    println!("=== Data Designer - nom Parser Test Suite ===\n");

    let context = generate_test_context();

    println!("Test Context:");
    println!("{:#?}\n", context);

    // Test individual rules
    println!("=== Testing Individual Rules ===\n");

    let sample_rules = get_sample_rules();

    for rule in &sample_rules {
        println!("Testing Rule: {} - {}", rule.id, rule.name);
        println!("Rule Text: {}", rule.rule_text);

        let mut test_rule = rule.clone();
        match test_rule.parse() {
            Ok(_) => {
                println!("✓ Parse successful");
                println!("AST: {:#?}", test_rule.ast);

                match test_rule.evaluate(&context) {
                    Ok(result) => {
                        println!("✓ Evaluation Result: {}", result);
                    }
                    Err(e) => {
                        println!("✗ Evaluation Error: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("✗ Parse Error: {}", e);
            }
        }
        println!("---\n");
    }

    // Test Rules Engine
    println!("=== Testing Rules Engine ===\n");

    let mut engine = RulesEngine::new();

    // Add all sample rules to the engine
    for rule in sample_rules {
        match engine.add_rule(rule.clone()) {
            Ok(_) => println!("✓ Added rule: {}", rule.id),
            Err(e) => println!("✗ Failed to add rule {}: {}", rule.id, e),
        }
    }

    println!("\n=== Evaluating All Rules ===\n");

    let results = engine.evaluate_all(&context);
    for (rule_id, result) in results {
        match result {
            Ok(value) => println!("Rule {}: ✓ Result = {}", rule_id, value),
            Err(e) => println!("Rule {}: ✗ Error = {}", rule_id, e),
        }
    }

    // Test specific expressions
    println!("\n=== Testing Specific Expressions ===\n");

    test_expression("10 + 20 * 3", &context);
    test_expression("(10 + 20) * 3", &context);
    test_expression(r#""Hello " & "World""#, &context);
    test_expression("price * quantity", &context);
    test_expression("100 / 0", &context); // Division by zero test
    test_expression("not true", &context);
    test_expression("5 > 3 and 10 < 20", &context);
    test_expression("[1, 2, 3, 4]", &context);
    test_expression("UPPER(name)", &context);
    test_expression("LEN(user_id)", &context);
}

fn test_expression(expr: &str, context: &HashMap<String, serde_json::Value>) {
    println!("Expression: {}", expr);

    let mut rule = BusinessRule::new(
        "test".to_string(),
        "Test".to_string(),
        "Test expression".to_string(),
        expr.to_string(),
    );

    match rule.parse() {
        Ok(_) => {
            match rule.evaluate(context) {
                Ok(result) => println!("  → Result: {}", result),
                Err(e) => println!("  → Error: {}", e),
            }
        }
        Err(e) => println!("  → Parse Error: {}", e),
    }
}