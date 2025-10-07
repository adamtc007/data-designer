use data_designer::*;
use std::collections::HashMap;

fn main() {
    println!("=== Testing Enhanced Pest DSL Parser with All 5 Extensions ===\n");

    // Test 1: Basic arithmetic with multiple operators
    println!("Test 1: Multiple arithmetic operators (-, *, /, +)");
    let dsl1 = r#"RULE "Complex Math" IF status == "active" THEN result = 100 + 25 * 2 - 10 / 2"#;
    test_enhanced_rule(dsl1, "Complex Math", |engine| {
        let mut context = HashMap::new();
        context.insert("status".to_string(), LiteralValue::String("active".to_string()));
        engine.run(&context)
    });

    // Test 2: String concatenation with & operator
    println!("\nTest 2: String concatenation with & operator");
    let dsl2 = r#"RULE "Concat String" IF country == "US" THEN message = "Hello " & name & "!""#;
    test_enhanced_rule(dsl2, "Concat String", |engine| {
        let mut context = HashMap::new();
        context.insert("country".to_string(), LiteralValue::String("US".to_string()));
        context.insert("name".to_string(), LiteralValue::String("World".to_string()));
        engine.run(&context)
    });

    // Test 3: Parentheses for precedence
    println!("\nTest 3: Parentheses for operator precedence");
    let dsl3 = r#"RULE "Precedence Test" IF level > 5 THEN total = (100 + 50) * 2"#;
    test_enhanced_rule(dsl3, "Precedence Test", |engine| {
        let mut context = HashMap::new();
        context.insert("level".to_string(), LiteralValue::Number(10.0));
        engine.run(&context)
    });

    // Test 4: SUBSTRING function
    println!("\nTest 4: SUBSTRING function");
    let dsl4 = r#"RULE "Extract Code" IF type == "user" THEN code = SUBSTRING(user_id, 0, 3)"#;
    test_enhanced_rule(dsl4, "Extract Code", |engine| {
        let mut context = HashMap::new();
        context.insert("type".to_string(), LiteralValue::String("user".to_string()));
        context.insert("user_id".to_string(), LiteralValue::String("USR12345".to_string()));
        engine.run(&context)
    });

    // Test 5: CONCAT function with multiple arguments
    println!("\nTest 5: CONCAT function with multiple arguments");
    let dsl5 = r#"RULE "Build Message" IF active == "true" THEN full_message = CONCAT("User: ", name, " (", role, ")")"#;
    test_enhanced_rule(dsl5, "Build Message", |engine| {
        let mut context = HashMap::new();
        context.insert("active".to_string(), LiteralValue::String("true".to_string()));
        context.insert("name".to_string(), LiteralValue::String("Alice".to_string()));
        context.insert("role".to_string(), LiteralValue::String("Admin".to_string()));
        engine.run(&context)
    });

    // Test 6: LOOKUP function
    println!("\nTest 6: LOOKUP function with external data");
    let dsl6 = r#"RULE "Country Lookup" IF region == "NA" THEN country_name = LOOKUP(country_code, "countries")"#;
    test_enhanced_rule(dsl6, "Country Lookup", |engine| {
        let mut context = HashMap::new();
        context.insert("region".to_string(), LiteralValue::String("NA".to_string()));
        context.insert("country_code".to_string(), LiteralValue::String("US".to_string()));
        engine.run(&context)
    });

    // Test 7: Complex expression combining multiple features
    println!("\nTest 7: Complex expression with mixed operations");
    let dsl7 = r#"RULE "Ultimate Test" IF status == "premium" THEN result = CONCAT("Rate: ", (base_rate + LOOKUP(tier, "rates")) * 100, "%")"#;
    test_enhanced_rule(dsl7, "Ultimate Test", |engine| {
        let mut context = HashMap::new();
        context.insert("status".to_string(), LiteralValue::String("premium".to_string()));
        context.insert("base_rate".to_string(), LiteralValue::Number(0.05));
        context.insert("tier".to_string(), LiteralValue::String("premium".to_string()));
        engine.run(&context)
    });

    // Test 8: Runtime attribute resolution
    println!("\nTest 8: Runtime attribute resolution");
    let dsl8 = r#"RULE "Runtime Calc" IF enabled == "yes" THEN computed = price * quantity + tax"#;
    test_enhanced_rule(dsl8, "Runtime Calc", |engine| {
        let mut context = HashMap::new();
        context.insert("enabled".to_string(), LiteralValue::String("yes".to_string()));
        context.insert("price".to_string(), LiteralValue::Number(10.50));
        context.insert("quantity".to_string(), LiteralValue::Number(3.0));
        context.insert("tax".to_string(), LiteralValue::Number(2.15));
        engine.run(&context)
    });

    println!("\n=== Enhanced Parser Testing Complete ===");
}

fn test_enhanced_rule<F>(dsl: &str, test_name: &str, test_fn: F)
where
    F: Fn(&EnhancedRulesEngine) -> Option<DerivationResult>
{
    match transpile_dsl_to_enhanced_rules(dsl) {
        Ok(rules) => {
            println!("✅ Successfully parsed rule: '{}'", test_name);
            if rules.len() > 0 {
                let engine = EnhancedRulesEngine::new(rules);
                match test_fn(&engine) {
                    Some(result) => {
                        println!("   ✅ Rule executed successfully!");
                        println!("   📊 Result: {:?}", result.derived_value);
                    }
                    None => {
                        println!("   ⚠️  Rule conditions not met or execution failed");
                    }
                }
            } else {
                println!("   ⚠️  No rules generated from DSL");
            }
        }
        Err(e) => {
            println!("❌ Parse error for '{}': {}", test_name, e);
        }
    }
}