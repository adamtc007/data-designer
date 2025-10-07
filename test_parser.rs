use data_designer::transpile_dsl_to_rules;

fn main() {
    println!("=== Testing Basic Pest DSL Parser ===\n");

    // Test 1: Simple literal assignment
    println!("Test 1: Simple literal assignment");
    let dsl1 = r#"RULE "Simple Assignment" IF status == "active" THEN level = 5"#;

    match transpile_dsl_to_rules(dsl1) {
        Ok(rules) => {
            println!("✅ Successfully parsed {} rules:", rules.len());
            for rule in rules {
                println!("   Rule: '{}' -> target: '{}', value: {:?}",
                    rule.id, rule.action.target_attribute, rule.action.derived_value);
            }
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }

    // Test 2: Addition of two numbers
    println!("\nTest 2: Addition of two numbers");
    let dsl2 = r#"RULE "Add Numbers" IF country == "US" THEN total = 100 + 25"#;

    match transpile_dsl_to_rules(dsl2) {
        Ok(rules) => {
            println!("✅ Successfully parsed {} rules:", rules.len());
            for rule in rules {
                println!("   Rule: '{}' -> target: '{}', value: {:?}",
                    rule.id, rule.action.target_attribute, rule.action.derived_value);
            }
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }

    // Test 3: Multiple conditions
    println!("\nTest 3: Multiple conditions");
    let dsl3 = r#"RULE "Multi Condition" IF country == "US" AND status == "active" THEN score = 10"#;

    match transpile_dsl_to_rules(dsl3) {
        Ok(rules) => {
            println!("✅ Successfully parsed {} rules:", rules.len());
            for rule in rules {
                println!("   Rule: '{}' has {} conditions", rule.id, rule.conditions.len());
                for (i, condition) in rule.conditions.iter().enumerate() {
                    println!("     Condition {}: {} == {:?}", i+1, condition.attribute_name, condition.value);
                }
                println!("   Action: {} = {:?}", rule.action.target_attribute, rule.action.derived_value);
            }
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }

    println!("\n=== Parser Testing Complete ===");
}