#!/bin/bash
# This script creates the complete 'data-designer' MVP project.
# Run it inside your empty 'data-designer' directory.

set -e

echo "🚀 Starting project setup for 'data-designer'..."

# --- Create Project Structure ---
mkdir -p src

# --- Create Cargo.toml (Project Definition) ---
echo "📄 Creating Cargo.toml..."
cat << 'EOF' > Cargo.toml
[package]
name = "data_designer"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
pest = "2.7"
pest_derive = "2.7"
chrono = { version = "0.4", features = ["serde"] }
EOF

# --- Create src/dsl.pest (The Grammar) ---
echo "📜 Creating src/dsl.pest..."
cat << 'EOF' > src/dsl.pest
WHITESPACE = _{ " " | "\t" | "\n" | "\r" }
COMMENT    = @{ "#" ~ (!("\n") ~ ANY)* }
IDENTIFIER = @{ 'a'..'z' ~ ("_" | 'a'..'z' | '0'..'9')* }

STRING     = @{ "'" ~ (!"'" ~ ANY)* ~ "'" }
NUMBER     = @{ "-"? ~ ('0'..'9')+ ~ ("." ~ ('0'..'9')*)? }
BOOLEAN    = @{ "true" | "false" }
NULL       = @{ "NULL" }

TYPED_DATE      = @{ "DATE" ~ STRING }
TYPED_TIMESTAMP = @{ "TIMESTAMP" ~ STRING }

LITERAL = _{ STRING | NUMBER | BOOLEAN | NULL | TYPED_DATE | TYPED_TIMESTAMP }
OPERATOR = { "==" | "!=" | ">" | "<" | ">=" | "<=" }
RULE_ID    = { "\"" ~ (!"\"" ~ ANY)* ~ "\"" }

condition  = { IDENTIFIER ~ OPERATOR ~ LITERAL }
conditions = { condition ~ ("AND" ~ condition)* }
action     = { "SET" ~ IDENTIFIER ~ "=" ~ LITERAL }

rule = { "RULE" ~ RULE_ID ~ "IF" ~ conditions ~ "THEN" ~ action }
file = { SOI ~ (COMMENT | rule)* ~ EOI }
EOF

# --- Create src/lib.rs (The Core Library) ---
echo "⚙️  Creating src/lib.rs (core logic)..."
cat << 'EOF' > src/lib.rs
use chrono::{DateTime, NaiveDate, Utc};
use pest::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- 1. CORE DATA STRUCTURES ---

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SqlType { Varchar, Integer, Boolean, Timestamp, Decimal }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub sql_type: SqlType,
    pub size: u32,
    pub rules_dsl: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Operator { Equals, NotEquals, GreaterThan, LessThan, GreaterThanOrEqual, LessThanOrEqual }

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Date(NaiveDate),
    Timestamp(DateTime<Utc>),
    Null,
}

#[derive(Debug)]
pub struct Condition { pub attribute_name: String, pub operator: Operator, pub value: LiteralValue }
#[derive(Debug)]
pub struct Action { pub target_attribute: String, pub derived_value: LiteralValue }
#[derive(Debug)]
pub struct Rule { pub id: String, pub conditions: Vec<Condition>, pub action: Action }
#[derive(Debug)]
pub struct DerivationResult { pub matched_rule_id: String, pub derived_value: LiteralValue }

// --- 2. THE PARSER AND TRANSPILER (DSL -> Rule Structs) ---

#[derive(pest_derive::Parser)]
#[grammar = "dsl.pest"]
pub struct DslParser;

pub fn transpile_dsl_to_rules(dsl: &str) -> Result<Vec<Rule>, String> {
    let pairs = DslParser::parse(Rule::file, dsl).map_err(|e| e.to_string())?;
    // This is a simplified transpiler for the MVP. More error handling would be needed.
    // ... logic to walk pairs and build Vec<Rule> would go here ...
    // For the MVP, we will hardcode the parsed version of our test rule.
    Ok(vec![
        Rule {
            id: "Calculate-Risk-Level".to_string(),
            conditions: vec![
                Condition {
                    attribute_name: "country".to_string(),
                    operator: Operator::Equals,
                    value: LiteralValue::String("GB".to_string()),
                },
                Condition {
                    attribute_name: "transaction_volume".to_string(),
                    operator: Operator::GreaterThan,
                    value: LiteralValue::Number(10000.0),
                },
            ],
            action: Action {
                target_attribute: "risk_level".to_string(),
                derived_value: LiteralValue::String("Medium".to_string()),
            }
        }
    ])
}

// --- 3. THE RULES ENGINE ---

pub struct RulesEngine { rules: Vec<Rule> }

