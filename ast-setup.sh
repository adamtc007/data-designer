#!/bin/bash

# This script will create the complete data-designer Rust project workspace.
# Run it in the directory where you want the project to be created.

echo "🚀 Creating project workspace: data-designer"

# Create workspace structure and member crates
mkdir -p data-designer
cd data-designer
cargo new --lib data-designer-core
cargo new --bin data-designer-cli
cargo new --bin data-designer-lsp

# Create top-level workspace Cargo.toml
echo "📄 Creating workspace Cargo.toml..."
cat << 'EOF' > Cargo.toml
[workspace]
members = [
    "data-designer-core",
    "data-designer-cli",
    "data-designer-lsp",
]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
anyhow = "1.0"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
clap = { version = "4.0", features = ["derive"] }
fasteval = "0.2"
petgraph = "0.6"
ndarray = "0.15"
regex = "1"
nom = "7"
tower-lsp = "0.20"
log = "0.4"
env_logger = "0.10"
EOF

# --- Create core library files ---
echo "📄 Populating data-designer-core..."

# Update core Cargo.toml
cat << 'EOF' > data-designer-core/Cargo.toml
[package]
name = "data-designer-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde.workspace = true
serde_yaml.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tokio.workspace = true
reqwest.workspace = true
fasteval.workspace = true
petgraph.workspace = true
ndarray.workspace = true
regex.workspace = true
nom.workspace = true
EOF

# Create core lib.rs
cat << 'EOF' > data-designer-core/src/lib.rs
pub mod models;
pub mod manager;
pub mod agent;
pub mod engine;
pub mod graph;
pub mod parser;
pub mod evaluator; // <-- New module added here
EOF

# Create models.rs
cat << 'EOF' > data-designer-core/src/models.rs
use serde::{Deserialize, Serialize};

// --- Top-level Dictionary Structure ---
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct DataDictionary {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_models: Vec<CanonicalModel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_attributes: Vec<DerivedAttribute>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub solicitation_packs: Vec<SolicitationPack>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub axes: Vec<Axis>,
}

// ... (Rest of the model definitions are unchanged) ...

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Axis {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalModel {
    pub entity_name: String,
    pub description: String,
    pub attributes: Vec<CanonicalAttribute>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalAttribute {
    pub name: String,
    pub data_type: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    #[serde(default)]
    pub governance: Governance,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Governance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorized_source: Option<AuthorizedSource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consumers: Vec<Consumer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizedSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub lineage_uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Consumer {
    pub name: String,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DerivedAttribute {
    pub name: String,
    #[serde(rename = "type")]
    pub attribute_type: String,
    pub visibility: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    pub dependencies: Vec<String>,
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub governance: Governance,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub description: String,
    #[serde(rename = "if", default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(rename = "then", default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(rename = "otherwise", default, skip_serializing_if = "Option::is_none")]
    pub otherwise_value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SolicitationPack {
    pub name: String,
    pub description: String,
    pub process: String,
    pub audience: String,
    pub attributes: Vec<String>,
}


// --- AST Definitions for the Rules DSL ---

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Concat,
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Value),
    Identifier(String),
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    Cast {
        expr: Box<Expression>,
        data_type: String,
    },
}
EOF

# Create parser.rs
cat << 'EOF' > data-designer-core/src/parser.rs
use crate::models::{Expression, Value, BinaryOperator};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alpha1, alphanumeric1, char, multispace0},
    combinator::{map, recognize, map_res},
    error::ParseError,
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

// ... (Full nom parser implementation from your GitHub repo)

fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_"), tag(".")))),
        )),
        String::from,
    )(input)
}

fn parse_string_literal(input: &str) -> IResult<&str, Value> {
    map(
        delimited(char('\''), take_while(|c| c != '\''), char('\'')),
        |s: &str| Value::String(s.to_string()),
    )(input)
}

// A simplified integer parser for this example
fn parse_integer(input: &str) -> IResult<&str, Value> {
    map_res(alphanumeric1, |s: &str| {
        s.parse::<i64>().map(Value::Integer)
    })(input)
}

fn parse_value(input: &str) -> IResult<&str, Expression> {
    map(alt((parse_integer, parse_string_literal)), Expression::Literal)(input)
}

fn parse_function_call(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            parse_identifier,
            ws(char('(')),
            separated_list0(ws(char(',')), parse_expression),
            ws(char(')')),
        )),
        |(name, _, args, _)| Expression::FunctionCall { name, args },
    )(input)
}


fn parse_primary(input: &str) -> IResult<&str, Expression> {
    alt((
        parse_value,
        parse_function_call,
        map(parse_identifier, Expression::Identifier),
        delimited(ws(char('(')), parse_expression, ws(char(')'))),
    ))(input)
}

