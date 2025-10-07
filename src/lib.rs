use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- 1. CORE DATA STRUCTURES ---

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SqlType { Varchar, Integer, Boolean, Timestamp, Decimal }

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Operator {
    Equals, NotEquals, GreaterThan, LessThan, GreaterThanOrEqual, LessThanOrEqual
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum LogicalOperator { And, Or }

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition { pub attribute_name: String, pub operator: Operator, pub value: LiteralValue }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action { pub target_attribute: String, pub derived_value: LiteralValue }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRule { pub id: String, pub conditions: Vec<Condition>, pub action: Action }

#[derive(Debug, Clone)]
pub struct DerivationResult { pub matched_rule_id: String, pub derived_value: LiteralValue }

// --- 2. EXTENDED DATA STRUCTURES FOR EXPRESSIONS ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    // Literals and variables
    Attribute(String),
    Number(f64),
    String(String),

    // Arithmetic operations
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),

    // String operations
    Concat(Box<Expression>, Box<Expression>),
    Substring(Box<Expression>, Box<Expression>, Box<Expression>), // (string, start, length)

    // Function calls
    ConcatMany(Vec<Expression>),
    Lookup(Box<Expression>, String), // (key, table_name)
}

impl Expression {
    pub fn evaluate(&self, context: &std::collections::HashMap<String, LiteralValue>) -> Result<LiteralValue, String> {
        match self {
            Expression::Attribute(name) => {
                context.get(name)
                    .cloned()
                    .ok_or_else(|| format!("Attribute '{}' not found", name))
            }
            Expression::Number(n) => Ok(LiteralValue::Number(*n)),
            Expression::String(s) => Ok(LiteralValue::String(s.clone())),

            Expression::Add(left, right) => {
                let l = left.evaluate(context)?;
                let r = right.evaluate(context)?;
                match (l, r) {
                    (LiteralValue::Number(a), LiteralValue::Number(b)) => Ok(LiteralValue::Number(a + b)),
                    _ => Err("Addition requires numeric values".to_string()),
                }
            }

            Expression::Subtract(left, right) => {
                let l = left.evaluate(context)?;
                let r = right.evaluate(context)?;
                match (l, r) {
                    (LiteralValue::Number(a), LiteralValue::Number(b)) => Ok(LiteralValue::Number(a - b)),
                    _ => Err("Subtraction requires numeric values".to_string()),
                }
            }

            Expression::Multiply(left, right) => {
                let l = left.evaluate(context)?;
                let r = right.evaluate(context)?;
                match (l, r) {
                    (LiteralValue::Number(a), LiteralValue::Number(b)) => Ok(LiteralValue::Number(a * b)),
                    _ => Err("Multiplication requires numeric values".to_string()),
                }
            }

            Expression::Divide(left, right) => {
                let l = left.evaluate(context)?;
                let r = right.evaluate(context)?;
                match (l, r) {
                    (LiteralValue::Number(a), LiteralValue::Number(b)) => {
                        if b == 0.0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(LiteralValue::Number(a / b))
                        }
                    }
                    _ => Err("Division requires numeric values".to_string()),
                }
            }

            Expression::Concat(left, right) => {
                let l = left.evaluate(context)?;
                let r = right.evaluate(context)?;
                let l_str = match l {
                    LiteralValue::String(s) => s,
                    LiteralValue::Number(n) => n.to_string(),
                    _ => return Err("Cannot concatenate non-string/numeric values".to_string()),
                };
                let r_str = match r {
                    LiteralValue::String(s) => s,
                    LiteralValue::Number(n) => n.to_string(),
                    _ => return Err("Cannot concatenate non-string/numeric values".to_string()),
                };
                Ok(LiteralValue::String(format!("{}{}", l_str, r_str)))
            }

            Expression::Substring(string_expr, start_expr, length_expr) => {
                let string_val = string_expr.evaluate(context)?;
                let start_val = start_expr.evaluate(context)?;
                let length_val = length_expr.evaluate(context)?;

                match (string_val, start_val, length_val) {
                    (LiteralValue::String(s), LiteralValue::Number(start), LiteralValue::Number(len)) => {
                        let start_idx = start as usize;
                        let length = len as usize;
                        if start_idx <= s.len() {
                            let end_idx = std::cmp::min(start_idx + length, s.len());
                            Ok(LiteralValue::String(s[start_idx..end_idx].to_string()))
                        } else {
                            Ok(LiteralValue::String(String::new()))
                        }
                    }
                    _ => Err("SUBSTRING requires (string, number, number)".to_string()),
                }
            }

            Expression::ConcatMany(expressions) => {
                let mut result = String::new();
                for expr in expressions {
                    let val = expr.evaluate(context)?;
                    match val {
                        LiteralValue::String(s) => result.push_str(&s),
                        LiteralValue::Number(n) => result.push_str(&n.to_string()),
                        _ => return Err("CONCAT function requires string or numeric values".to_string()),
                    }
                }
                Ok(LiteralValue::String(result))
            }

            Expression::Lookup(key_expr, table_name) => {
                let key_val = key_expr.evaluate(context)?;
                let key_str = match key_val {
                    LiteralValue::String(s) => s,
                    LiteralValue::Number(n) => n.to_string(),
                    _ => return Err("Lookup key must be string or number".to_string()),
                };

                // Simple mock lookup - in real implementation, this would query external data
                match table_name.as_str() {
                    "countries" => match key_str.as_str() {
                        "US" => Ok(LiteralValue::String("United States".to_string())),
                        "GB" => Ok(LiteralValue::String("United Kingdom".to_string())),
                        "CA" => Ok(LiteralValue::String("Canada".to_string())),
                        _ => Ok(LiteralValue::String("Unknown".to_string())),
                    },
                    "rates" => match key_str.as_str() {
                        "standard" => Ok(LiteralValue::Number(0.1)),
                        "premium" => Ok(LiteralValue::Number(0.15)),
                        "basic" => Ok(LiteralValue::Number(0.05)),
                        _ => Ok(LiteralValue::Number(0.0)),
                    },
                    _ => Err(format!("Unknown lookup table: {}", table_name)),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAction {
    pub target_attribute: String,
    pub expression: Expression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedRule {
    pub id: String,
    pub conditions: Vec<Condition>,
    pub action: EnhancedAction,
}

// --- 3. THE PEST PARSER ---

#[derive(Parser)]
#[grammar = "dsl.pest"]
pub struct DslParser;

// Type alias to avoid naming conflict with Pest-generated Rule enum

// --- 4. THE TRANSPILER ---

// Enhanced transpiler that creates rules with Expression-based actions
pub fn transpile_dsl_to_enhanced_rules(dsl: &str) -> Result<Vec<EnhancedRule>, String> {
    let pairs = DslParser::parse(Rule::file, dsl).map_err(|e| e.to_string())?;

    let mut rules = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::rule {
            let rule = parse_enhanced_rule(pair)?;
            rules.push(rule);
        } else if pair.as_rule() == Rule::file {
            // Recursively process file contents
            for inner_pair in pair.into_inner() {
                if inner_pair.as_rule() == Rule::rule {
                    let rule = parse_enhanced_rule(inner_pair)?;
                    rules.push(rule);
                }
            }
        }
    }

    Ok(rules)
}

// Backward compatibility transpiler
pub fn transpile_dsl_to_rules(dsl: &str) -> Result<Vec<BusinessRule>, String> {
    let pairs = DslParser::parse(Rule::file, dsl).map_err(|e| e.to_string())?;

    let mut rules = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::rule {
            let rule = parse_rule(pair)?;
            rules.push(rule);
        }
    }

    Ok(rules)
}

fn parse_enhanced_rule(pair: pest::iterators::Pair<Rule>) -> Result<EnhancedRule, String> {
    let mut inner = pair.into_inner();

    // Parse rule name (string)
    let name_pair = inner.next().ok_or("Missing rule name")?;
    let rule_id = name_pair.as_str().trim_matches('"').to_string();

    // Parse IF clause
    let if_clause = inner.next().ok_or("Missing IF clause")?;
    let conditions = parse_if_clause(if_clause)?;

    // Parse THEN clause
    let then_clause = inner.next().ok_or("Missing THEN clause")?;
    let action = parse_enhanced_then_clause(then_clause)?;

    Ok(EnhancedRule {
        id: rule_id,
        conditions,
        action,
    })
}

fn parse_rule(pair: pest::iterators::Pair<Rule>) -> Result<BusinessRule, String> {
    let mut inner = pair.into_inner();

    // Parse rule name (string)
    let name_pair = inner.next().ok_or("Missing rule name")?;
    let rule_id = name_pair.as_str().trim_matches('"').to_string();

    // Parse IF clause
    let if_clause = inner.next().ok_or("Missing IF clause")?;
    let conditions = parse_if_clause(if_clause)?;

    // Parse THEN clause
    let then_clause = inner.next().ok_or("Missing THEN clause")?;
    let action = parse_then_clause(then_clause)?;

    Ok(BusinessRule {
        id: rule_id,
        conditions,
        action,
    })
}

fn parse_if_clause(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Condition>, String> {
    let mut conditions = Vec::new();

    for condition_pair in pair.into_inner() {
        if condition_pair.as_rule() == Rule::condition {
            let condition = parse_condition(condition_pair)?;
            conditions.push(condition);
        }
    }

    Ok(conditions)
}

fn parse_condition(pair: pest::iterators::Pair<Rule>) -> Result<Condition, String> {
    let mut inner = pair.into_inner();

    let attr_name = inner.next().ok_or("Missing attribute name")?.as_str().to_string();
    let op_pair = inner.next().ok_or("Missing comparison operator")?;
    let value_pair = inner.next().ok_or("Missing condition value")?;

    let operator = match op_pair.as_rule() {
        Rule::comparison_op => match op_pair.as_str() {
            "==" => Operator::Equals,
            "!=" => Operator::NotEquals,
            ">" => Operator::GreaterThan,
            "<" => Operator::LessThan,
            ">=" => Operator::GreaterThanOrEqual,
            "<=" => Operator::LessThanOrEqual,
            _ => return Err("Invalid comparison operator".to_string()),
        }
        _ => return Err("Expected comparison operator".to_string()),
    };

    let value = match value_pair.as_rule() {
        Rule::identifier => LiteralValue::String(value_pair.as_str().to_string()),
        Rule::number => LiteralValue::Number(value_pair.as_str().parse().map_err(|_| "Invalid number")?),
        Rule::string => LiteralValue::String(value_pair.as_str().trim_matches('"').to_string()),
        _ => return Err("Invalid condition value".to_string()),
    };

    Ok(Condition {
        attribute_name: attr_name,
        operator,
        value,
    })
}

fn parse_enhanced_then_clause(pair: pest::iterators::Pair<Rule>) -> Result<EnhancedAction, String> {
    let assignment = pair.into_inner().next().ok_or("Missing assignment")?;
    parse_enhanced_assignment(assignment)
}

fn parse_then_clause(pair: pest::iterators::Pair<Rule>) -> Result<Action, String> {
    let assignment = pair.into_inner().next().ok_or("Missing assignment")?;
    parse_assignment(assignment)
}

fn parse_enhanced_assignment(pair: pest::iterators::Pair<Rule>) -> Result<EnhancedAction, String> {
    let mut inner = pair.into_inner();

    let target = inner.next().ok_or("Missing target attribute")?.as_str().to_string();
    let expression_pair = inner.next().ok_or("Missing expression")?;

    let expression = parse_expression(expression_pair)?;

    Ok(EnhancedAction {
        target_attribute: target,
        expression,
    })
}

fn parse_assignment(pair: pest::iterators::Pair<Rule>) -> Result<Action, String> {
    let mut inner = pair.into_inner();

    let target = inner.next().ok_or("Missing target attribute")?.as_str().to_string();
    let expression_pair = inner.next().ok_or("Missing expression")?;

    let derived_value = parse_expression_simple(expression_pair)?;

    Ok(Action {
        target_attribute: target,
        derived_value,
    })
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    match pair.as_rule() {
        Rule::expression => {
            let mut inner = pair.into_inner();
            let first_term = parse_term(inner.next().ok_or("Empty expression")?)?;

            let mut result = first_term;
            while let Some(op_pair) = inner.next() {
                let op_rule = op_pair.as_rule();
                let right_term = parse_term(inner.next().ok_or("Missing operand after operator")?)?;

                result = match op_rule {
                    Rule::add_op => Expression::Add(Box::new(result), Box::new(right_term)),
                    Rule::sub_op => Expression::Subtract(Box::new(result), Box::new(right_term)),
                    Rule::concat_op => Expression::Concat(Box::new(result), Box::new(right_term)),
                    _ => return Err(format!("Unexpected operator: {:?}", op_rule)),
                };
            }
            Ok(result)
        }
        _ => Err("Expected expression".to_string()),
    }
}

fn parse_term(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    match pair.as_rule() {
        Rule::term => {
            let mut inner = pair.into_inner();
            let first_primary = parse_primary(inner.next().ok_or("Empty term")?)?;

            let mut result = first_primary;
            while let Some(op_pair) = inner.next() {
                let op_rule = op_pair.as_rule();
                let right_primary = parse_primary(inner.next().ok_or("Missing operand after operator")?)?;

                result = match op_rule {
                    Rule::mul_op => Expression::Multiply(Box::new(result), Box::new(right_primary)),
                    Rule::div_op => Expression::Divide(Box::new(result), Box::new(right_primary)),
                    _ => return Err(format!("Unexpected term operator: {:?}", op_rule)),
                };
            }
            Ok(result)
        }
        Rule::primary => parse_primary(pair),
        _ => Err("Expected term or primary".to_string()),
    }
}

fn parse_primary(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    match pair.as_rule() {
        Rule::substring_fn => {
            let mut inner = pair.into_inner();
            let string_expr = parse_expression(inner.next().ok_or("Missing string in SUBSTRING")?)?;
            let start_expr = parse_expression(inner.next().ok_or("Missing start in SUBSTRING")?)?;
            let length_expr = parse_expression(inner.next().ok_or("Missing length in SUBSTRING")?)?;
            Ok(Expression::Substring(Box::new(string_expr), Box::new(start_expr), Box::new(length_expr)))
        }
        Rule::concat_fn => {
            let mut args = Vec::new();
            for arg_pair in pair.into_inner() {
                args.push(parse_expression(arg_pair)?);
            }
            Ok(Expression::ConcatMany(args))
        }
        Rule::lookup_fn => {
            let mut inner = pair.into_inner();
            let key_expr = parse_expression(inner.next().ok_or("Missing key in LOOKUP")?)?;
            let table_name = inner.next().ok_or("Missing table name in LOOKUP")?.as_str().trim_matches('"').to_string();
            Ok(Expression::Lookup(Box::new(key_expr), table_name))
        }
        Rule::identifier => Ok(Expression::Attribute(pair.as_str().to_string())),
        Rule::number => Ok(Expression::Number(pair.as_str().parse().map_err(|_| "Invalid number")?)),
        Rule::string => Ok(Expression::String(pair.as_str().trim_matches('"').to_string())),
        Rule::expression => parse_expression(pair), // Handle parenthesized expressions
        _ => Err(format!("Unexpected primary type: {:?}", pair.as_rule())),
    }
}

// Update the simple function for backward compatibility
fn parse_expression_simple(pair: pest::iterators::Pair<Rule>) -> Result<LiteralValue, String> {
    let expr = parse_expression(pair)?;

    // For simple cases where we can compute at compile time
    match expr {
        Expression::Number(n) => Ok(LiteralValue::Number(n)),
        Expression::String(s) => Ok(LiteralValue::String(s)),
        Expression::Add(left, right) => {
            match (left.as_ref(), right.as_ref()) {
                (Expression::Number(a), Expression::Number(b)) => Ok(LiteralValue::Number(a + b)),
                _ => Err("Complex expressions require runtime evaluation".to_string()),
            }
        }
        _ => Err("Complex expressions require runtime evaluation".to_string()),
    }
}

fn parse_atom(pair: pest::iterators::Pair<Rule>) -> Result<LiteralValue, String> {
    match pair.as_rule() {
        Rule::identifier => Ok(LiteralValue::String(pair.as_str().to_string())), // Will be resolved at runtime
        Rule::number => Ok(LiteralValue::Number(pair.as_str().parse().map_err(|_| "Invalid number")?)),
        Rule::string => Ok(LiteralValue::String(pair.as_str().trim_matches('"').to_string())),
        _ => Err(format!("Unexpected atom type: {:?}", pair.as_rule())),
    }
}

// --- 3. THE RULES ENGINE ---

pub struct EnhancedRulesEngine { rules: Vec<EnhancedRule> }

impl EnhancedRulesEngine {
    pub fn new(rules: Vec<EnhancedRule>) -> Self { Self { rules } }

    pub fn run(&self, input_values: &HashMap<String, LiteralValue>) -> Option<DerivationResult> {
        for rule in &self.rules {
            if self.conditions_met(&rule.conditions, input_values) {
                // Evaluate the expression to get the derived value
                match rule.action.expression.evaluate(input_values) {
                    Ok(derived_value) => {
                        return Some(DerivationResult {
                            matched_rule_id: rule.id.clone(),
                            derived_value,
                        });
                    }
                    Err(_) => continue, // Skip this rule if expression evaluation fails
                }
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

pub struct RulesEngine { rules: Vec<BusinessRule> }

impl RulesEngine {
    pub fn new(rules: Vec<BusinessRule>) -> Self { Self { rules } }

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
            // String comparisons
            (LiteralValue::String(s1), Operator::Equals, LiteralValue::String(s2)) => s1 == s2,
            (LiteralValue::String(s1), Operator::NotEquals, LiteralValue::String(s2)) => s1 != s2,

            // Numeric comparisons
            (LiteralValue::Number(n1), Operator::Equals, LiteralValue::Number(n2)) => n1 == n2,
            (LiteralValue::Number(n1), Operator::NotEquals, LiteralValue::Number(n2)) => n1 != n2,
            (LiteralValue::Number(n1), Operator::GreaterThan, LiteralValue::Number(n2)) => n1 > n2,
            (LiteralValue::Number(n1), Operator::LessThan, LiteralValue::Number(n2)) => n1 < n2,
            (LiteralValue::Number(n1), Operator::GreaterThanOrEqual, LiteralValue::Number(n2)) => n1 >= n2,
            (LiteralValue::Number(n1), Operator::LessThanOrEqual, LiteralValue::Number(n2)) => n1 <= n2,

            // Boolean comparisons
            (LiteralValue::Boolean(b1), Operator::Equals, LiteralValue::Boolean(b2)) => b1 == b2,
            (LiteralValue::Boolean(b1), Operator::NotEquals, LiteralValue::Boolean(b2)) => b1 != b2,

            // Null comparisons
            (LiteralValue::Null, Operator::Equals, LiteralValue::Null) => true,
            (LiteralValue::Null, Operator::NotEquals, LiteralValue::Null) => false,
            (LiteralValue::Null, Operator::Equals, _) => false,
            (LiteralValue::Null, Operator::NotEquals, _) => true,
            (_, Operator::Equals, LiteralValue::Null) => false,
            (_, Operator::NotEquals, LiteralValue::Null) => true,

            // Mixed type comparisons (convert to strings for equality)
            (LiteralValue::String(s), Operator::Equals, LiteralValue::Number(n)) => s == &n.to_string(),
            (LiteralValue::Number(n), Operator::Equals, LiteralValue::String(s)) => &n.to_string() == s,

            _ => false, // Unsupported comparison
        }
    }
}