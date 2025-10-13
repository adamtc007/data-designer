use std::collections::HashMap;
use std::fs;
use data_designer::{generate_test_context, BusinessRule, parser::{parse_rule, ASTNode as ParserASTNode}};
use serde::{Deserialize, Serialize};
use serde_json::{Value, Value as JsonValue};
use tauri::State;
use ts_rs::TS;
use sqlx::Row;

// AI Context Engine dependencies
use async_openai::{Client as OpenAIClient, types::{CreateChatCompletionRequestArgs, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs}};

// Configuration module
mod config;
use config::Config;

// Unified database interface
mod db;
use db::{DbPool, RuleOperations};
use db::{CreateRuleWithTemplateRequest, CreateRuleRequest};
use db::persistence::{PersistenceService, CompositePersistenceService};
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

// AI Context Engine data structures
#[derive(Serialize, Deserialize, Clone, Debug)]
struct GenerationExample {
    prompt: String,
    response: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AttributePerspective {
    description: String,
    #[serde(rename = "generationExamples")]
    generation_examples: Option<Vec<GenerationExample>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AttributeMetadata {
    name: String,
    #[serde(rename = "dataType")]
    data_type: String,
    description: String,
    #[serde(rename = "generationExamples")]
    generation_examples: Option<Vec<GenerationExample>>,
    perspectives: Option<HashMap<String, AttributePerspective>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ResourceConfig {
    #[serde(rename = "resourceName")]
    resource_name: String,
    description: String,
    attributes: Vec<AttributeMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AISuggestionRequest {
    user_prompt: String,
    perspective: String,
    selected_attributes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AISuggestionResponse {
    success: bool,
    generated_dsl: Option<String>,
    explanation: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RetrievedExample {
    attribute_name: String,
    example: GenerationExample,
    perspective: String,
    relevance_score: f32,
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

// ====================
// ENHANCED METADATA PROCESSING COMMANDS
// ====================

// Global user context state
use std::sync::Mutex;
static USER_CONTEXT: Mutex<Option<String>> = Mutex::new(None);

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceDictionaryFile {
    pub metadata: ResourceDictionaryMetadata,
    pub resources: Vec<ResourceObjectFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceDictionaryMetadata {
    pub version: String,
    pub description: String,
    pub author: String,
    pub creation_date: String,
    pub last_modified: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceObjectFile {
    #[serde(rename = "resourceName")]
    pub resource_name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    #[serde(rename = "ownerTeam")]
    pub owner_team: String,
    pub status: String,
    pub ui: ResourceUIConfig,
    pub attributes: Vec<AttributeObjectFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceUIConfig {
    pub layout: String,
    #[serde(rename = "groupOrder")]
    pub group_order: Vec<String>,
    pub navigation: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttributeObjectFile {
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: String,
    pub description: String,
    pub constraints: Option<serde_json::Value>,
    #[serde(rename = "persistence_locator")]
    pub persistence_locator: PersistenceLocator,
    pub ui: AttributeUIConfig,
    pub perspectives: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistenceLocator {
    pub system: String,
    pub entity: String,
    pub column: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttributeUIConfig {
    pub group: String,
    #[serde(rename = "displayOrder")]
    pub display_order: i32,
    #[serde(rename = "renderHint")]
    pub render_hint: String,
    pub label: String,
    pub placeholder: Option<String>,
    #[serde(rename = "isRequired")]
    pub is_required: bool,
    pub validation: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolvedAttributeUI {
    pub group: String,
    pub display_order: i32,
    pub render_hint: String,
    pub label: String,
    pub placeholder: Option<String>,
    pub is_required: bool,
    pub validation: Option<serde_json::Value>,
    pub description: String,
    pub ai_example: Option<String>,
}

#[tauri::command]
async fn load_resource_dictionary_from_file(file_path: String) -> Result<ResourceDictionaryFile, String> {
    let contents = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;

    let dictionary: ResourceDictionaryFile = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(dictionary)
}

#[tauri::command]
async fn save_resource_dictionary_to_file(
    file_path: String,
    dictionary: ResourceDictionaryFile
) -> Result<(), String> {
    let json_content = serde_json::to_string_pretty(&dictionary)
        .map_err(|e| format!("Failed to serialize dictionary: {}", e))?;

    std::fs::write(&file_path, json_content)
        .map_err(|e| format!("Failed to write file {}: {}", file_path, e))?;

    Ok(())
}

#[tauri::command]
async fn resolve_attribute_with_perspective(
    attribute: AttributeObjectFile,
    perspective: String
) -> Result<ResolvedAttributeUI, String> {
    let mut resolved = ResolvedAttributeUI {
        group: attribute.ui.group,
        display_order: attribute.ui.display_order,
        render_hint: attribute.ui.render_hint,
        label: attribute.ui.label,
        placeholder: attribute.ui.placeholder,
        is_required: attribute.ui.is_required,
        validation: attribute.ui.validation,
        description: attribute.description,
        ai_example: None,
    };

    // Apply perspective overrides if they exist
    if let Some(perspectives) = attribute.perspectives {
        if let Some(perspective_config) = perspectives.get(&perspective) {
            if let Some(label) = perspective_config.get("label").and_then(|v| v.as_str()) {
                resolved.label = label.to_string();
            }
            if let Some(desc) = perspective_config.get("description").and_then(|v| v.as_str()) {
                resolved.description = desc.to_string();
            }
            if let Some(ai_ex) = perspective_config.get("aiExample").and_then(|v| v.as_str()) {
                resolved.ai_example = Some(ai_ex.to_string());
            }
        }
    }

    Ok(resolved)
}

#[tauri::command]
async fn get_attribute_ui_config(
    resource_name: String,
    attribute_name: String,
    perspective: Option<String>
) -> Result<ResolvedAttributeUI, String> {
    // This would normally load from database or file
    // For now, return a mock response
    let current_perspective = perspective.unwrap_or_else(|| {
        USER_CONTEXT.lock().unwrap().clone().unwrap_or_else(|| "default".to_string())
    });

    // Mock response - in real implementation, this would load the actual attribute
    Ok(ResolvedAttributeUI {
        group: "Default Group".to_string(),
        display_order: 1,
        render_hint: "text".to_string(),
        label: format!("{} ({})", attribute_name, current_perspective),
        placeholder: Some(format!("Enter {}", attribute_name)),
        is_required: false,
        validation: None,
        description: format!("Description for {} in {} context", attribute_name, current_perspective),
        ai_example: Some(format!("AI example for {} in {} perspective", attribute_name, current_perspective)),
    })
}

#[tauri::command]
async fn set_user_context(context: String) -> Result<(), String> {
    *USER_CONTEXT.lock().unwrap() = Some(context);
    Ok(())
}

#[tauri::command]
async fn get_user_context() -> Result<Option<String>, String> {
    Ok(USER_CONTEXT.lock().unwrap().clone())
}

// Live Data Connection - Test persistence service functionality
#[tauri::command]
async fn test_live_data_connection(
    persistence_service: tauri::State<'_, std::sync::Arc<db::CompositePersistenceService>>
) -> Result<HashMap<String, JsonValue>, String> {
    println!("🔍 Testing live data connections...");

    let mut results = HashMap::new();

    // Test entity master lookup
    let entity_locator = db::PersistenceLocator {
        system: "EntityMasterDB".to_string(),
        entity: "legal_entities".to_string(),
        identifier: "entity_name".to_string(),
    };

    match persistence_service.get_value(&entity_locator, "ACME_CORP").await {
        Ok(value) => {
            results.insert("entity_name".to_string(), JsonValue::from(value));
        },
        Err(e) => {
            results.insert("entity_name_error".to_string(), JsonValue::String(e.to_string()));
        }
    }

    // Test compliance screening
    let compliance_locator = db::PersistenceLocator {
        system: "ComplianceDB".to_string(),
        entity: "screening_results".to_string(),
        identifier: "result_status".to_string(),
    };

    match persistence_service.get_value(&compliance_locator, "ACME_CORP").await {
        Ok(value) => {
            results.insert("compliance_status".to_string(), JsonValue::from(value));
        },
        Err(e) => {
            results.insert("compliance_error".to_string(), JsonValue::String(e.to_string()));
        }
    }

    // Test lookup table (Redis-style)
    let lookup_locator = db::PersistenceLocator {
        system: "CacheService".to_string(),
        entity: "countries".to_string(),
        identifier: "country_name".to_string(),
    };

    match persistence_service.get_value(&lookup_locator, "US").await {
        Ok(value) => {
            results.insert("country_lookup".to_string(), JsonValue::from(value));
        },
        Err(e) => {
            results.insert("country_error".to_string(), JsonValue::String(e.to_string()));
        }
    }

    println!("✅ Live data connection test completed");
    Ok(results)
}

// Live Data Fetching - Get values from persistence_locator configurations
#[tauri::command]
async fn fetch_live_data(
    locator: db::PersistenceLocator,
    key: String,
    persistence_service: tauri::State<'_, std::sync::Arc<db::CompositePersistenceService>>
) -> Result<JsonValue, String> {
    println!("📡 Fetching live data: {}.{}.{} for key: {}",
             locator.system, locator.entity, locator.identifier, key);

    match persistence_service.get_value(&locator, &key).await {
        Ok(value) => {
            println!("✅ Data fetched successfully");
            Ok(JsonValue::from(value))
        },
        Err(e) => {
            let error_msg = format!("Failed to fetch data: {}", e);
            println!("❌ {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
async fn get_ai_suggestion(
    request: AISuggestionRequest,
    db_pool: tauri::State<'_, db::DbPool>,
    persistence_service: tauri::State<'_, std::sync::Arc<db::CompositePersistenceService>>,
) -> Result<AISuggestionResponse, String> {
    println!("AI Context Engine: Processing request for perspective: {} (database-only)", request.perspective);

    // Step 1: Load resource configurations from PostgreSQL rules database
    let resource_configs = match load_resource_configs_from_database(&db_pool).await {
        Ok(configs) => configs,
        Err(e) => {
            return Ok(AISuggestionResponse {
                success: false,
                generated_dsl: None,
                explanation: None,
                error: Some(format!("Failed to load resource configurations from database: {}", e)),
            });
        }
    };

    // Step 2: Perform hybrid search to find relevant examples
    let retrieved_examples = perform_hybrid_search(
        &request.user_prompt,
        &request.perspective,
        &request.selected_attributes,
        &resource_configs,
    );

    // Step 3: Construct augmented prompt
    let augmented_prompt = build_augmented_prompt(
        &request.user_prompt,
        &request.perspective,
        &retrieved_examples,
        &request.selected_attributes,
    );

    // Step 4: Call LLM API
    match call_llm_api(&augmented_prompt).await {
        Ok((generated_dsl, explanation)) => Ok(AISuggestionResponse {
            success: true,
            generated_dsl: Some(generated_dsl),
            explanation: Some(explanation),
            error: None,
        }),
        Err(e) => Ok(AISuggestionResponse {
            success: false,
            generated_dsl: None,
            explanation: None,
            error: Some(format!("LLM API call failed: {}", e)),
        }),
    }
}

// Helper function to load resource configurations from PostgreSQL database
async fn load_resource_configs_from_database(
    db_pool: &db::DbPool
) -> Result<Vec<ResourceConfig>, String> {
    println!("Loading resource configurations from PostgreSQL rules database...");

    // Query active rules from the database with their AST and metadata
    let query = r#"
        SELECT
            r.rule_id,
            r.rule_name,
            r.description,
            r.rule_definition,
            r.parsed_ast,
            r.tags,
            rc.name as category_name,
            da.attribute_name,
            da.data_type,
            da.entity_name
        FROM rules r
        LEFT JOIN rule_categories rc ON r.category_id = rc.id
        LEFT JOIN derived_attributes da ON r.target_attribute_id = da.id
        WHERE r.status = 'active'
        ORDER BY r.created_at DESC
    "#;

    let rows = db::DbOperations::query_raw_all_no_params(db_pool, query).await
        .map_err(|e| format!("Database query failed: {}", e))?;

    let mut resource_configs = Vec::new();
    let mut current_config = ResourceConfig {
        resource_name: "Database_DSL_Rules".to_string(),
        description: "DSL rules and attributes loaded from PostgreSQL database".to_string(),
        attributes: Vec::new(),
    };

    for row in rows {
        // Extract rule data from database row
        let rule_id: String = row.try_get("rule_id").map_err(|e| format!("Failed to get rule_id: {}", e))?;
        let rule_name: String = row.try_get("rule_name").map_err(|e| format!("Failed to get rule_name: {}", e))?;
        let description: Option<String> = row.try_get("description").ok();
        let rule_definition: String = row.try_get("rule_definition").map_err(|e| format!("Failed to get rule_definition: {}", e))?;
        let category_name: Option<String> = row.try_get("category_name").ok();
        let attribute_name: Option<String> = row.try_get("attribute_name").ok();
        let data_type: Option<String> = row.try_get("data_type").ok();

        // Create generation examples based on the actual DSL rule
        let generation_examples = vec![
            GenerationExample {
                prompt: format!("Generate a rule similar to: {}", rule_name),
                response: rule_definition.clone(),
            },
            GenerationExample {
                prompt: format!("Create a {} validation rule", category_name.as_deref().unwrap_or("business")),
                response: rule_definition.clone(),
            },
        ];

        // Create attribute metadata from database rule
        let attribute_metadata = AttributeMetadata {
            name: attribute_name.unwrap_or_else(|| format!("rule_{}", rule_id)),
            data_type: data_type.unwrap_or_else(|| "String".to_string()),
            description: description.unwrap_or_else(|| format!("Database rule: {}", rule_name)),
            generation_examples: Some(generation_examples),
            perspectives: None, // Database rules don't need perspective switching
        };

        current_config.attributes.push(attribute_metadata);
    }

    if current_config.attributes.is_empty() {
        return Err("No active rules found in database".to_string());
    }

    let attributes_count = current_config.attributes.len();
    resource_configs.push(current_config);
    println!("Successfully loaded {} rules from database into resource configuration", attributes_count);
    Ok(resource_configs)
}

// Tauri command to reload DSL parser rules from database
#[tauri::command]
async fn reload_parser_rules_from_database(
    db_pool: tauri::State<'_, db::DbPool>,
) -> Result<serde_json::Value, String> {
    println!("🔄 Reloading DSL parser rules from PostgreSQL database...");

    let query = r#"
        SELECT
            r.rule_id,
            r.rule_name,
            r.rule_definition,
            r.parsed_ast,
            rc.name as category_name,
            r.created_at
        FROM rules r
        LEFT JOIN rule_categories rc ON r.category_id = rc.id
        WHERE r.status = 'active'
        ORDER BY r.created_at ASC
    "#;

    let rows = db::DbOperations::query_raw_all_no_params(&db_pool, query).await
        .map_err(|e| format!("Failed to query active rules: {}", e))?;

    let mut parser_rules = serde_json::Map::new();
    let mut rule_definitions = Vec::new();

    for row in rows {
        let rule_id: String = row.try_get("rule_id").map_err(|e| format!("Failed to get rule_id: {}", e))?;
        let rule_name: String = row.try_get("rule_name").map_err(|e| format!("Failed to get rule_name: {}", e))?;
        let rule_definition: String = row.try_get("rule_definition").map_err(|e| format!("Failed to get rule_definition: {}", e))?;
        let category_name: Option<String> = row.try_get("category_name").ok();

        // Build rule metadata for the parser
        let rule_metadata = serde_json::json!({
            "id": rule_id,
            "name": rule_name,
            "definition": rule_definition,
            "category": category_name.unwrap_or_else(|| "general".to_string()),
            "source": "database"
        });

        parser_rules.insert(rule_id.clone(), rule_metadata);
        rule_definitions.push(serde_json::json!({
            "id": rule_id,
            "dsl": rule_definition
        }));
    }

    let result = serde_json::json!({
        "success": true,
        "rulesLoaded": parser_rules.len(),
        "rules": parser_rules,
        "definitions": rule_definitions,
        "source": "postgresql_database",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    println!("✅ Successfully loaded {} active DSL rules from database for parser", parser_rules.len());
    Ok(result)
}

// Tauri command to save new DSL rule to database and update parser
#[tauri::command]
async fn save_rule_to_database(
    rule_id: String,
    rule_name: String,
    rule_definition: String,
    description: Option<String>,
    category: Option<String>,
    db_pool: tauri::State<'_, db::DbPool>,
) -> Result<serde_json::Value, String> {
    println!("💾 Saving new DSL rule '{}' to database...", rule_name);

    // Parse the AST for the rule (this would normally use your parser)
    let parsed_ast = match data_designer::parser::parse_rule(&rule_definition) {
        Ok(ast) => {
            // Convert AST to JSON for storage
            serde_json::to_value(&ast).unwrap_or_else(|_| serde_json::json!({}))
        },
        Err(e) => {
            println!("⚠️  Failed to parse AST for rule, storing without AST: {}", e);
            serde_json::json!({})
        }
    };

    // Get category ID if specified
    let category_id = if let Some(cat) = category {
        let category_query = "SELECT id FROM rule_categories WHERE category_key = $1 OR name = $1 LIMIT 1";
        match db::DbOperations::query_raw_all_one_param(&db_pool, category_query, &cat).await {
            Ok(rows) if !rows.is_empty() => {
                let id: i32 = rows[0].try_get("id").unwrap_or(1);
                Some(id)
            },
            _ => Some(1) // Default to first category
        }
    } else {
        Some(1) // Default category
    };

    // Insert the rule into database
    let insert_query = r#"
        INSERT INTO rules (rule_id, rule_name, description, category_id, rule_definition, parsed_ast, status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, 'active', NOW())
        ON CONFLICT (rule_id)
        DO UPDATE SET
            rule_name = EXCLUDED.rule_name,
            description = EXCLUDED.description,
            rule_definition = EXCLUDED.rule_definition,
            parsed_ast = EXCLUDED.parsed_ast,
            updated_at = NOW()
        RETURNING id
    "#;

    // Execute the insert with proper parameter binding
    let result_rows = sqlx::query(insert_query)
        .bind(&rule_id)
        .bind(&rule_name)
        .bind(&description.unwrap_or_else(|| "Generated rule".to_string()))
        .bind(category_id)
        .bind(&rule_definition)
        .bind(&parsed_ast)
        .fetch_all(&*db_pool)
        .await
        .map_err(|e| format!("Failed to save rule to database: {}", e))?;

    let db_id: i32 = if let Ok(id) = result_rows[0].try_get("id") {
        id
    } else {
        0
    };

    let result = serde_json::json!({
        "success": true,
        "ruleId": rule_id,
        "ruleName": rule_name,
        "databaseId": db_id,
        "definition": rule_definition,
        "hasAst": !parsed_ast.is_null(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    println!("✅ Successfully saved rule '{}' to database with ID {}", rule_name, db_id);
    Ok(result)
}

// Tauri command to check database state and return appropriate UI state
#[tauri::command]
async fn check_database_rule_state(
    db_pool: tauri::State<'_, db::DbPool>,
) -> Result<serde_json::Value, String> {
    println!("🔍 Checking database rule state...");

    // Count total rules in database
    let count_query = "SELECT COUNT(*) as rule_count FROM rules";
    let count_result = db::DbOperations::query_raw_all_no_params(&db_pool, count_query).await
        .map_err(|e| format!("Failed to count rules: {}", e))?;

    let total_rules: i64 = count_result[0].try_get("rule_count").unwrap_or(0);

    if total_rules == 0 {
        // GUARD CONDITION 1: No rules in DB - only show "Create New"
        return Ok(serde_json::json!({
            "hasRules": false,
            "totalRules": 0,
            "activeRules": 0,
            "inRepairRules": 0,
            "uiState": "create_only",
            "message": "No rules found in database. Create your first rule to get started.",
            "allowedActions": ["create_new"]
        }));
    }

    // Count rules by status
    let status_query = r#"
        SELECT
            status,
            COUNT(*) as count
        FROM rules
        GROUP BY status
    "#;

    let status_results = db::DbOperations::query_raw_all_no_params(&db_pool, status_query).await
        .map_err(|e| format!("Failed to count rules by status: {}", e))?;

    let mut active_count = 0;
    let mut in_repair_count = 0;
    let mut other_count = 0;

    for row in status_results {
        let status: String = row.try_get("status").unwrap_or_else(|_| "unknown".to_string());
        let count: i64 = row.try_get("count").unwrap_or(0);

        match status.as_str() {
            "active" => active_count = count,
            "in_repair" => in_repair_count = count,
            _ => other_count += count,
        }
    }

    let ui_state = if active_count > 0 {
        "normal" // Show full interface with existing rules
    } else if in_repair_count > 0 {
        "repair_needed" // Show rules that need fixing
    } else {
        "create_only" // Only inactive/draft rules exist
    };

    Ok(serde_json::json!({
        "hasRules": true,
        "totalRules": total_rules,
        "activeRules": active_count,
        "inRepairRules": in_repair_count,
        "otherRules": other_count,
        "uiState": ui_state,
        "message": format!("Found {} total rules ({} active, {} in repair)", total_rules, active_count, in_repair_count),
        "allowedActions": ["create_new", "edit_existing", "view_rules"]
    }))
}

// Tauri command to save rule with validation and status management
#[tauri::command]
async fn save_rule_with_validation(
    attribute_name: String, // PRIMARY KEY: Unique attribute name
    rule_name: String,
    rule_definition: String,
    description: Option<String>,
    category: Option<String>,
    db_pool: tauri::State<'_, db::DbPool>,
) -> Result<serde_json::Value, String> {
    println!("💾 Saving rule with attribute name '{}' (PRIMARY KEY)...", attribute_name);

    // RULE 2: Validate unique attribute name
    let existing_check = "SELECT COUNT(*) as count FROM rules WHERE rule_id = $1";
    let existing_result = db::DbOperations::query_raw_all_one_param(&db_pool, existing_check, &attribute_name).await
        .map_err(|e| format!("Failed to check existing rule: {}", e))?;

    let existing_count: i64 = existing_result[0].try_get("count").unwrap_or(0);
    let is_update = existing_count > 0;

    // Parse and validate the rule
    let (parsed_ast, validation_status, validation_error) = match data_designer::parser::parse_rule(&rule_definition) {
        Ok(ast) => {
            println!("✅ Rule validation passed for '{}'", attribute_name);
            (serde_json::to_value(&ast).unwrap_or_else(|_| serde_json::json!({})), "active", None)
        },
        Err(e) => {
            // RULE 3: Failed validation -> save with 'in_repair' status
            println!("⚠️  Rule validation failed for '{}': {}", attribute_name, e);
            (serde_json::json!({}), "in_repair", Some(e.to_string()))
        }
    };

    // Get category ID
    let category_id = if let Some(cat) = category {
        let category_query = "SELECT id FROM rule_categories WHERE category_key = $1 OR name = $1 LIMIT 1";
        match db::DbOperations::query_raw_all_one_param(&db_pool, category_query, &cat).await {
            Ok(rows) if !rows.is_empty() => {
                let id: i32 = rows[0].try_get("id").unwrap_or(1);
                Some(id)
            },
            _ => Some(1) // Default to first category
        }
    } else {
        Some(1)
    };

    // Insert or update rule using attribute_name as PRIMARY KEY
    let upsert_query = r#"
        INSERT INTO rules (rule_id, rule_name, description, category_id, rule_definition, parsed_ast, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        ON CONFLICT (rule_id)
        DO UPDATE SET
            rule_name = EXCLUDED.rule_name,
            description = EXCLUDED.description,
            category_id = EXCLUDED.category_id,
            rule_definition = EXCLUDED.rule_definition,
            parsed_ast = EXCLUDED.parsed_ast,
            status = EXCLUDED.status,
            updated_at = NOW()
        RETURNING id
    "#;

    let result_rows = sqlx::query(upsert_query)
        .bind(&attribute_name) // Using attribute_name as rule_id (PRIMARY KEY)
        .bind(&rule_name)
        .bind(&description.unwrap_or_else(|| "Generated rule".to_string()))
        .bind(category_id)
        .bind(&rule_definition)
        .bind(&parsed_ast)
        .bind(validation_status)
        .fetch_all(&*db_pool)
        .await
        .map_err(|e| format!("Failed to save rule to database: {}", e))?;

    let db_id: i32 = if let Ok(id) = result_rows[0].try_get("id") {
        id
    } else {
        0
    };

    let result = serde_json::json!({
        "success": true,
        "attributeName": attribute_name,
        "ruleName": rule_name,
        "databaseId": db_id,
        "definition": rule_definition,
        "status": validation_status,
        "isUpdate": is_update,
        "hasValidAst": validation_status == "active",
        "validationError": validation_error,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if validation_status == "active" {
        println!("✅ Successfully saved ACTIVE rule '{}' (ID: {})", attribute_name, db_id);
    } else {
        println!("⚠️  Saved rule '{}' with IN_REPAIR status due to validation failure", attribute_name);
    }

    Ok(result)
}

// Tauri command to populate database with sample DSL rules for testing
#[tauri::command]
async fn populate_sample_database_rules(
    db_pool: tauri::State<'_, db::DbPool>,
) -> Result<serde_json::Value, String> {
    println!("🌱 Populating database with sample DSL rules...");

    let sample_rules = vec![
        ("customer_risk_score", "Customer Risk Score", "100 + 25 * 2 - 10 / 2", "Mathematical risk calculation"),
        ("greeting_message", "Greeting Message", r#""Hello " & customer_name & "!""#, "Dynamic greeting generation"),
        ("account_prefix", "Account Prefix", "SUBSTRING(account_id, 0, 3)", "Extract account prefix"),
        ("country_lookup", "Country Name Lookup", "LOOKUP(country_code, \"countries\")", "Resolve country name"),
        ("pricing_formula", "Pricing Formula", r#"CONCAT("Rate: ", (base_rate + LOOKUP(tier, "rates")) * 100, "%")"#, "Complex pricing calculation"),
        ("email_validation", "Email Validation", "IS_EMAIL(email_address)", "Validate email format"),
        ("pattern_check", "Pattern Validation", r#"account_number ~ /^[A-Z]+\d+$/"#, "Account number pattern check"),
    ];

    let mut saved_count = 0;
    let mut results = Vec::new();

    for (attribute_name, rule_name, rule_definition, description) in &sample_rules {
        match save_rule_with_validation(
            attribute_name.to_string(),
            rule_name.to_string(),
            rule_definition.to_string(),
            Some(description.to_string()),
            Some("validation".to_string()),
            db_pool.clone(),
        ).await {
            Ok(result) => {
                saved_count += 1;
                results.push(result);
                println!("✅ Saved sample rule: {}", rule_name);
            },
            Err(e) => {
                println!("❌ Failed to save sample rule {}: {}", rule_name, e);
                results.push(serde_json::json!({
                    "success": false,
                    "attributeName": attribute_name,
                    "error": e
                }));
            }
        }
    }

    let response = serde_json::json!({
        "success": true,
        "totalRules": sample_rules.len(),
        "savedSuccessfully": saved_count,
        "results": results,
        "message": format!("Successfully populated {} sample rules in database", saved_count)
    });

    println!("🎉 Database population complete: {}/{} rules saved", saved_count, sample_rules.len());
    Ok(response)
}

// Tauri command to get rules that are in repair status
#[tauri::command]
async fn get_rules_in_repair(
    db_pool: tauri::State<'_, db::DbPool>,
) -> Result<serde_json::Value, String> {
    println!("🔧 Fetching rules in repair status...");

    let repair_query = r#"
        SELECT
            r.rule_id,
            r.rule_name,
            r.description,
            r.rule_definition,
            r.status,
            r.created_at,
            r.updated_at,
            rc.name as category_name
        FROM rules r
        LEFT JOIN rule_categories rc ON r.category_id = rc.id
        WHERE r.status = 'in_repair'
        ORDER BY r.updated_at DESC
    "#;

    let rows = db::DbOperations::query_raw_all_no_params(&db_pool, repair_query).await
        .map_err(|e| format!("Failed to fetch repair rules: {}", e))?;

    let mut repair_rules = Vec::new();

    for row in rows {
        let rule_id: String = row.try_get("rule_id").unwrap_or_else(|_| "unknown".to_string());
        let rule_name: String = row.try_get("rule_name").unwrap_or_else(|_| "Unnamed Rule".to_string());
        let description: Option<String> = row.try_get("description").ok();
        let rule_definition: String = row.try_get("rule_definition").unwrap_or_else(|_| "".to_string());
        let category_name: Option<String> = row.try_get("category_name").ok();

        // Try to re-validate the rule to show current error
        let validation_error = match data_designer::parser::parse_rule(&rule_definition) {
            Ok(_) => None, // Rule is now valid, could be moved to active
            Err(e) => Some(e.to_string())
        };

        repair_rules.push(serde_json::json!({
            "attributeName": rule_id,
            "ruleName": rule_name,
            "description": description,
            "ruleDefinition": rule_definition,
            "category": category_name.unwrap_or_else(|| "General".to_string()),
            "currentValidationError": validation_error,
            "canBeFixed": validation_error.is_some(),
            "lastUpdated": row.try_get::<chrono::NaiveDateTime, _>("updated_at").ok()
        }));
    }

    Ok(serde_json::json!({
        "success": true,
        "rulesInRepair": repair_rules,
        "count": repair_rules.len(),
        "message": if repair_rules.is_empty() {
            "No rules currently in repair status".to_string()
        } else {
            format!("Found {} rules that need attention", repair_rules.len())
        }
    }))
}

// Tauri command to attempt to fix a rule in repair
#[tauri::command]
async fn attempt_rule_repair(
    attribute_name: String,
    updated_rule_definition: String,
    db_pool: tauri::State<'_, db::DbPool>,
) -> Result<serde_json::Value, String> {
    println!("🔧 Attempting to repair rule '{}'...", attribute_name);

    // Validate the updated rule
    let (parsed_ast, new_status, validation_result) = match data_designer::parser::parse_rule(&updated_rule_definition) {
        Ok(ast) => {
            println!("✅ Rule repair successful for '{}'", attribute_name);
            (serde_json::to_value(&ast).unwrap_or_else(|_| serde_json::json!({})), "active", "success")
        },
        Err(e) => {
            println!("⚠️  Rule repair failed for '{}': {}", attribute_name, e);
            (serde_json::json!({}), "in_repair", "still_invalid")
        }
    };

    // Update the rule in database
    let update_query = r#"
        UPDATE rules
        SET rule_definition = $1, parsed_ast = $2, status = $3, updated_at = NOW()
        WHERE rule_id = $4
        RETURNING id, rule_name
    "#;

    let result_rows = sqlx::query(update_query)
        .bind(&updated_rule_definition)
        .bind(&parsed_ast)
        .bind(new_status)
        .bind(&attribute_name)
        .fetch_all(&*db_pool)
        .await
        .map_err(|e| format!("Failed to update rule: {}", e))?;

    if result_rows.is_empty() {
        return Err(format!("Rule with attribute name '{}' not found", attribute_name));
    }

    let db_id: i32 = result_rows[0].try_get("id").unwrap_or(0);
    let rule_name: String = result_rows[0].try_get("rule_name").unwrap_or_else(|_| "Unknown".to_string());

    Ok(serde_json::json!({
        "success": true,
        "attributeName": attribute_name,
        "ruleName": rule_name,
        "databaseId": db_id,
        "newStatus": new_status,
        "repairResult": validation_result,
        "isFixed": new_status == "active",
        "message": if new_status == "active" {
            format!("Rule '{}' successfully repaired and activated", attribute_name)
        } else {
            format!("Rule '{}' still has validation errors", attribute_name)
        }
    }))
}

// Hybrid search implementation combining semantic and structured filtering
fn perform_hybrid_search(
    user_prompt: &str,
    perspective: &str,
    selected_attributes: &[String],
    resource_configs: &[ResourceConfig],
) -> Vec<RetrievedExample> {
    let mut retrieved_examples = Vec::new();

    // Extract all generation examples from the resource configurations
    for resource in resource_configs {
        for attribute in &resource.attributes {
            // Skip if this attribute is not in the selected attributes list
            if !selected_attributes.is_empty() && !selected_attributes.contains(&attribute.name) {
                continue;
            }

            // Check global generation examples
            if let Some(examples) = &attribute.generation_examples {
                for example in examples {
                    let relevance_score = calculate_semantic_similarity(user_prompt, &example.prompt);
                    retrieved_examples.push(RetrievedExample {
                        attribute_name: attribute.name.clone(),
                        example: example.clone(),
                        perspective: "global".to_string(),
                        relevance_score,
                    });
                }
            }

            // Check perspective-specific examples
            if let Some(perspectives) = &attribute.perspectives {
                if let Some(perspective_data) = perspectives.get(perspective) {
                    if let Some(examples) = &perspective_data.generation_examples {
                        for example in examples {
                            let relevance_score = calculate_semantic_similarity(user_prompt, &example.prompt);
                            retrieved_examples.push(RetrievedExample {
                                attribute_name: attribute.name.clone(),
                                example: example.clone(),
                                perspective: perspective.to_string(),
                                relevance_score,
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by relevance score (highest first) and take top 5
    retrieved_examples.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
    retrieved_examples.truncate(5);

    retrieved_examples
}

// Enhanced semantic similarity using embeddings and fallback to word overlap
fn calculate_semantic_similarity(user_prompt: &str, example_prompt: &str) -> f32 {
    // TODO: Implement true vector similarity when embeddings are available
    // For now, use enhanced word-based similarity with domain-specific improvements

    let user_lower = user_prompt.to_lowercase();
    let example_lower = example_prompt.to_lowercase();
    let user_words: Vec<&str> = user_lower.split_whitespace().collect();
    let example_words: Vec<&str> = example_lower.split_whitespace().collect();

    // Calculate Jaccard similarity (intersection / union)
    let user_set: std::collections::HashSet<&str> = user_words.iter().cloned().collect();
    let example_set: std::collections::HashSet<&str> = example_words.iter().cloned().collect();

    let intersection = user_set.intersection(&example_set).count();
    let union = user_set.union(&example_set).count();

    let jaccard_similarity = if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    };

    // Enhanced domain-specific keyword matching with weights
    let mut domain_score = 0.0;
    let high_value_keywords = [
        ("kyc", 1.0), ("compliance", 1.0), ("sanctions", 1.0), ("risk", 0.8),
        ("validation", 0.8), ("screening", 0.9), ("verify", 0.7), ("check", 0.6),
        ("calculate", 0.7), ("derive", 0.7), ("lookup", 0.6), ("match", 0.7),
        ("entity", 0.6), ("ubo", 0.9), ("beneficial", 0.8), ("owner", 0.7),
        ("legal", 0.6), ("name", 0.5), ("aml", 1.0), ("cdd", 0.9)
    ];

    let mut matched_keywords = 0;
    let mut total_weight: f32 = 0.0;

    for (keyword, weight) in &high_value_keywords {
        if user_lower.contains(keyword) && example_lower.contains(keyword) {
            domain_score += weight;
            matched_keywords += 1;
            total_weight += weight;
        }
    }

    // Normalize domain score
    let normalized_domain_score = if matched_keywords > 0 {
        domain_score / total_weight.max(1.0)
    } else {
        0.0
    };

    // Semantic phrase matching
    let phrase_boost = calculate_phrase_similarity(&user_lower, &example_lower);

    // Combine scores with weights
    let final_score = (jaccard_similarity * 0.4) + (normalized_domain_score * 0.4) + (phrase_boost * 0.2);

    final_score.min(1.0)
}

// Calculate semantic phrase similarity
fn calculate_phrase_similarity(user_text: &str, example_text: &str) -> f32 {
    let semantic_phrases = [
        ("email validation", "email verify", 0.9),
        ("sanctions screening", "sanctions check", 0.95),
        ("beneficial owner", "ultimate beneficial owner", 0.9),
        ("risk assessment", "risk calculation", 0.85),
        ("kyc check", "customer due diligence", 0.9),
        ("compliance screening", "regulatory check", 0.8),
    ];

    let mut max_similarity: f32 = 0.0;

    for (phrase1, phrase2, similarity) in &semantic_phrases {
        if (user_text.contains(phrase1) && example_text.contains(phrase2)) ||
           (user_text.contains(phrase2) && example_text.contains(phrase1)) ||
           (user_text.contains(phrase1) && example_text.contains(phrase1)) ||
           (user_text.contains(phrase2) && example_text.contains(phrase2)) {
            max_similarity = max_similarity.max(*similarity);
        }
    }

    max_similarity
}

// Vector-based similarity function (for future integration with embeddings)
async fn calculate_vector_similarity(user_prompt: &str, example_prompt: &str) -> Result<f32, String> {
    // This would integrate with the embeddings module for true vector similarity
    // For now, return error to fall back to word-based similarity
    Err("Vector embeddings not yet available".to_string())
}

// Build augmented prompt with retrieved examples
fn build_augmented_prompt(
    user_prompt: &str,
    perspective: &str,
    retrieved_examples: &[RetrievedExample],
    selected_attributes: &[String],
) -> String {
    let mut prompt = String::new();

    // System message with context
    prompt.push_str(&format!(
        "You are an expert DSL (Domain Specific Language) generator for {} business rules. \
        Your task is to generate precise, executable DSL code based on the user's natural language request.\n\n",
        perspective
    ));

    // Add domain context
    prompt.push_str("DOMAIN CONTEXT:\n");
    prompt.push_str(&format!("- Business Perspective: {}\n", perspective));
    if !selected_attributes.is_empty() {
        prompt.push_str(&format!("- Available Attributes: {}\n", selected_attributes.join(", ")));
    }
    prompt.push_str("\n");

    // Add retrieved examples as context
    if !retrieved_examples.is_empty() {
        prompt.push_str("RELEVANT EXAMPLES FROM KNOWLEDGE BASE:\n");
        for (i, example) in retrieved_examples.iter().enumerate() {
            prompt.push_str(&format!(
                "Example {} (attribute: {}, perspective: {}):\n",
                i + 1, example.attribute_name, example.perspective
            ));
            prompt.push_str(&format!("  User Request: {}\n", example.example.prompt));
            prompt.push_str(&format!("  Generated DSL: {}\n\n", example.example.response));
        }
    }

    // Add DSL syntax guidelines
    prompt.push_str("DSL SYNTAX GUIDELINES:\n");
    prompt.push_str("- Use RULE \"name\" IF condition THEN action syntax\n");
    prompt.push_str("- Supported operators: ==, !=, >, <, >=, <=, CONTAINS, MATCHES\n");
    prompt.push_str("- Supported functions: LOOKUP(key, table), SUBSTRING(text, start, length), CONCAT(...)\n");
    prompt.push_str("- Regex support: text ~ /pattern/ or text MATCHES r\"pattern\"\n");
    prompt.push_str("- Actions: SET, ALERT, NOTIFY, REQUIRE, BLOCK_ONBOARDING, etc.\n\n");

    // Add the actual user request
    prompt.push_str("USER REQUEST:\n");
    prompt.push_str(user_prompt);
    prompt.push_str("\n\nGenerate ONLY the DSL code. Provide a brief explanation after the code.");

    prompt
}

// Call LLM API with multiple provider support
async fn call_llm_api(prompt: &str) -> Result<(String, String), String> {
    // Try OpenAI first (if API key is available)
    if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
        match call_openai_api(prompt, &openai_key).await {
            Ok(result) => return Ok(result),
            Err(e) => println!("OpenAI API failed: {}", e),
        }
    }

    // Try Anthropic Claude (if API key is available)
    if let Ok(claude_key) = std::env::var("ANTHROPIC_API_KEY") {
        match call_claude_api(prompt, &claude_key).await {
            Ok(result) => return Ok(result),
            Err(e) => println!("Claude API failed: {}", e),
        }
    }

    // Try Google Gemini (if API key is available)
    if let Ok(gemini_key) = std::env::var("GOOGLE_API_KEY") {
        match call_gemini_api(prompt, &gemini_key).await {
            Ok(result) => return Ok(result),
            Err(e) => println!("Gemini API failed: {}", e),
        }
    }

    // Fallback to rule-based generation
    println!("All LLM APIs unavailable, using fallback generation");
    Ok(generate_fallback_dsl(prompt))
}

// OpenAI API implementation
async fn call_openai_api(prompt: &str, api_key: &str) -> Result<(String, String), String> {
    let client = OpenAIClient::with_config(
        async_openai::config::OpenAIConfig::new().with_api_key(api_key)
    );

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are an expert DSL generator. Generate precise, executable DSL code.")
                .build()
                .map_err(|e| format!("Failed to build system message: {}", e))?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()
                .map_err(|e| format!("Failed to build user message: {}", e))?
                .into(),
        ])
        .max_tokens(500u16)
        .temperature(0.3)
        .build()
        .map_err(|e| format!("Failed to build request: {}", e))?;

    let response = client
        .chat()
        .create(request)
        .await
        .map_err(|e| format!("OpenAI API error: {}", e))?;

    let content = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .ok_or("No response content from OpenAI")?;

    // Split response into DSL code and explanation
    let parts: Vec<&str> = content.splitn(2, "\n\n").collect();
    let dsl_code = parts[0].trim().to_string();
    let explanation = parts.get(1).unwrap_or(&"Generated using OpenAI GPT-4").trim().to_string();

    Ok((dsl_code, explanation))
}

// Anthropic Claude API implementation using reqwest
async fn call_claude_api(prompt: &str, api_key: &str) -> Result<(String, String), String> {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": "claude-3-5-sonnet-20241022",
        "max_tokens": 1000,
        "temperature": 0.3,
        "messages": [
            {
                "role": "user",
                "content": format!("You are an expert DSL generator for financial services. Generate precise, executable DSL code based on this request:\n\n{}", prompt)
            }
        ]
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Claude API request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Claude API returned status: {}", response.status()));
    }

    let response_body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Claude response: {}", e))?;

    let content = response_body
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|text| text.as_str())
        .ok_or("No content in Claude response")?;

    // Split response into DSL code and explanation
    let parts: Vec<&str> = content.splitn(2, "\n\n").collect();
    let dsl_code = parts[0].trim().to_string();
    let explanation = parts.get(1).unwrap_or(&"Generated using Anthropic Claude").trim().to_string();

    Ok((dsl_code, explanation))
}

// Google Gemini API implementation using reqwest
async fn call_gemini_api(prompt: &str, api_key: &str) -> Result<(String, String), String> {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": format!("You are an expert DSL generator for financial services. Generate precise, executable DSL code based on this request:\n\n{}", prompt)
            }]
        }],
        "generationConfig": {
            "temperature": 0.3,
            "maxOutputTokens": 1000
        }
    });

    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-pro:generateContent?key={}", api_key);

    let response = client
        .post(&url)
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Gemini API request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Gemini API returned status: {}", response.status()));
    }

    let response_body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Gemini response: {}", e))?;

    let content = response_body
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(|parts| parts.as_array())
        .and_then(|arr| arr.first())
        .and_then(|part| part.get("text"))
        .and_then(|text| text.as_str())
        .ok_or("No content in Gemini response")?;

    // Split response into DSL code and explanation
    let parts: Vec<&str> = content.splitn(2, "\n\n").collect();
    let dsl_code = parts[0].trim().to_string();
    let explanation = parts.get(1).unwrap_or(&"Generated using Google Gemini").trim().to_string();

    Ok((dsl_code, explanation))
}

// Fallback DSL generation using rule templates
fn generate_fallback_dsl(prompt: &str) -> (String, String) {
    let prompt_lower = prompt.to_lowercase();

    // Pattern-based rule generation
    if prompt_lower.contains("validation") || prompt_lower.contains("verify") {
        if prompt_lower.contains("email") {
            return (
                "RULE \"Email-Validation\" IF IS_EMAIL(email_address) == false THEN ALERT 'Invalid email format'".to_string(),
                "Generated validation rule for email addresses using built-in IS_EMAIL function.".to_string()
            );
        }
        if prompt_lower.contains("sanctions") || prompt_lower.contains("screening") {
            return (
                "RULE \"Sanctions-Check\" IF sanctions_screening_result == 'CONFIRMED_MATCH' THEN BLOCK_ONBOARDING AND ALERT 'Sanctions match detected'".to_string(),
                "Generated sanctions screening rule that blocks onboarding for confirmed matches.".to_string()
            );
        }
    }

    if prompt_lower.contains("calculate") || prompt_lower.contains("derive") {
        if prompt_lower.contains("risk") {
            return (
                "DERIVE risk_score FROM country_risk_rating * 0.3 + business_risk_rating * 0.4 + ubo_risk_rating * 0.3".to_string(),
                "Generated risk score calculation using weighted factors.".to_string()
            );
        }
    }

    if prompt_lower.contains("lookup") || prompt_lower.contains("table") {
        return (
            "RULE \"Lookup-Example\" SET result = LOOKUP(country_code, \"countries\")".to_string(),
            "Generated lookup rule to retrieve data from external table.".to_string()
        );
    }

    // Default generic rule
    (
        "RULE \"Generated-Rule\" IF condition THEN action".to_string(),
        "Generated template rule. Please provide more specific requirements for better results.".to_string()
    )
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

    // Initialize Persistence Service with dependency injection
    let persistence_service = std::sync::Arc::new(
        db::CompositePersistenceService::new()
            .with_postgres(db_pool.clone())
            .with_redis("redis://localhost:6379".to_string())
    );

    // Pure desktop application - no external server required
    println!("🚀 Data Designer Desktop IDE starting...");
    println!("💾 Persistence services initialized with PostgreSQL and Redis support");
    println!("📦 Using bundled frontend from src/dist/");

    tauri::Builder::default()
        .manage(db_pool)
        .manage(persistence_service)
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
            cd_search_resources,
            // New enhanced metadata commands
            load_resource_dictionary_from_file,
            save_resource_dictionary_to_file,
            resolve_attribute_with_perspective,
            get_attribute_ui_config,
            set_user_context,
            get_user_context,
            // AI Context Engine (Database-Only)
            get_ai_suggestion,
            // Live Data Connection
            test_live_data_connection,
            fetch_live_data,
            // Dynamic Parser Rules (Database-Driven) with Guard Conditions
            check_database_rule_state,
            save_rule_with_validation,
            get_rules_in_repair,
            attempt_rule_repair,
            reload_parser_rules_from_database,
            save_rule_to_database,
            populate_sample_database_rules
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}