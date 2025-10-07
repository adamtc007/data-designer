use std::collections::HashMap;
use std::fs;
use data_designer::{BusinessRule, generate_test_context};
use serde::{Deserialize, Serialize};

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
fn generate_pest_grammar() -> Result<String, String> {
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

// Learn to accept the things we cannot change...
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_rules,
            get_test_rules,
            test_rule,
            get_grammar_rules,
            update_grammar_rule,
            generate_pest_grammar,
            load_grammar,
            save_grammar
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}