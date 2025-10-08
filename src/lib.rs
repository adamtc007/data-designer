pub mod parser;
mod test_regex;

use parser::{parse_rule, ASTNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value as JsonValue;

// --- Core Data Structures ---

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SqlType {
    Varchar,
    Integer,
    Boolean,
    Timestamp,
    Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_text: String,
    pub ast: Option<ASTNode>,
}

impl BusinessRule {
    pub fn new(id: String, name: String, description: String, rule_text: String) -> Self {
        let ast = parse_rule(&rule_text)
            .ok()
            .map(|(_, node)| node);

        BusinessRule {
            id,
            name,
            description,
            rule_text,
            ast,
        }
    }

    pub fn parse(&mut self) -> Result<(), String> {
        match parse_rule(&self.rule_text) {
            Ok((remaining, ast)) => {
                if !remaining.trim().is_empty() {
                    return Err(format!("Unexpected input after parsing: '{}'", remaining));
                }
                self.ast = Some(ast);
                Ok(())
            }
            Err(e) => Err(format!("Parse error: {:?}", e))
        }
    }

    pub fn evaluate(&self, context: &HashMap<String, JsonValue>) -> Result<JsonValue, String> {
        self.ast
            .as_ref()
            .ok_or_else(|| "Rule not parsed".to_string())
            .and_then(|ast| ast.evaluate(context))
    }
}

// --- Rules Engine ---

#[derive(Debug, Clone)]
pub struct RulesEngine {
    rules: Vec<BusinessRule>,
}

impl RulesEngine {
    pub fn new() -> Self {
        RulesEngine {
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, mut rule: BusinessRule) -> Result<(), String> {
        rule.parse()?;
        self.rules.push(rule);
        Ok(())
    }

    pub fn evaluate_all(&self, context: &HashMap<String, JsonValue>) -> Vec<(String, Result<JsonValue, String>)> {
        self.rules
            .iter()
            .map(|rule| (rule.id.clone(), rule.evaluate(context)))
            .collect()
    }

    pub fn evaluate_rule(&self, rule_id: &str, context: &HashMap<String, JsonValue>) -> Result<JsonValue, String> {
        self.rules
            .iter()
            .find(|r| r.id == rule_id)
            .ok_or_else(|| format!("Rule '{}' not found", rule_id))
            .and_then(|rule| rule.evaluate(context))
    }
}

// --- Grammar Management ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrammarRule {
    pub name: String,
    pub definition: String,
    pub rule_type: String, // "normal", "silent", "atomic"
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrammarDefinition {
    pub metadata: GrammarMetadata,
    pub rules: Vec<GrammarRule>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrammarMetadata {
    pub version: String,
    pub description: String,
    pub created_by: String,
    pub created_date: String,
}

impl GrammarDefinition {
    pub fn new() -> Self {
        GrammarDefinition {
            metadata: GrammarMetadata {
                version: "1.0.0".to_string(),
                description: "Dynamic DSL Grammar using nom parser".to_string(),
                created_by: "System".to_string(),
                created_date: chrono::Utc::now().to_rfc3339(),
            },
            rules: vec![
                GrammarRule {
                    name: "expression".to_string(),
                    definition: "assignment | binary_op | function_call | literal".to_string(),
                    rule_type: "normal".to_string(),
                    description: "Main expression rule".to_string(),
                },
                GrammarRule {
                    name: "assignment".to_string(),
                    definition: "identifier '=' expression".to_string(),
                    rule_type: "normal".to_string(),
                    description: "Variable assignment".to_string(),
                },
                GrammarRule {
                    name: "binary_op".to_string(),
                    definition: "expression operator expression".to_string(),
                    rule_type: "normal".to_string(),
                    description: "Binary operations".to_string(),
                },
                GrammarRule {
                    name: "operator".to_string(),
                    definition: "'+' | '-' | '*' | '/' | '&' | '==' | '!=' | '<' | '>' | '<=' | '>=' | 'and' | 'or'".to_string(),
                    rule_type: "normal".to_string(),
                    description: "Supported operators".to_string(),
                },
                GrammarRule {
                    name: "function_call".to_string(),
                    definition: "identifier '(' argument_list? ')'".to_string(),
                    rule_type: "normal".to_string(),
                    description: "Function invocation".to_string(),
                },
                GrammarRule {
                    name: "literal".to_string(),
                    definition: "number | string | boolean | identifier".to_string(),
                    rule_type: "normal".to_string(),
                    description: "Literal values".to_string(),
                },
            ],
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let grammar = serde_json::from_str(&content)?;
        Ok(grammar)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn add_rule(&mut self, rule: GrammarRule) {
        self.rules.push(rule);
    }

    pub fn update_rule(&mut self, name: &str, rule: GrammarRule) -> Result<(), String> {
        if let Some(existing) = self.rules.iter_mut().find(|r| r.name == name) {
            *existing = rule;
            Ok(())
        } else {
            Err(format!("Rule '{}' not found", name))
        }
    }

    pub fn get_rule(&self, name: &str) -> Option<&GrammarRule> {
        self.rules.iter().find(|r| r.name == name)
    }

    pub fn list_rules(&self) -> Vec<String> {
        self.rules.iter().map(|r| r.name.clone()).collect()
    }
}

// --- Test Data Generator ---

pub fn generate_test_context() -> HashMap<String, JsonValue> {
    let mut context = HashMap::new();

    // Basic variables
    context.insert("name".to_string(), JsonValue::String("Alice".to_string()));
    context.insert("age".to_string(), JsonValue::Number(serde_json::Number::from(30)));
    context.insert("price".to_string(), JsonValue::Number(serde_json::Number::from_f64(29.99).unwrap()));
    context.insert("quantity".to_string(), JsonValue::Number(serde_json::Number::from(5)));
    context.insert("tax".to_string(), JsonValue::Number(serde_json::Number::from_f64(0.08).unwrap()));

    // For function tests
    context.insert("user_id".to_string(), JsonValue::String("USR123456".to_string()));
    context.insert("country_code".to_string(), JsonValue::String("US".to_string()));
    context.insert("tier".to_string(), JsonValue::String("premium".to_string()));
    context.insert("base_rate".to_string(), JsonValue::Number(serde_json::Number::from_f64(0.05).unwrap()));
    context.insert("role".to_string(), JsonValue::String("Admin".to_string()));

    context
}

// --- Sample Rules ---

pub fn get_sample_rules() -> Vec<BusinessRule> {
    vec![
        BusinessRule::new(
            "rule1".to_string(),
            "Simple Math".to_string(),
            "Basic arithmetic operations".to_string(),
            "result = 100 + 25 * 2 - 10 / 2".to_string(),
        ),
        BusinessRule::new(
            "rule2".to_string(),
            "String Concatenation".to_string(),
            "Join strings together".to_string(),
            r#"message = "Hello " & name & "!""#.to_string(),
        ),
        BusinessRule::new(
            "rule3".to_string(),
            "Complex Expression".to_string(),
            "Parentheses and precedence".to_string(),
            "(100 + 50) * 2".to_string(),
        ),
        BusinessRule::new(
            "rule4".to_string(),
            "Function Call".to_string(),
            "Using CONCAT function".to_string(),
            r#"CONCAT("User: ", name, " (", role, ")")"#.to_string(),
        ),
        BusinessRule::new(
            "rule5".to_string(),
            "Substring Function".to_string(),
            "Extract part of a string".to_string(),
            "SUBSTRING(user_id, 0, 3)".to_string(),
        ),
        BusinessRule::new(
            "rule6".to_string(),
            "Lookup Function".to_string(),
            "External data lookup".to_string(),
            r#"LOOKUP(country_code, "countries")"#.to_string(),
        ),
        BusinessRule::new(
            "rule7".to_string(),
            "Complex Calculation".to_string(),
            "Mixed operations and functions".to_string(),
            r#"CONCAT("Rate: ", (base_rate + LOOKUP(tier, "rates")) * 100, "%")"#.to_string(),
        ),
        BusinessRule::new(
            "rule8".to_string(),
            "Runtime Variables".to_string(),
            "Using context variables".to_string(),
            "price * quantity + tax".to_string(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_arithmetic() {
        let context = generate_test_context();
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Test".to_string(),
            "Test rule".to_string(),
            "10 + 20 * 3".to_string(),
        );

        assert!(rule.parse().is_ok());
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result, JsonValue::Number(serde_json::Number::from_f64(70.0).unwrap()));
    }

    #[test]
    fn test_string_concatenation() {
        let context = generate_test_context();
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Test".to_string(),
            "Test rule".to_string(),
            r#""Hello " & "World""#.to_string(),
        );

        assert!(rule.parse().is_ok());
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result, JsonValue::String("Hello World".to_string()));
    }

    #[test]
    fn test_variable_reference() {
        let context = generate_test_context();
        let mut rule = BusinessRule::new(
            "test".to_string(),
            "Test".to_string(),
            "Test rule".to_string(),
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
            "Test rule".to_string(),
            r#"UPPER(name)"#.to_string(),
        );

        assert!(rule.parse().is_ok());
        let result = rule.evaluate(&context).unwrap();
        assert_eq!(result, JsonValue::String("ALICE".to_string()));
    }

    #[test]
    fn test_rules_engine() {
        let context = generate_test_context();
        let mut engine = RulesEngine::new();

        for rule in get_sample_rules().into_iter().take(3) {
            engine.add_rule(rule).unwrap();
        }

        let results = engine.evaluate_all(&context);
        assert_eq!(results.len(), 3);

        // Check that all rules evaluated without error
        for (rule_id, result) in results {
            assert!(result.is_ok(), "Rule {} failed", rule_id);
        }
    }
}