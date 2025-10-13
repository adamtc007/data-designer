use std::collections::HashMap;
use std::fs;
use data_designer::{generate_test_context, BusinessRule, parser::{parse_rule, ASTNode as ParserASTNode}};
use serde::{Deserialize, Serialize};
use serde_json::{Value, Value as JsonValue};
use tauri::State;
use ts_rs::TS;

// Configuration module
mod config;
use config::Config;

// Unified database interface
mod db;
use db::{DbPool, RuleOperations};
use db::{CreateRuleWithTemplateRequest, CreateRuleRequest};
use db::CreateDerivedAttributeRequest;
use db::grammar::GrammarOperations;
use db::{CreateCbuRequest, AddCbuMemberRequest, ClientBusinessUnit, CbuSummary, CbuRole, CbuMember, CbuMemberDetail};
use db::{CreateProductRequest, CreateServiceRequest, UpdateServiceRequest, CreateResourceRequest, CreateOnboardingRequest, SubscribeCbuToProductRequest};
use db::{Product, Service, Resource, ProductHierarchyView, CbuSubscriptionView, OnboardingProgressView};

// Legacy modules (to be cleaned up)
mod database;
mod embeddings;
mod schema_visualizer;
mod data_dictionary;
use data_dictionary::DataDictionaryState;