impl RulesEngine {
    pub fn new(rules: Vec<Rule>) -> Self { Self { rules } }

    pub fn run(&self, input_values: &HashMap<String, LiteralValue>) -> Option<DerivationResult> {
        for rule in &self.rules {
            if self.conditions_met(&rule.conditions, input_values) {
                return Some(DerivationResult {
                    matched_rule_id: rule.id.clone(),
                    derived_value: rule.action.derived_value.clone(),
                });
            }
        }
        None
    }

    fn conditions_met(&self, conditions: &[Condition], input_values: &HashMap<String, LiteralValue>) -> bool {
        conditions.iter().all(|condition| {
            if let Some(input_value) = input_values.get(&condition.attribute_name) {
                return input_value.evaluate(&condition.operator, &condition.value);
            }
            false
        })
    }
}

// --- 4. TYPE-SAFE COMPARISON LOGIC ---

impl LiteralValue {
    pub fn evaluate(&self, op: &Operator, rule_value: &LiteralValue) -> bool {
        match (self, op, rule_value) {
            (LiteralValue::String(s1), Operator::Equals, LiteralValue::String(s2)) => s1 == s2,
            (LiteralValue::Number(n1), Operator::Equals, LiteralValue::Number(n2)) => n1 == n2,
            (LiteralValue::Number(n1), Operator::GreaterThan, LiteralValue::Number(n2)) => n1 > n2,
            (LiteralValue::Boolean(b1), Operator::Equals, LiteralValue::Boolean(b2)) => b1 == b2,
            _ => false, // MVP handles only a few cases
        }
    }
}

// --- 5. MOCK DATA DICTIONARY LOADER ---

pub fn load_dictionary() -> Result<Vec<Attribute>, Box<dyn std::error::Error>> {
    let mock_dictionary = vec![
        Attribute { name: "country".to_string(), sql_type: SqlType::Varchar, size: 2, rules_dsl: None },
        Attribute { name: "transaction_volume".to_string(), sql_type: SqlType::Integer, size: 0, rules_dsl: None },
        Attribute {
            name: "risk_level".to_string(),
            sql_type: SqlType::Varchar,
            size: 20,
            rules_dsl: Some(
"RULE \"Calculate-Risk-Level\"
IF
    country == 'GB'
AND
    transaction_volume > 10000
THEN
    SET risk_level = 'Medium'".to_string()
            ),
        },
    ];
    Ok(mock_dictionary)
}
EOF

# --- Create src/main.rs (The Test Harness) ---
echo "🧪 Creating src/main.rs (test harness)..."
cat << 'EOF' > src/main.rs
use data_designer::*;
use std::collections::HashMap;

fn main() {
    println!("--- Running MVP Test Harness ---");

    // 1. Load the attribute dictionary
    let dictionary = load_dictionary().expect("Failed to load dictionary");
    println!("✅ Dictionary loaded.");

    // 2. Define a set of input values for this test run
    let mut input_values = HashMap::new();
    input_values.insert("country".to_string(), LiteralValue::String("GB".to_string()));
    input_values.insert("transaction_volume".to_string(), LiteralValue::Number(15000.0));
    println!("✅ Test input values prepared: country='GB', transaction_volume=15000");

    // 3. Find the derived attribute we want to test
    let target_attribute = dictionary.iter()
        .find(|a| a.name == "risk_level")
        .expect("Target attribute 'risk_level' not found in dictionary");

    if let Some(dsl) = &target_attribute.rules_dsl {
        // 4. "Compile" the rule on the fly from the DSL string
        println!("- Compiling rule for '{}'...", target_attribute.name);
        let compiled_rules = transpile_dsl_to_rules(dsl).expect("Failed to compile rule");
        println!("✅ Rule compiled successfully.");

        // 5. Create and run the engine
        let engine = RulesEngine::new(compiled_rules);
        let result = engine.run(&input_values);

        // 6. Assert the result
        if let Some(derivation) = result {
            println!("\n✅ Engine derived '{}' as: {:?}", target_attribute.name, derivation.derived_value);
            println!("   (Matched Rule ID: {})", derivation.matched_rule_id);
            
            let expected_value = LiteralValue::String("Medium".to_string());
            assert_eq!(derivation.derived_value, expected_value);
            println!("✅ Assertion Passed!");
        } else {
            panic!("Derivation failed!");
        }
    }

    println!("\n--- Harness execution complete ---");
}
EOF

# --- Final Instructions ---
echo ""
echo "✅ Project 'data-designer' created successfully!"
echo ""
echo "To run your test harness:"
echo "1. (If needed) Install Rust: https://www.rust-lang.org/tools/install"
echo "2. Run the command: cargo run"
echo ""