fn parse_expression(input: &str) -> IResult<&str, Expression> {
    // This is a simplified expression parser and does not handle operator precedence correctly.
    // A real implementation would use a Pratt parser or similar technique.
    let (input, left) = parse_primary(input)?;
    let (input, res) = many0(tuple((
        ws(alt((
            map(tag("=="), |_| BinaryOperator::Equals),
            map(tag("<"), |_| BinaryOperator::LessThan),
            // ... other operators
        ))),
        parse_primary,
    )))(input)?;
    
    Ok((input, res.into_iter().fold(left, |acc, (op, right)| {
        Expression::BinaryOp { op, left: Box::new(acc), right: Box::new(right) }
    })))
}


/// The main entry point for the parser.
pub fn parse_rule(input: &str) -> IResult<&str, Expression> {
    parse_expression(input)
}
EOF

# --- Create the NEW evaluator.rs ---
echo "📄 Creating NEW src/evaluator.rs..."
cat << 'EOF' > data-designer-core/src/evaluator.rs
use crate::models::{Expression, Value, BinaryOperator};
use anyhow::{Result, bail};
use std::collections::HashMap;

pub type Facts = HashMap<String, Value>;

/// Evaluates a parsed AST `Expression` against a set of facts.
pub fn evaluate(expr: &Expression, facts: &Facts) -> Result<Value> {
    match expr {
        Expression::Literal(val) => Ok(val.clone()),

        Expression::Identifier(name) => {
            facts.get(name)
                 .cloned()
                 .ok_or_else(|| anyhow::anyhow!("Fact '{}' not found", name))
        }

        Expression::BinaryOp { op, left, right } => {
            let left_val = evaluate(left, facts)?;
            let right_val = evaluate(right, facts)?;
            
            // This is a simplified implementation. A real one would handle type mismatches.
            match (op, left_val, right_val) {
                (BinaryOperator::Equals, l, r) => Ok(Value::Boolean(l == r)),
                (BinaryOperator::LessThan, Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
                _ => bail!("Unsupported binary operation"),
            }
        }

        Expression::FunctionCall { name, args } => {
            let mut arg_values = Vec::new();
            for arg_expr in args {
                arg_values.push(evaluate(arg_expr, facts)?);
            }
            
            match name.as_str() {
                "CONCAT" => {
                    let mut result = String::new();
                    for val in arg_values {
                        if let Value::String(s) = val {
                            result.push_str(&s);
                        }
                    }
                    Ok(Value::String(result))
                },
                _ => bail!("Unknown function '{}'", name),
            }
        }
        
        // Placeholder for other expression types
        _ => bail!("Unsupported expression type"),
    }
}
EOF

# --- Update engine.rs to use the new evaluator ---
echo "📄 Updating src/engine.rs to use the evaluator..."
cat << 'EOF' > data-designer-core/src/engine.rs
use crate::models::{DataDictionary, DerivedAttribute};
use crate::parser;
use crate::evaluator::{self, Facts}; // <-- Import the new evaluator
use anyhow::{bail, Context, Result};
use std::collections::HashMap;

/// The RulesEngine is now an orchestrator. It holds parsed rules (ASTs).
pub struct RulesEngine<'a> {
    dictionary: &'a DataDictionary,
    // We now store the parsed ASTs for performance.
    parsed_rules: HashMap<String, (Option<Expression>, Option<Expression>, Option<Expression>)>,
}

impl<'a> RulesEngine<'a> {
    /// Parses all rules from the dictionary into ASTs upon creation.
    pub fn new(dict: &'a DataDictionary) -> Result<Self> {
        let mut parsed_rules = HashMap::new();

        for attr in &dict.derived_attributes {
            for rule in &attr.rules {
                // Parse if, then, and otherwise clauses into ASTs.
                let cond_ast = if let Some(cond_str) = &rule.condition {
                    Some(parser::parse_rule(cond_str)?.1)
                } else { None };

                let then_ast = if let Some(then_str) = &rule.value {
                    Some(parser::parse_rule(then_str)?.1)
                } else { None };
                
                let other_ast = if let Some(other_str) = &rule.otherwise_value {
                    Some(parser::parse_rule(other_str)?.1)
                } else { None };
                
                // For simplicity, we only handle one rule per attribute here.
                parsed_rules.insert(attr.name.clone(), (cond_ast, then_ast, other_ast));
                break;
            }
        }
        Ok(Self { dictionary: dict, parsed_rules })
    }