#[derive(Serialize, Deserialize)]
struct TestRule {
    id: u32,
    name: String,
    dsl: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct TestResult {
    success: bool,
    result: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct GrammarRule {
    name: String,
    definition: String,
    #[serde(rename = "type")]
    rule_type: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct GrammarMetadata {
    version: String,
    description: String,
    created: String,
    author: String,
}

#[derive(Serialize, Deserialize)]
struct FunctionInfo {
    name: String,
    signature: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct GrammarExtensions {
    operators: HashMap<String, Vec<String>>,
    functions: Vec<FunctionInfo>,
    keywords: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SourceDataset {
    id: String,
    name: String,
    description: String,
    attributes: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct SourceData {
    datasets: Vec<SourceDataset>,
    lookup_tables: HashMap<String, HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct RuleMapping {
    rule_id: String,
    rule_name: String,
    description: String,
    source_dataset: String,
    rule_expression: String,
    target_attributes: HashMap<String, String>,
    expected_result: Value,
}

#[derive(Serialize, Deserialize)]
struct TargetData {
    rule_mappings: Vec<RuleMapping>,
    metadata: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct Grammar {
    name: String,
    rules: Vec<GrammarRule>,
}

#[derive(Serialize, Deserialize)]
struct DynamicGrammar {
    metadata: GrammarMetadata,
    grammar: Grammar,
    extensions: GrammarExtensions,
}

#[derive(Debug, Serialize, Deserialize)]
struct ASTNode {
    node_type: String,
    value: Option<String>,
    children: Vec<ASTNode>,
    metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct ASTVisualization {
    ast: ASTNode,
    dot_format: String,
    json_format: String,
    text_tree: String,
}

// Data Dictionary Structures
#[derive(Serialize, Deserialize, Clone)]
struct AttributeConstraints {
    allowed_values: Option<Vec<String>>,
    required: Option<bool>,
    max_length: Option<u32>,
    min_length: Option<u32>,
    pattern: Option<String>,
    min_value: Option<f64>,
    max_value: Option<f64>,
    decimal_places: Option<u32>,
    integer_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
struct AttributeSource {
    system: String,
    field: String,
    table: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct AttributeDefinition {
    name: String,
    display_name: String,
    data_type: String, // "String", "Number", "Boolean"
    description: String,
    constraints: AttributeConstraints,
    source: AttributeSource,
    tags: Vec<String>,
    created_date: String,
    last_modified: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct AttributeCategory {
    name: String,
    description: String,
    color: String,
}

#[derive(Serialize, Deserialize)]
struct DataDictionary {
    metadata: GrammarMetadata,
    attributes: Vec<AttributeDefinition>,
    categories: Vec<AttributeCategory>,
}

#[derive(Serialize, Deserialize, Clone)]
struct CompiledRule {
    rule_id: String,
    rule_name: String,
    generated_code: String,
    rhai_script: Option<String>,
    input_attributes: Vec<String>,
    output_attribute: String,
    compilation_timestamp: String,
}

#[tauri::command]
fn save_rules(dsl_text: String) -> Result<String, String> {
    println!("Received DSL text: {}", dsl_text);

    // Parse the rule using the new parser
    let mut rule = BusinessRule::new(
        "user_rule".to_string(),
        "User Rule".to_string(),
        "User-defined rule".to_string(),
        dsl_text.clone(),
    );

    // Validate the rule can be parsed
    rule.parse().map_err(|e| format!("Parse error: {}", e))?;

    // Save the rule text
    std::fs::write("my_rules.rules", &dsl_text).map_err(|e| e.to_string())?;

    // Save the parsed rule as JSON
    let json_output = serde_json::to_string_pretty(&rule).map_err(|e| e.to_string())?;
    std::fs::write("rules.json", &json_output).map_err(|e| e.to_string())?;

    Ok("Rules saved successfully".to_string())
}

#[tauri::command]
fn get_test_rules() -> Vec<TestRule> {
    vec![
        TestRule {
            id: 1,
            name: "Simple Math".to_string(),
            dsl: "result = 100 + 25 * 2 - 10 / 2".to_string(),
            description: "Basic arithmetic operations".to_string(),
        },
        TestRule {
            id: 2,
            name: "String Concatenation".to_string(),
            dsl: r#"message = "Hello " & name & "!""#.to_string(),
            description: "Join strings together".to_string(),
        },
        TestRule {
            id: 3,
            name: "Complex Expression".to_string(),
            dsl: "(100 + 50) * 2".to_string(),
            description: "Parentheses and precedence".to_string(),
        },
        TestRule {
            id: 4,
            name: "Function Call".to_string(),
            dsl: r#"CONCAT("User: ", name, " (", role, ")")"#.to_string(),
            description: "Using CONCAT function".to_string(),
        },
        TestRule {
            id: 5,
            name: "Substring Function".to_string(),
            dsl: "SUBSTRING(user_id, 0, 3)".to_string(),
            description: "Extract part of a string".to_string(),
        },
        TestRule {
            id: 6,
            name: "Lookup Function".to_string(),
            dsl: r#"LOOKUP(country_code, "countries")"#.to_string(),
            description: "External data lookup".to_string(),
        },
        TestRule {
            id: 7,
            name: "Complex Calculation".to_string(),
            dsl: r#"CONCAT("Rate: ", (base_rate + LOOKUP(tier, "rates")) * 100, "%")"#.to_string(),
            description: "Mixed operations and functions".to_string(),
        },
        TestRule {
            id: 8,
            name: "Runtime Variables".to_string(),
            dsl: "price * quantity + tax".to_string(),
            description: "Using context variables".to_string(),
        },
    ]
}

#[tauri::command]
fn test_rule(dsl_text: String) -> TestResult {
    // Create a test context
    let context = generate_test_context();

    // Parse and evaluate the rule
    let mut rule = BusinessRule::new(
        "test".to_string(),
        "Test Rule".to_string(),
        "Testing user input".to_string(),
        dsl_text,
    );

    match rule.parse() {
        Ok(_) => {
            match rule.evaluate(&context) {
                Ok(result) => {
                    TestResult {
                        success: true,
                        result: Some(serde_json::to_string(&result).unwrap_or_else(|_| "Unknown".to_string())),
                        error: None,
                    }
                }
                Err(e) => {
                    TestResult {
                        success: false,
                        result: None,
                        error: Some(format!("Evaluation error: {}", e)),
                    }
                }
            }
        }
        Err(e) => {
            TestResult {
                success: false,
                result: None,
                error: Some(format!("Parse error: {}", e)),
            }
        }
    }
}

#[tauri::command]
fn get_grammar_rules() -> Result<Vec<GrammarRule>, String> {
    // Return a simplified grammar description for the UI
    Ok(vec![
        GrammarRule {
            name: "expression".to_string(),
            definition: "assignment | arithmetic | function_call | literal".to_string(),
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
            name: "arithmetic".to_string(),
            definition: "expression ('+' | '-' | '*' | '/' | '%') expression".to_string(),
            rule_type: "normal".to_string(),
            description: "Arithmetic operations".to_string(),
        },
        GrammarRule {
            name: "string_concat".to_string(),
            definition: "expression '&' expression".to_string(),
            rule_type: "normal".to_string(),
            description: "String concatenation".to_string(),
        },
        GrammarRule {
            name: "comparison".to_string(),
            definition: "expression ('==' | '!=' | '<' | '>' | '<=' | '>=') expression".to_string(),
            rule_type: "normal".to_string(),
            description: "Comparison operations".to_string(),
        },
        GrammarRule {
            name: "logical".to_string(),
            definition: "expression ('and' | 'or' | 'not') expression".to_string(),
            rule_type: "normal".to_string(),
            description: "Logical operations".to_string(),
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
        GrammarRule {
            name: "number".to_string(),
            definition: "'-'? [0-9]+ ('.' [0-9]+)?".to_string(),
            rule_type: "atomic".to_string(),
            description: "Numeric literals".to_string(),
        },
        GrammarRule {
            name: "string".to_string(),
            definition: r#"'"' (~["\] | '\' .)* '"' | "'" (~['\] | '\' .)* "'""#.to_string(),
            rule_type: "atomic".to_string(),
            description: "String literals".to_string(),
        },
        GrammarRule {
            name: "boolean".to_string(),
            definition: "'true' | 'false'".to_string(),
            rule_type: "atomic".to_string(),
            description: "Boolean literals".to_string(),
        },
        GrammarRule {
            name: "identifier".to_string(),
            definition: "[a-zA-Z_][a-zA-Z0-9_]*".to_string(),
            rule_type: "atomic".to_string(),
            description: "Variable and function names".to_string(),
        },
    ])
}

#[tauri::command]
fn update_grammar_rule(name: String, _rule: GrammarRule) -> Result<String, String> {
    // In a real implementation, this would update the grammar
    // For now, just acknowledge the update
    Ok(format!("Grammar rule '{}' updated", name))
}

#[tauri::command]
fn generate_grammar_visualization() -> Result<String, String> {
    // Generate a representation of the nom parser grammar
    Ok(r#"// nom Parser Grammar (Conceptual Representation)

expression = { assignment | logical_or }

assignment = { identifier ~ "=" ~ expression }

logical_or = { logical_and ~ ("or" | "||") ~ logical_and }*

logical_and = { comparison ~ ("and" | "&&") ~ comparison }*

comparison = { concatenation ~ (comparison_op ~ concatenation)? }
comparison_op = { "<=" | ">=" | "!=" | "<>" | "==" | "=" | "<" | ">" }

concatenation = { arithmetic ~ ("&" ~ arithmetic)* }

arithmetic = { term ~ (("+"|"-") ~ term)* }

term = { unary ~ (("*"|"/"|"%") ~ unary)* }

unary = { ("not"|"!"|"-")? ~ power }

power = { primary ~ ("**" ~ primary)* }

primary = {
    number |
    string_literal |
    boolean |
    function_call |
    list |
    identifier |
    "(" ~ expression ~ ")"
}

function_call = { identifier ~ "(" ~ argument_list? ~ ")" }

argument_list = { expression ~ ("," ~ expression)* }

list = { "[" ~ (expression ~ ("," ~ expression)*)? ~ "]" }

number = @{ "-"? ~ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }

string_literal = @{
    "\"" ~ (!"\"" ~ ("\\" ~ ANY | ANY))* ~ "\"" |
    "'" ~ (!"'" ~ ("\\" ~ ANY | ANY))* ~ "'"
}

boolean = { "true" | "false" }

identifier = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }

WHITESPACE = _{ " " | "\t" | "\r" | "\n" }"#.to_string())
}

#[tauri::command]
fn load_grammar() -> Result<DynamicGrammar, String> {
    let grammar = DynamicGrammar {
        metadata: GrammarMetadata {
            version: "2.0.0".to_string(),
            description: "Dynamic DSL Grammar using nom parser".to_string(),
            created: "2024-01-01".to_string(),
            author: "System".to_string(),
        },
        grammar: Grammar {
            name: "DSL Grammar".to_string(),
            rules: get_grammar_rules()?,
        },
        extensions: GrammarExtensions {
            operators: {
                let mut ops = HashMap::new();
                ops.insert("Arithmetic".to_string(), vec!["+".to_string(), "-".to_string(), "*".to_string(), "/".to_string(), "%".to_string(), "**".to_string()]);
                ops.insert("String".to_string(), vec!["&".to_string()]);
                ops.insert("Comparison".to_string(), vec!["==".to_string(), "!=".to_string(), "<".to_string(), ">".to_string(), "<=".to_string(), ">=".to_string()]);
                ops.insert("Logical".to_string(), vec!["and".to_string(), "or".to_string(), "not".to_string()]);
                ops
            },
            functions: vec![
                FunctionInfo {
                    name: "CONCAT".to_string(),
                    signature: "CONCAT(...args)".to_string(),
                    description: "Concatenate multiple values".to_string(),
                },
                FunctionInfo {
                    name: "SUBSTRING".to_string(),
                    signature: "SUBSTRING(string, start, end)".to_string(),
                    description: "Extract substring".to_string(),
                },
                FunctionInfo {
                    name: "LOOKUP".to_string(),
                    signature: "LOOKUP(key, table)".to_string(),
                    description: "Lookup value from external table".to_string(),
                },
                FunctionInfo {
                    name: "UPPER".to_string(),
                    signature: "UPPER(string)".to_string(),
                    description: "Convert to uppercase".to_string(),
                },
                FunctionInfo {
                    name: "LOWER".to_string(),
                    signature: "LOWER(string)".to_string(),
                    description: "Convert to lowercase".to_string(),
                },
                FunctionInfo {
                    name: "LEN".to_string(),
                    signature: "LEN(string)".to_string(),
                    description: "Get string length".to_string(),
                },
            ],
            keywords: vec!["true".to_string(), "false".to_string(), "and".to_string(), "or".to_string(), "not".to_string()],
        },
    };

    Ok(grammar)
}

#[tauri::command]
fn save_grammar(grammar: DynamicGrammar) -> Result<String, String> {
    let json = serde_json::to_string_pretty(&grammar).map_err(|e| e.to_string())?;
    fs::write("grammar_rules.json", json).map_err(|e| e.to_string())?;
    Ok("Grammar saved successfully".to_string())
}

fn parser_ast_to_viz_node(node: &ParserASTNode) -> ASTNode {
    use ParserASTNode::*;

    match node {
        Number(n) => {
            let mut metadata = HashMap::new();
            metadata.insert("data_type".to_string(), "Number".to_string());

            ASTNode {
                node_type: "Number".to_string(),
                value: Some(n.to_string()),
                children: vec![],
                metadata,
            }
        },
        String(s) => {
            let mut metadata = HashMap::new();
            metadata.insert("data_type".to_string(), "String".to_string());

            ASTNode {
                node_type: "String".to_string(),
                value: Some(format!("\"{}\"", s)),
                children: vec![],
                metadata,
            }
        },
        Boolean(b) => {
            let mut metadata = HashMap::new();
            metadata.insert("data_type".to_string(), "Boolean".to_string());

            ASTNode {
                node_type: "Boolean".to_string(),
                value: Some(b.to_string()),
                children: vec![],
                metadata,
            }
        },
        Regex(r) => {
            let mut metadata = HashMap::new();
            metadata.insert("data_type".to_string(), "Regex".to_string());

            ASTNode {
                node_type: "Regex".to_string(),
                value: Some(format!("/{}/", r)),
                children: vec![],
                metadata,
            }
        },
        Identifier(name) => {
            let mut metadata = HashMap::new();
            metadata.insert("identifier_type".to_string(), "Variable".to_string());

            ASTNode {
                node_type: "Identifier".to_string(),
                value: Some(name.clone()),
                children: vec![],
                metadata,
            }
        },
        Assignment { target, value } => {
            let mut metadata = HashMap::new();
            metadata.insert("target".to_string(), target.clone());

            ASTNode {
                node_type: "Assignment".to_string(),
                value: Some(format!("{} =", target)),
                children: vec![parser_ast_to_viz_node(value)],
                metadata,
            }
        },
        BinaryOp { op, left, right } => {
            let mut metadata = HashMap::new();
            let op_str = format!("{:?}", op);
            metadata.insert("operator".to_string(), op_str.clone());

            ASTNode {
                node_type: "BinaryOp".to_string(),
                value: Some(op_str),
                children: vec![
                    parser_ast_to_viz_node(left),
                    parser_ast_to_viz_node(right),
                ],
                metadata,
            }
        },
        UnaryOp { op, operand } => {
            let mut metadata = HashMap::new();
            let op_str = format!("{:?}", op);
            metadata.insert("operator".to_string(), op_str.clone());

            ASTNode {
                node_type: "UnaryOp".to_string(),
                value: Some(op_str),
                children: vec![parser_ast_to_viz_node(operand)],
                metadata,
            }
        },
        FunctionCall { name, args } => {
            let mut metadata = HashMap::new();
            metadata.insert("function_name".to_string(), name.clone());
            metadata.insert("arg_count".to_string(), args.len().to_string());

            ASTNode {
                node_type: "FunctionCall".to_string(),
                value: Some(format!("{}()", name)),
                children: args.iter().map(parser_ast_to_viz_node).collect(),
                metadata,
            }
        },
        List(items) => {
            let mut metadata = HashMap::new();
            metadata.insert("item_count".to_string(), items.len().to_string());

            ASTNode {
                node_type: "List".to_string(),
                value: Some(format!("[{} items]", items.len())),
                children: items.iter().map(parser_ast_to_viz_node).collect(),
                metadata,
            }
        },
    }
}

fn generate_dot_format(node: &ASTNode, node_id: &mut usize) -> String {
    let current_id = *node_id;
    *node_id += 1;

    let label = if let Some(ref val) = node.value {
        format!("{}\\n{}", node.node_type, val)
    } else {
        node.node_type.clone()
    };

    let mut dot = format!("  node{} [label=\"{}\"];\n", current_id, label);

    for child in &node.children {
        let child_id = *node_id;
        dot += &generate_dot_format(child, node_id);
        dot += &format!("  node{} -> node{};\n", current_id, child_id);
    }

    dot
}

fn generate_text_tree(node: &ASTNode, prefix: &str, is_last: bool) -> String {
    let mut result = String::new();

    let connector = if is_last { "└─ " } else { "├─ " };
    let value_str = if let Some(ref val) = node.value {
        format!("{}: {}", node.node_type, val)
    } else {
        node.node_type.clone()
    };

    result += &format!("{}{}{}\n", prefix, connector, value_str);

    let child_prefix = if is_last {
        format!("{}   ", prefix)
    } else {
        format!("{}│  ", prefix)
    };

    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;
        result += &generate_text_tree(child, &child_prefix, is_last_child);
    }

    result
}

#[tauri::command]
fn visualize_ast(dsl_text: String) -> Result<ASTVisualization, String> {
    println!("visualize_ast called with: {}", dsl_text);

    // Parse the DSL expression
    let parser_ast = match parse_rule(&dsl_text) {
        Ok((_, ast)) => {
            println!("Parse successful: {:?}", ast);
            ast
        },
        Err(e) => {
            let error_msg = format!("Parse error: {:?}", e);
            println!("{}", error_msg);
            return Err(error_msg);
        }
    };

    // Convert to AST node representation
    let ast = parser_ast_to_viz_node(&parser_ast);
    println!("Converted AST: {:?}", ast);

    // Generate DOT format for GraphViz
    let mut node_id = 0;
    let dot_content = generate_dot_format(&ast, &mut node_id);
    let dot_format = format!("digraph AST {{\n  rankdir=TB;\n  node [shape=box];\n{}}}", dot_content);

    // Generate JSON format
    let json_format = serde_json::to_string_pretty(&ast)
        .map_err(|e| e.to_string())?;

    // Generate text tree
    let text_tree = generate_text_tree(&ast, "", true);

    println!("Generated text_tree length: {}", text_tree.len());
    println!("Text tree:\n{}", text_tree);

    Ok(ASTVisualization {
        ast,
        dot_format,
        json_format,
        text_tree,
    })
}

#[tauri::command]
fn get_api_keys() -> Result<HashMap<String, String>, String> {
    use std::env;

    let mut keys = HashMap::new();

    // Check for Anthropic/Claude API key
    if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
        keys.insert("anthropic".to_string(), key);
    }

    // Check for OpenAI API key
    if let Ok(key) = env::var("OPENAI_API_KEY") {
        keys.insert("openai".to_string(), key);
    }

    // Check for alternate Claude key
    if let Ok(key) = env::var("CLAUDE_API_KEY") {
        keys.insert("claude".to_string(), key);
    }

    if keys.is_empty() {
        Err("No API keys found in environment variables".to_string())
    } else {
        Ok(keys)
    }
}

#[tauri::command]
fn load_source_data() -> Result<SourceData, String> {
    let data_path = "../test_data/source_attributes.json";
    let content = fs::read_to_string(data_path)
        .map_err(|e| format!("Failed to load source data: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse source data: {}", e))
}

#[tauri::command]
fn load_target_rules() -> Result<TargetData, String> {
    let data_path = "../test_data/target_attributes.json";
    let content = fs::read_to_string(data_path)
        .map_err(|e| format!("Failed to load target rules: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse target rules: {}", e))
}

#[tauri::command]
fn test_rule_with_dataset(rule_expression: String, dataset_id: String) -> TestResult {
    // Load source data
    let source_data = match load_source_data() {
        Ok(data) => data,
        Err(e) => return TestResult {
            success: false,
            result: None,
            error: Some(e),
        }
    };

    // Find the dataset
    let dataset = source_data.datasets.iter()
        .find(|d| d.id == dataset_id);

    if dataset.is_none() {
        return TestResult {
            success: false,
            result: None,
            error: Some(format!("Dataset '{}' not found", dataset_id)),
        };
    }

    // Convert attributes to context
    let dataset = dataset.unwrap();
    let mut context: HashMap<String, JsonValue> = HashMap::new();

    for (key, value) in &dataset.attributes {
        context.insert(key.clone(), value.clone());
    }

    // Add lookup tables as JSON values
    if let Some(countries) = source_data.lookup_tables.get("countries") {
        context.insert("__lookup_countries".to_string(), serde_json::to_value(countries).unwrap_or(JsonValue::Null));
    }
    if let Some(rates) = source_data.lookup_tables.get("rates") {
        context.insert("__lookup_rates".to_string(), serde_json::to_value(rates).unwrap_or(JsonValue::Null));
    }

    // Parse and evaluate the rule
    let mut rule = BusinessRule::new(
        "test_rule".to_string(),
        "Test Rule".to_string(),
        "Testing with dataset".to_string(),
        rule_expression
    );
    match rule.parse() {
        Ok(_) => {
            match rule.evaluate(&context) {
                Ok(result) => {
                    TestResult {
                        success: true,
                        result: Some(serde_json::to_string(&result).unwrap_or_else(|_| "Unknown".to_string())),
                        error: None,
                    }
                }
                Err(e) => {
                    TestResult {
                        success: false,
                        result: None,
                        error: Some(format!("Evaluation error: {}", e)),
                    }
                }
            }
        }
        Err(e) => {
            TestResult {
                success: false,
                result: None,
                error: Some(format!("Parse error: {}", e)),
            }
        }
    }
}

// Database commands
#[tauri::command]
async fn db_get_all_rules(pool: State<'_, DbPool>) -> Result<Vec<database::Rule>, String> {
    database::get_all_rules(&pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_get_rule(pool: State<'_, DbPool>, rule_id: String) -> Result<Option<database::Rule>, String> {
    database::get_rule_by_id(&pool, &rule_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_create_rule(pool: State<'_, DbPool>, request: CreateRuleRequest) -> Result<database::Rule, String> {
    database::create_rule(&pool, request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_update_rule(pool: State<'_, DbPool>, rule_id: String, rule_definition: String) -> Result<database::Rule, String> {
    database::update_rule(&pool, &rule_id, &rule_definition)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_search_rules(pool: State<'_, DbPool>, query: String) -> Result<Vec<database::Rule>, String> {
    database::search_rules(&pool, &query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_get_business_attributes(pool: State<'_, DbPool>) -> Result<Vec<database::BusinessAttribute>, String> {
    database::get_all_business_attributes(&pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_get_derived_attributes(pool: State<'_, DbPool>) -> Result<Vec<database::DerivedAttribute>, String> {
    database::get_all_derived_attributes(&pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_get_categories(pool: State<'_, DbPool>) -> Result<Vec<database::RuleCategory>, String> {
    database::get_all_categories(&pool)
        .await
        .map_err(|e| e.to_string())
}

// Embedding commands
#[tauri::command]
async fn db_find_similar_rules(
    pool: State<'_, DbPool>,
    dsl_text: String,
    limit: i32
) -> Result<Vec<db::SimilarRule>, String> {
    embeddings::find_similar_rules(&pool, &dsl_text, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_update_rule_embedding(
    pool: State<'_, DbPool>,
    rule_id: String,
    dsl_text: String
) -> Result<(), String> {
    embeddings::update_rule_embedding(&pool, &rule_id, &dsl_text)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_generate_all_embeddings(pool: State<'_, DbPool>) -> Result<(), String> {
    embeddings::generate_all_embeddings(&pool)
        .await
        .map_err(|e| e.to_string())
}

// Data Dictionary commands
#[tauri::command]
async fn dd_get_data_dictionary(
    pool: State<'_, DbPool>,
    entity_filter: Option<String>
) -> Result<data_dictionary::DataDictionaryResponse, String> {
    data_dictionary::get_data_dictionary(&pool, entity_filter)
        .await
}

#[tauri::command]
async fn dd_refresh_data_dictionary(pool: State<'_, DbPool>) -> Result<(), String> {
    data_dictionary::refresh_data_dictionary(&pool)
        .await
}

#[tauri::command]
async fn dd_create_derived_attribute(
    pool: State<'_, DbPool>,
    request: CreateDerivedAttributeRequest
) -> Result<i32, String> {
    data_dictionary::create_derived_attribute(&pool, request)
        .await
}

#[tauri::command]
async fn dd_create_and_compile_rule(
    pool: State<'_, DbPool>,
    rule_name: String,
    dsl_code: String,
    target_attribute_id: i32,
    dependencies: Vec<String>
) -> Result<data_dictionary::CompiledRule, String> {
    data_dictionary::create_and_compile_rule(&pool, rule_name, dsl_code, target_attribute_id, dependencies)
        .await
}

#[tauri::command]
async fn dd_search_attributes(
    pool: State<'_, DbPool>,
    search_term: String
) -> Result<Vec<data_dictionary::AttributeDefinition>, String> {
    data_dictionary::search_attributes(&pool, search_term)
        .await
}

// Schema visualization commands
#[tauri::command]
async fn db_get_schema_info(pool: State<'_, DbPool>) -> Result<schema_visualizer::SchemaInfo, String> {
    schema_visualizer::get_schema_info(&pool)
        .await
}

#[tauri::command]
async fn open_schema_viewer(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;

    // Check if window already exists
    if let Some(window) = app.get_webview_window("schema-viewer") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    } else {
        // Create new window for schema viewer in Tauri v2
        // Using the correct API for Tauri v2
        tauri::WebviewWindowBuilder::new(
            &app,
            "schema-viewer",
            tauri::WebviewUrl::App("schema.html".into())
        )
        .title("Database Schema Visualizer")
        .inner_size(1200.0, 800.0)
        .build()
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn db_get_table_relationships(
    pool: State<'_, DbPool>,
    table_name: String,
) -> Result<Vec<schema_visualizer::RelationshipInfo>, String> {
    schema_visualizer::get_table_relationships(&pool)
        .await
}

#[tauri::command]
async fn db_execute_sql(
    pool: State<'_, DbPool>,
    query: String,
) -> Result<schema_visualizer::SqlQueryResult, String> {
    schema_visualizer::execute_sql_query(&pool, &query)
        .await
}

// ====================
// NEW RULE WORKFLOW COMMANDS (FIXED POOL DEREFERENCING ISSUES)
// ====================

#[tauri::command]
async fn check_attribute_exists(
    pool: State<'_, DbPool>,
    attribute_name: String,
) -> Result<bool, String> {
    RuleOperations::check_attribute_exists(&*pool, &attribute_name).await
}

#[tauri::command]
async fn get_business_attributes(
    pool: State<'_, DbPool>,
) -> Result<Vec<serde_json::Value>, String> {
    RuleOperations::get_business_attributes(&*pool).await
}

// Removed: Now using unified database interface

#[tauri::command]
async fn create_rule_with_template(
    pool: State<'_, DbPool>,
    request: CreateRuleWithTemplateRequest,
) -> Result<(), String> {
    RuleOperations::create_rule_with_template(&*pool, request).await
}

#[tauri::command]
async fn get_existing_rules(
    pool: State<'_, DbPool>,
) -> Result<Vec<serde_json::Value>, String> {
    RuleOperations::get_existing_rules(&*pool).await
}

#[tauri::command]
async fn get_rule_by_id(
    pool: State<'_, DbPool>,
    rule_id: String,
) -> Result<serde_json::Value, String> {
    RuleOperations::get_rule_by_id(&*pool, &rule_id).await
}

#[derive(Serialize, Deserialize)]
struct GrammarInfo {
    keywords: Vec<String>,
    functions: Vec<FunctionInfo>,
    operators: Vec<String>,
    kyc_attributes: Vec<String>,
}

#[tauri::command]
async fn get_grammar_info(pool: State<'_, DbPool>) -> Result<GrammarInfo, String> {
    println!("🔥 get_grammar_info command called!");

    // Get compact grammar info from database
    println!("🔥 About to call GrammarOperations::get_compact_grammar_info...");
    let compact_info = match GrammarOperations::get_compact_grammar_info(&pool).await {
        Ok(info) => {
            println!("🔥 Grammar loading succeeded!");
            info
        },
        Err(e) => {
            println!("🔥 Grammar loading failed: {}", e);
            return Err(e);
        }
    };

    // Convert from CompactGrammarInfo to GrammarInfo
    let functions = compact_info.functions.into_iter().map(|f| FunctionInfo {
        name: f.name,
        signature: f.signature,
        description: f.description,
    }).collect();

    Ok(GrammarInfo {
        keywords: compact_info.keywords,
        functions,
        operators: compact_info.operators,
        kyc_attributes: compact_info.kyc_attributes,
    })
}

#[derive(Serialize, Deserialize)]
struct DatabaseStatus {
    connected: bool,
    error: Option<String>,
    version: Option<String>,
}

#[tauri::command]
async fn check_database_connection(pool: State<'_, DbPool>) -> Result<DatabaseStatus, String> {
    // Use a query that returns a simple value we can handle
    match sqlx::query_scalar::<_, String>("SELECT 'connected'").fetch_one(pool.inner()).await {
        Ok(_) => {
            // If we can execute a query, we're connected
            Ok(DatabaseStatus {
                connected: true,
                error: None,
                version: Some("PostgreSQL Connected".to_string()),
            })
        }
        Err(e) => {
            Ok(DatabaseStatus {
                connected: false,
                error: Some(format!("Database connection failed: {}", e)),
                version: None,
            })
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ConfigInfo {
    database_host: String,
    database_port: u16,
    database_name: String,
    database_username: String,
    has_password: bool,
    max_connections: u32,
    config_source: String,
}

// === CBU Management Commands ===

#[tauri::command]
async fn create_cbu(request: CreateCbuRequest) -> Result<ClientBusinessUnit, String> {
    db::DbOperations::create_cbu(request).await
}

#[tauri::command]
async fn get_cbu_by_id(cbu_id: String) -> Result<Option<ClientBusinessUnit>, String> {
    db::DbOperations::get_cbu_by_id(&cbu_id).await
}

#[tauri::command]
async fn list_cbus() -> Result<Vec<CbuSummary>, String> {
    db::DbOperations::list_cbus().await
}

#[tauri::command]
async fn get_cbu_roles() -> Result<Vec<CbuRole>, String> {
    db::DbOperations::get_cbu_roles().await
}

#[tauri::command]
async fn add_cbu_member(request: AddCbuMemberRequest) -> Result<CbuMember, String> {
    db::DbOperations::add_cbu_member(request).await
}

#[tauri::command]
async fn get_cbu_members(cbu_id: String) -> Result<Vec<CbuMemberDetail>, String> {
    db::DbOperations::get_cbu_members(&cbu_id).await
}

#[tauri::command]
async fn remove_cbu_member(
    cbu_id: String,
    entity_id: String,
    role_code: String,
    updated_by: Option<String>
) -> Result<(), String> {
    db::DbOperations::remove_cbu_member(&cbu_id, &entity_id, &role_code, updated_by).await
}

#[tauri::command]
async fn search_cbus(search_term: String) -> Result<Vec<CbuSummary>, String> {
    db::DbOperations::search_cbus(&search_term).await
}

#[tauri::command]
async fn get_cbu_roles_by_category() -> Result<std::collections::HashMap<String, Vec<CbuRole>>, String> {
    db::DbOperations::get_cbu_roles_by_category().await
}

#[tauri::command]
async fn update_cbu(
    cbu_id: String,
    cbu_name: Option<String>,
    description: Option<String>,
    business_type: Option<String>,
    updated_by: Option<String>
) -> Result<ClientBusinessUnit, String> {
    db::DbOperations::update_cbu(&cbu_id, cbu_name, description, business_type, updated_by).await
}

#[tauri::command]
async fn get_config_info() -> Result<ConfigInfo, String> {
    let config = Config::load().map_err(|e| format!("Failed to load config: {}", e))?;

    Ok(ConfigInfo {
        database_host: config.database.host,
        database_port: config.database.port,
        database_name: config.database.database,
        database_username: config.database.username,
        has_password: config.database.password.is_some(),
        max_connections: config.database.max_connections,
        config_source: if std::path::Path::new("config.toml").exists() {
            "config.toml + environment variables".to_string()
        } else {
            "environment variables + defaults".to_string()
        },
    })
}

// ===== PRODUCT MANAGEMENT COMMANDS =====

#[tauri::command]
async fn create_product(request: CreateProductRequest) -> Result<Product, String> {
    db::DbOperations::create_product(request).await
}

#[tauri::command]
async fn list_products(line_of_business: Option<String>) -> Result<Vec<Product>, String> {
    db::DbOperations::list_products(line_of_business).await
}

#[tauri::command]
async fn get_product_hierarchy(product_id: Option<String>) -> Result<Vec<ProductHierarchyView>, String> {
    db::DbOperations::get_product_hierarchy(product_id).await
}

#[tauri::command]
async fn create_service(request: CreateServiceRequest) -> Result<Service, String> {
    db::DbOperations::create_service(request).await
}

#[tauri::command]
async fn list_services(category: Option<String>) -> Result<Vec<Service>, String> {
    db::DbOperations::list_services(category).await
}

#[tauri::command]
async fn get_service_by_id(service_id: i32) -> Result<Option<Service>, String> {
    db::DbOperations::get_service_by_id(service_id).await
}

#[tauri::command]
async fn update_service(service_id: i32, request: UpdateServiceRequest) -> Result<Service, String> {
    db::DbOperations::update_service(service_id, request).await
}

#[tauri::command]
async fn create_resource(request: CreateResourceRequest) -> Result<Resource, String> {
    db::DbOperations::create_resource(request).await
}

#[tauri::command]
async fn list_resources(resource_type: Option<String>) -> Result<Vec<Resource>, String> {
    db::DbOperations::list_resources(resource_type).await
}

#[tauri::command]
async fn subscribe_cbu_to_product(request: SubscribeCbuToProductRequest) -> Result<db::CbuProductSubscription, String> {
    db::DbOperations::subscribe_cbu_to_product(request).await
}

#[tauri::command]
async fn get_cbu_subscriptions(cbu_id: Option<String>) -> Result<Vec<CbuSubscriptionView>, String> {
    db::DbOperations::get_cbu_subscriptions(cbu_id).await
}

#[tauri::command]
async fn create_onboarding_request(request: CreateOnboardingRequest) -> Result<db::OnboardingRequest, String> {
    db::DbOperations::create_onboarding_request(request).await
}

#[tauri::command]
async fn get_onboarding_progress(
    cbu_id: Option<String>,
    product_id: Option<String>
) -> Result<Vec<OnboardingProgressView>, String> {
    db::DbOperations::get_onboarding_progress(cbu_id, product_id).await
}

#[tauri::command]
async fn get_lines_of_business() -> Result<Vec<String>, String> {
    db::DbOperations::get_lines_of_business().await
}

#[tauri::command]
async fn get_service_categories() -> Result<Vec<String>, String> {
    db::DbOperations::get_service_categories().await
}

#[tauri::command]
async fn get_resource_types() -> Result<Vec<String>, String> {
    db::DbOperations::get_resource_types().await
}

// DSL to Rules Transpilation
#[tauri::command]
async fn transpile_dsl_to_rules(
    pool: State<'_, DbPool>,
    rule_id: String
) -> Result<TranspiledRule, String> {
    // Fetch the rule from the database
    let rule = database::get_rule_by_id(&pool, &rule_id)
        .await
        .map_err(|e| format!("Failed to fetch rule: {}", e))?;

    if rule.is_none() {
        return Err(format!("Rule with id '{}' not found", rule_id));
    }

    let rule = rule.unwrap();

    // Parse the DSL using nom parser
    let ast = match parse_rule(&rule.rule_definition) {
        Ok((_, parsed_ast)) => parsed_ast,
        Err(e) => return Err(format!("Failed to parse DSL: {:?}", e)),
    };

    // Convert AST to transpiled rule format
    let transpiled = TranspiledRule {
        rule_id: rule.rule_id.clone(),
        rule_name: rule.rule_name.clone(),
        description: rule.description.clone(),
        original_dsl: rule.rule_definition.clone(),
        parsed_ast: serde_json::to_value(&ast).map_err(|e| format!("AST serialization error: {}", e))?,
        compiled_code: generate_rust_code_from_ast(&ast)?,
        target_attribute: rule.target_attribute_id.map(|id| id.to_string()),
        dependencies: extract_dependencies_from_ast(&ast),
        compilation_timestamp: chrono::Utc::now(),
    };

    // Update the database with the parsed AST
    if let Err(e) = update_rule_ast(&pool, &rule_id, &transpiled.parsed_ast).await {
        eprintln!("Warning: Failed to update rule AST in database: {}", e);
    }

    Ok(transpiled)
}

#[derive(Serialize, Deserialize)]
struct TranspiledRule {
    rule_id: String,
    rule_name: String,
    description: Option<String>,
    original_dsl: String,
    parsed_ast: serde_json::Value,
    compiled_code: String,
    target_attribute: Option<String>,
    dependencies: Vec<String>,
    compilation_timestamp: chrono::DateTime<chrono::Utc>,
}

// Helper function to generate Rust code from AST
fn generate_rust_code_from_ast(ast: &data_designer::parser::ASTNode) -> Result<String, String> {
    use data_designer::parser::ASTNode::*;

    match ast {
        Assignment { target, value } => {
            let value_code = generate_rust_code_from_ast(value)?;
            Ok(format!("let {} = {};", target, value_code))
        },
        BinaryOp { left, op, right } => {
            let left_code = generate_rust_code_from_ast(left)?;
            let right_code = generate_rust_code_from_ast(right)?;
            let op_str = match op {
                data_designer::parser::BinaryOperator::Add => "+",
                data_designer::parser::BinaryOperator::Subtract => "-",
                data_designer::parser::BinaryOperator::Multiply => "*",
                data_designer::parser::BinaryOperator::Divide => "/",
                data_designer::parser::BinaryOperator::Modulo => "%",
                data_designer::parser::BinaryOperator::Power => ".powf",
                data_designer::parser::BinaryOperator::Concat => "concat",
                data_designer::parser::BinaryOperator::Equal => "==",
                data_designer::parser::BinaryOperator::NotEqual => "!=",
                data_designer::parser::BinaryOperator::LessThan => "<",
                data_designer::parser::BinaryOperator::GreaterThan => ">",
                data_designer::parser::BinaryOperator::LessThanOrEqual => "<=",
                data_designer::parser::BinaryOperator::GreaterThanOrEqual => ">=",
                data_designer::parser::BinaryOperator::And => "&&",
                data_designer::parser::BinaryOperator::Or => "||",
                data_designer::parser::BinaryOperator::Matches => "matches_regex",
            };

            match op {
                data_designer::parser::BinaryOperator::Power => {
                    Ok(format!("({}).powf({})", left_code, right_code))
                },
                data_designer::parser::BinaryOperator::Concat => {
                    Ok(format!("format!(\"{{}}{{}}\", {}, {})", left_code, right_code))
                },
                data_designer::parser::BinaryOperator::Matches => {
                    Ok(format!("matches_regex(&{}, &{})", left_code, right_code))
                },
                _ => {
                    Ok(format!("({} {} {})", left_code, op_str, right_code))
                }
            }
        },
        UnaryOp { op, operand } => {
            let operand_code = generate_rust_code_from_ast(operand)?;
            let op_str = match op {
                data_designer::parser::UnaryOperator::Not => "!",
                data_designer::parser::UnaryOperator::Negate => "-",
            };
            Ok(format!("({}{})", op_str, operand_code))
        },
        FunctionCall { name, args } => {
            let arg_codes: Result<Vec<std::string::String>, std::string::String> = args.iter()
                .map(generate_rust_code_from_ast)
                .collect();
            let arg_codes = arg_codes?;

            match name.as_str() {
                "CONCAT" => {
                    Ok(format!("format!(\"{}\", {})",
                        "{}".repeat(args.len()),
                        arg_codes.join(", ")))
                },
                "SUBSTRING" => {
                    if args.len() == 3 {
                        Ok(format!("substring(&{}, {}, {})", arg_codes[0], arg_codes[1], arg_codes[2]))
                    } else {
                        Err("SUBSTRING requires exactly 3 arguments".to_string())
                    }
                },
                "LOOKUP" => {
                    if args.len() == 2 {
                        Ok(format!("lookup(&{}, &{})", arg_codes[0], arg_codes[1]))
                    } else {
                        Err("LOOKUP requires exactly 2 arguments".to_string())
                    }
                },
                "UPPER" => Ok(format!("({}).to_uppercase()", arg_codes[0])),
                "LOWER" => Ok(format!("({}).to_lowercase()", arg_codes[0])),
                "LEN" => Ok(format!("({}).len()", arg_codes[0])),
                _ => Ok(format!("{}({})", name.to_lowercase(), arg_codes.join(", "))),
            }
        },
        Identifier(name) => {
            Ok(format!("context.get(\"{}\")", name))
        },
        Number(n) => Ok(n.to_string()),
        String(s) => Ok(format!("\"{}\"", s.replace("\"", "\\\""))),
        Boolean(b) => Ok(b.to_string()),
        List(items) => {
            let item_codes: Result<Vec<std::string::String>, std::string::String> = items.iter()
                .map(generate_rust_code_from_ast)
                .collect();
            let item_codes = item_codes?;
            Ok(format!("vec![{}]", item_codes.join(", ")))
        },
        Regex(pattern) => {
            Ok(format!("Regex::new(r\"{}\")", pattern.replace("\"", "\\\"")))
        },
    }
}

// Helper function to extract dependencies from AST
fn extract_dependencies_from_ast(ast: &data_designer::parser::ASTNode) -> Vec<String> {
    use data_designer::parser::ASTNode::*;
    let mut deps = Vec::new();

    match ast {
        Assignment { value, .. } => {
            deps.extend(extract_dependencies_from_ast(value));
        },
        BinaryOp { left, right, .. } => {
            deps.extend(extract_dependencies_from_ast(left));
            deps.extend(extract_dependencies_from_ast(right));
        },
        UnaryOp { operand, .. } => {
            deps.extend(extract_dependencies_from_ast(operand));
        },
        FunctionCall { args, .. } => {
            for arg in args {
                deps.extend(extract_dependencies_from_ast(arg));
            }
        },
        Identifier(name) => {
            deps.push(name.clone());
        },
        List(items) => {
            for item in items {
                deps.extend(extract_dependencies_from_ast(item));
            }
        },
        _ => {} // Literals don't have dependencies
    }

    // Remove duplicates and sort
    deps.sort();
    deps.dedup();
    deps
}

// Helper function to update rule AST in database
async fn update_rule_ast(
    pool: &DbPool,
    rule_id: &str,
    ast: &serde_json::Value
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE rules SET parsed_ast = $1, updated_at = NOW() WHERE rule_id = $2"
    )
    .bind(ast)
    .bind(rule_id)
    .execute(pool)
    .await?;

    Ok(())
}

// TypeScript generation command
#[tauri::command]
async fn generate_typescript_types() -> Result<String, String> {
    use std::fs;

    // Generate TypeScript types from Rust structs
    let mut ts_content = String::new();

    // Add header
    ts_content.push_str("// Auto-generated TypeScript types from Rust structs\n");
    ts_content.push_str("// Generated on: ");
    ts_content.push_str(&std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string());
    ts_content.push_str("\n\n");

    // Generate types for Product, Service, Resource structs
    ts_content.push_str(&db::Product::export_to_string().map_err(|e| e.to_string())?);
    ts_content.push_str(&db::Service::export_to_string().map_err(|e| e.to_string())?);
    ts_content.push_str(&db::Resource::export_to_string().map_err(|e| e.to_string())?);
    ts_content.push_str(&db::CreateProductRequest::export_to_string().map_err(|e| e.to_string())?);
    ts_content.push_str(&db::CreateServiceRequest::export_to_string().map_err(|e| e.to_string())?);
    ts_content.push_str(&db::CreateResourceRequest::export_to_string().map_err(|e| e.to_string())?);

    // Write to TypeScript definitions file
    let types_dir = "../src/types";
    fs::create_dir_all(types_dir).map_err(|e| format!("Failed to create types directory: {}", e))?;

    let types_path = format!("{}/generated.ts", types_dir);
    fs::write(&types_path, &ts_content).map_err(|e| format!("Failed to write TypeScript types: {}", e))?;

    Ok(format!("TypeScript types generated at {}", types_path))
}

// Configuration-driven UI commands
#[tauri::command]
async fn cd_get_resource_dictionaries(pool: State<'_, DbPool>) -> Result<Vec<db::ResourceDictionary>, String> {
    db::ConfigDrivenOperations::get_dictionaries(&pool).await
}

#[tauri::command]
async fn cd_get_resources(pool: State<'_, DbPool>, dictionary_id: i32) -> Result<Vec<db::ResourceObject>, String> {
    db::ConfigDrivenOperations::get_resources_by_dictionary(&pool, dictionary_id).await
}

#[tauri::command]
async fn cd_get_resource_config(pool: State<'_, DbPool>, resource_name: String) -> Result<Option<serde_json::Value>, String> {
    let config = db::ConfigDrivenOperations::get_full_resource_config(&pool, &resource_name).await?;
    Ok(config.map(|c| db::ConfigDrivenOperations::convert_to_frontend_format(&c)))
}

#[tauri::command]
async fn cd_get_resource_perspectives(pool: State<'_, DbPool>, resource_name: String) -> Result<Vec<String>, String> {
    db::ConfigDrivenOperations::get_resource_perspectives(&pool, &resource_name).await
}

#[tauri::command]
async fn cd_search_resources(pool: State<'_, DbPool>, search_term: String) -> Result<Vec<db::ResourceObject>, String> {
    db::ConfigDrivenOperations::search_resources(&pool, &search_term).await
}

// Learn to accept the things we cannot change...
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create async runtime for database and web server
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    // Initialize database pool using unified interface
    let db_pool = runtime.block_on(async {
        db::init_db()
            .await
            .expect("Failed to create database pool")
    });

    // Create data dictionary state
    let _data_dict_state = std::sync::Arc::new(DataDictionaryState::new(db_pool.clone()));

    // Pure desktop application - no external server required
    println!("🚀 Data Designer Desktop IDE starting...");
    println!("📦 Using bundled frontend from src/dist/");

    tauri::Builder::default()
        .manage(db_pool)
        .invoke_handler(tauri::generate_handler![
            save_rules,
            get_test_rules,
            test_rule,
            get_grammar_rules,
            update_grammar_rule,
            generate_grammar_visualization,
            load_grammar,
            save_grammar,
            get_api_keys,
            load_source_data,
            load_target_rules,
            test_rule_with_dataset,
            // Database commands
            db_get_all_rules,
            db_get_rule,
            db_create_rule,
            db_update_rule,
            db_search_rules,
            db_get_business_attributes,
            db_get_derived_attributes,
            db_get_categories,
            // Embedding commands
            db_find_similar_rules,
            db_update_rule_embedding,
            db_generate_all_embeddings,
            // AST visualization
            visualize_ast,
            // Data Dictionary
            dd_get_data_dictionary,
            dd_refresh_data_dictionary,
            dd_create_derived_attribute,
            dd_create_and_compile_rule,
            dd_search_attributes,
            // Schema visualization
            db_get_schema_info,
            db_get_table_relationships,
            db_execute_sql,
            open_schema_viewer,
            // New rule workflow commands (pool dereferencing fixed)
            check_attribute_exists,
            get_business_attributes,
            create_rule_with_template,
            get_existing_rules,
            get_rule_by_id,
            // Grammar management
            get_grammar_info,
            // Database connection check
            check_database_connection,
            // Configuration management
            get_config_info,
            // CBU Management
            create_cbu,
            get_cbu_by_id,
            list_cbus,
            get_cbu_roles,
            add_cbu_member,
            get_cbu_members,
            remove_cbu_member,
            search_cbus,
            get_cbu_roles_by_category,
            update_cbu,
            // Product Management
            create_product,
            list_products,
            get_product_hierarchy,
            create_service,
            list_services,
            get_service_by_id,
            update_service,
            create_resource,
            list_resources,
            subscribe_cbu_to_product,
            get_cbu_subscriptions,
            create_onboarding_request,
            get_onboarding_progress,
            get_lines_of_business,
            get_service_categories,
            get_resource_types,
            // DSL Transpilation
            transpile_dsl_to_rules,
            // TypeScript generation
            generate_typescript_types,
            // Configuration-driven UI
            cd_get_resource_dictionaries,
            cd_get_resources,
            cd_get_resource_config,
            cd_get_resource_perspectives,
            cd_search_resources
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}