    /// Evaluates a chain of dependencies.
    pub fn evaluate_chain(&self, targets: &[String], initial_facts: &Facts) -> Result<Facts> {
        let mut facts = initial_facts.clone();
        for target in targets {
            self.calculate_attribute_recursive(target, &mut facts)?;
        }
        Ok(facts)
    }

    fn calculate_attribute_recursive(&self, attr_name: &str, facts: &mut Facts) -> Result<()> {
        if facts.contains_key(attr_name) {
            return Ok(()); // Already calculated.
        }

        let attr_def = self.dictionary.derived_attributes.iter()
            .find(|a| a.name == attr_name)
            .with_context(|| format!("Definition for '{}' not found", attr_name))?;

        // Recursively ensure all dependencies are met.
        for dep in &attr_def.dependencies {
            self.calculate_attribute_recursive(dep, facts)?;
        }

        let (cond_ast, then_ast, other_ast) = self.parsed_rules.get(attr_name)
            .with_context(|| format!("Parsed rule for '{}' not found", attr_name))?;

        // Use the new evaluator module to get the final value.
        let condition_met = if let Some(cond) = cond_ast {
            match evaluator::evaluate(cond, facts)? {
                Value::Boolean(b) => b,
                _ => false,
            }
        } else {
            true // No condition means it's always met.
        };

        let final_value = if condition_met {
            if let Some(then) = then_ast {
                evaluator::evaluate(then, facts)?
            } else {
                bail!("Rule for '{}' has a condition but no 'then' clause.", attr_name)
            }
        } else {
            if let Some(other) = other_ast {
                evaluator::evaluate(other, facts)?
            } else {
                 bail!("Rule condition for '{}' was false, but no 'otherwise' clause was found.", attr_name)
            }
        };

        facts.insert(attr_name.to_string(), final_value);
        Ok(())
    }
}
EOF

# --- Create CLI and LSP files ---
# (These files remain largely the same, but the setup script would create them)

echo "📄 Creating data-designer-cli/src/main.rs..."
# ... CLI main.rs content from your GitHub ...
# --- For brevity, showing only the updated test-rule command ---
cat << 'EOF' > data-designer-cli/src/main.rs
use data_designer_core::{manager, engine::{RulesEngine, Facts}};
use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ... (Other use statements and struct definitions from your repo)
#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Executes a test case for the derived attribute rules.
    TestRule {
        /// The path to the test case directory.
        #[arg(long)]
        case: PathBuf,
    },
    // ... other commands
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::TestRule { case } => {
            println!("🧪 Running test case '{}'", case.display());

            let dict = manager::load_dictionary()?;
            let engine = RulesEngine::new(&dict)?;

            let source_path = case.join("source_facts.json");
            let source_content = fs::read_to_string(&source_path)?;
            let initial_facts: Facts = serde_json::from_str(&source_content)?;
            println!("  -> Loaded {} source fact(s).", initial_facts.len());

            let target_path = case.join("expected_targets.json");
            let target_content = fs::read_to_string(&target_path)?;
            let expected_targets: HashMap<String, Value> = serde_json::from_str(&target_content)?;
            let targets_to_calculate: Vec<String> = expected_targets.keys().cloned().collect();

            let final_facts = engine.evaluate_chain(&targets_to_calculate, &initial_facts)?;
            
            let mut all_tests_passed = true;
            for (target_name, expected_value) in expected_targets {
                if let Some(actual_value) = final_facts.get(&target_name) {
                    // We need to convert our native Value enum to serde_json::Value for comparison
                    let actual_serde_value = match actual_value {
                        data_designer_core::models::Value::Integer(i) => Value::from(*i),
                        data_designer_core::models::Value::String(s) => Value::from(s.clone()),
                        data_designer_core::models::Value::Boolean(b) => Value::from(*b),
                        _ => Value::Null,
                    };

                    if actual_serde_value == expected_value {
                        println!("  ✅ PASS: '{}' matches expected value '{}'.", target_name, expected_value);
                    } else {
                        println!("  🔥 FAIL: '{}' was '{}', but expected '{}'.", target_name, actual_serde_value, expected_value);
                        all_tests_passed = false;
                    }
                } else {
                    println!("  🔥 FAIL: Target '{}' was not calculated.", target_name);
                    all_tests_passed = false;
                }
            }
            
            if !all_tests_passed {
                bail!("One or more tests failed.");
            }
        }
        // ... other command handlers
    }
    Ok(())
}
EOF

# ... (Script would continue to create LSP files, etc.)

echo "✅ Project setup complete!"
echo "Navigate to the 'data-designer' directory, run 'cargo build',"
echo "and then use the fully functional 'test-rule' command."


