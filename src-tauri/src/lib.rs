use std::collections::HashMap;
use std::fs;
use data_designer::*;
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

    // Use the library function to transpile the DSL
    let rules = data_designer::transpile_dsl_to_rules(&dsl_text)?;

    // Convert to JSON for saving
    let json_output = serde_json::to_string_pretty(&rules).map_err(|e| e.to_string())?;

    // In a real app, you'd save to user-chosen files
    std::fs::write("my_rules.rules", &dsl_text).map_err(|e| e.to_string())?;
    std::fs::write("rules.json", &json_output).map_err(|e| e.to_string())?;

    Ok("Rules saved successfully".to_string())
}

#[tauri::command]
fn get_test_rules() -> Vec<TestRule> {
    vec![
        TestRule {
            id: 1,
            name: "Complex Math".to_string(),
            dsl: r#"RULE "Complex Math" IF status == "active" THEN result = 100 + 25 * 2 - 10 / 2"#.to_string(),
            description: "Multiple arithmetic operators (-, *, /, +) with precedence".to_string(),
        },
        TestRule {
            id: 2,
            name: "String Concatenation".to_string(),
            dsl: r#"RULE "Concat String" IF country == "US" THEN message = "Hello " & name & "!""#.to_string(),
            description: "String concatenation with & operator".to_string(),
        },
        TestRule {
            id: 3,
            name: "Parentheses Precedence".to_string(),
            dsl: r#"RULE "Precedence Test" IF level > 5 THEN total = (100 + 50) * 2"#.to_string(),
            description: "Parentheses for operator precedence".to_string(),
        },
        TestRule {
            id: 4,
            name: "SUBSTRING Function".to_string(),
            dsl: r#"RULE "Extract Code" IF type == "user" THEN code = SUBSTRING(user_id, 0, 3)"#.to_string(),
            description: "SUBSTRING function for string extraction".to_string(),
        },
        TestRule {
            id: 5,
            name: "CONCAT Function".to_string(),
            dsl: r#"RULE "Build Message" IF active == "true" THEN full_message = CONCAT("User: ", name, " (", role, ")")"#.to_string(),
            description: "CONCAT function with multiple arguments".to_string(),
        },
        TestRule {
            id: 6,
            name: "LOOKUP Function".to_string(),
            dsl: r#"RULE "Country Lookup" IF region == "NA" THEN country_name = LOOKUP(country_code, "countries")"#.to_string(),
            description: "LOOKUP function with external data".to_string(),
        },
        TestRule {
            id: 7,
            name: "Ultimate Test".to_string(),
            dsl: r#"RULE "Ultimate Test" IF status == "premium" THEN result = CONCAT("Rate: ", (base_rate + LOOKUP(tier, "rates")) * 100, "%")"#.to_string(),
            description: "Complex expression with mixed operations".to_string(),
        },
        TestRule {
            id: 8,
            name: "Runtime Calculation".to_string(),
            dsl: r#"RULE "Runtime Calc" IF enabled == "yes" THEN computed = price * quantity + tax"#.to_string(),
            description: "Runtime attribute resolution".to_string(),
        }
    ]
}

#[tauri::command]
fn run_test_rule(rule_id: u32) -> TestResult {
    let test_rules = get_test_rules();

    if let Some(test_rule) = test_rules.iter().find(|r| r.id == rule_id) {
        match transpile_dsl_to_enhanced_rules(&test_rule.dsl) {
            Ok(rules) => {
                if rules.len() > 0 {
                    let engine = EnhancedRulesEngine::new(rules);

                    // Create test context based on rule ID
                    let mut context = HashMap::new();
                    match rule_id {
                        1 => {
                            context.insert("status".to_string(), LiteralValue::String("active".to_string()));
                        },
                        2 => {
                            context.insert("country".to_string(), LiteralValue::String("US".to_string()));
                            context.insert("name".to_string(), LiteralValue::String("World".to_string()));
                        },
                        3 => {
                            context.insert("level".to_string(), LiteralValue::Number(10.0));
                        },
                        4 => {
                            context.insert("type".to_string(), LiteralValue::String("user".to_string()));
                            context.insert("user_id".to_string(), LiteralValue::String("USR12345".to_string()));
                        },
                        5 => {
                            context.insert("active".to_string(), LiteralValue::String("true".to_string()));
                            context.insert("name".to_string(), LiteralValue::String("Alice".to_string()));
                            context.insert("role".to_string(), LiteralValue::String("Admin".to_string()));
                        },
                        6 => {
                            context.insert("region".to_string(), LiteralValue::String("NA".to_string()));
                            context.insert("country_code".to_string(), LiteralValue::String("US".to_string()));
                        },
                        7 => {
                            context.insert("status".to_string(), LiteralValue::String("premium".to_string()));
                            context.insert("base_rate".to_string(), LiteralValue::Number(0.05));
                            context.insert("tier".to_string(), LiteralValue::String("premium".to_string()));
                        },
                        8 => {
                            context.insert("enabled".to_string(), LiteralValue::String("yes".to_string()));
                            context.insert("price".to_string(), LiteralValue::Number(10.50));
                            context.insert("quantity".to_string(), LiteralValue::Number(3.0));
                            context.insert("tax".to_string(), LiteralValue::Number(2.15));
                        },
                        _ => {}
                    }

                    match engine.run(&context) {
                        Some(result) => TestResult {
                            success: true,
                            result: Some(format!("{:?}", result.derived_value)),
                            error: None,
                        },
                        None => TestResult {
                            success: false,
                            result: None,
                            error: Some("Rule conditions not met or execution failed".to_string()),
                        }
                    }
                } else {
                    TestResult {
                        success: false,
                        result: None,
                        error: Some("No rules generated from DSL".to_string()),
                    }
                }
            },
            Err(e) => TestResult {
                success: false,
                result: None,
                error: Some(format!("Parse error: {}", e)),
            }
        }
    } else {
        TestResult {
            success: false,
            result: None,
            error: Some("Test rule not found".to_string()),
        }
    }
}

#[tauri::command]
fn load_grammar() -> Result<DynamicGrammar, String> {
    let content = fs::read_to_string("grammar_rules.json")
        .map_err(|e| format!("Failed to read grammar file: {}", e))?;

    let grammar: DynamicGrammar = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse grammar JSON: {}", e))?;

    Ok(grammar)
}

#[tauri::command]
fn save_grammar(grammar: DynamicGrammar) -> Result<String, String> {
    let json_content = serde_json::to_string_pretty(&grammar)
        .map_err(|e| format!("Failed to serialize grammar: {}", e))?;

    fs::write("grammar_rules.json", json_content)
        .map_err(|e| format!("Failed to save grammar file: {}", e))?;

    Ok("Grammar saved successfully".to_string())
}

#[tauri::command]
fn get_grammar_rules() -> Result<Vec<GrammarRule>, String> {
    let grammar = load_grammar()?;
    Ok(grammar.grammar.rules)
}

#[tauri::command]
fn update_grammar_rule(rule: GrammarRule) -> Result<String, String> {
    let mut grammar = load_grammar()?;

    // Find and update the rule
    if let Some(existing_rule) = grammar.grammar.rules.iter_mut().find(|r| r.name == rule.name) {
        *existing_rule = rule;
    } else {
        grammar.grammar.rules.push(rule);
    }

    save_grammar(grammar)?;
    Ok("Grammar rule updated successfully".to_string())
}

#[tauri::command]
fn generate_pest_grammar() -> Result<String, String> {
    let grammar = load_grammar()?;

    let mut pest_content = String::new();

    for rule in &grammar.grammar.rules {
        match rule.rule_type.as_str() {
            "silent" => pest_content.push_str(&format!("{} = {}\n", rule.name, rule.definition)),
            "atomic" => pest_content.push_str(&format!("{} = {}\n", rule.name, rule.definition)),
            "normal" => pest_content.push_str(&format!("{} = {}\n", rule.name, rule.definition)),
            _ => pest_content.push_str(&format!("{} = {}\n", rule.name, rule.definition)),
        }
    }

    Ok(pest_content)
}

#[tauri::command]
fn validate_grammar() -> Result<bool, String> {
    let _grammar = load_grammar()?; // This will fail if grammar is invalid

    // Additional validation could be added here
    // For now, just check if it parses as valid JSON

    Ok(true)
}

// Data Dictionary Commands
#[tauri::command]
fn load_data_dictionary() -> Result<DataDictionary, String> {
    let content = fs::read_to_string("data_dictionary.json")
        .map_err(|e| format!("Failed to read data dictionary file: {}", e))?;

    let dictionary: DataDictionary = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse data dictionary JSON: {}", e))?;

    Ok(dictionary)
}

#[tauri::command]
fn save_data_dictionary(dictionary: DataDictionary) -> Result<String, String> {
    let json_content = serde_json::to_string_pretty(&dictionary)
        .map_err(|e| format!("Failed to serialize data dictionary: {}", e))?;

    fs::write("data_dictionary.json", json_content)
        .map_err(|e| format!("Failed to save data dictionary file: {}", e))?;

    Ok("Data dictionary saved successfully".to_string())
}

#[tauri::command]
fn get_attributes() -> Result<Vec<AttributeDefinition>, String> {
    let dictionary = load_data_dictionary()?;
    Ok(dictionary.attributes)
}

#[tauri::command]
fn add_attribute(attribute: AttributeDefinition) -> Result<String, String> {
    let mut dictionary = load_data_dictionary()?;

    // Check if attribute already exists
    if dictionary.attributes.iter().any(|a| a.name == attribute.name) {
        return Err(format!("Attribute '{}' already exists", attribute.name));
    }

    dictionary.attributes.push(attribute);
    save_data_dictionary(dictionary)?;
    Ok("Attribute added successfully".to_string())
}

#[tauri::command]
fn update_attribute(attribute: AttributeDefinition) -> Result<String, String> {
    let mut dictionary = load_data_dictionary()?;

    if let Some(existing) = dictionary.attributes.iter_mut().find(|a| a.name == attribute.name) {
        *existing = attribute;
        save_data_dictionary(dictionary)?;
        Ok("Attribute updated successfully".to_string())
    } else {
        Err(format!("Attribute '{}' not found", attribute.name))
    }
}

#[tauri::command]
fn delete_attribute(attribute_name: String) -> Result<String, String> {
    let mut dictionary = load_data_dictionary()?;

    let initial_len = dictionary.attributes.len();
    dictionary.attributes.retain(|a| a.name != attribute_name);

    if dictionary.attributes.len() < initial_len {
        save_data_dictionary(dictionary)?;
        Ok("Attribute deleted successfully".to_string())
    } else {
        Err(format!("Attribute '{}' not found", attribute_name))
    }
}

// Rule Compilation Commands
#[tauri::command]
fn compile_rule_to_rust(rule_dsl: String, rule_name: String) -> Result<CompiledRule, String> {
    // Parse the DSL rule first
    let rules = transpile_dsl_to_enhanced_rules(&rule_dsl)?;

    if rules.is_empty() {
        return Err("No rules generated from DSL".to_string());
    }

    let rule = &rules[0];

    // Extract input attributes from conditions and expressions
    let mut input_attributes = Vec::new();
    for condition in &rule.conditions {
        if !input_attributes.contains(&condition.attribute_name) {
            input_attributes.push(condition.attribute_name.clone());
        }
    }

    // Extract attributes from expression (simplified for now)
    collect_attributes_from_expression(&rule.action.expression, &mut input_attributes);

    // Generate Rust code
    let rust_code = generate_rust_function(&rule, &rule_name, &input_attributes)?;

    // Generate Rhai script for actual execution
    let rhai_script = generate_rhai_script(&rule, &input_attributes)?;

    let compiled_rule = CompiledRule {
        rule_id: rule.id.clone(),
        rule_name: rule_name.clone(),
        generated_code: rust_code,
        rhai_script: Some(rhai_script),
        input_attributes,
        output_attribute: rule.action.target_attribute.clone(),
        compilation_timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Load existing compiled rules and add the new one
    let mut compiled_rules = load_compiled_rules().unwrap_or_default();

    // Remove any existing rule with the same name
    compiled_rules.retain(|r| r.rule_name != rule_name);

    // Add the new compiled rule
    compiled_rules.push(compiled_rule.clone());

    // Save to file
    save_compiled_rules(&compiled_rules)?;

    Ok(compiled_rule)
}

fn collect_attributes_from_expression(expr: &Expression, attributes: &mut Vec<String>) {
    match expr {
        Expression::Attribute(name) => {
            if !attributes.contains(name) {
                attributes.push(name.clone());
            }
        }
        Expression::Add(left, right) |
        Expression::Subtract(left, right) |
        Expression::Multiply(left, right) |
        Expression::Divide(left, right) |
        Expression::Concat(left, right) => {
            collect_attributes_from_expression(left, attributes);
            collect_attributes_from_expression(right, attributes);
        }
        Expression::Substring(str_expr, start_expr, len_expr) => {
            collect_attributes_from_expression(str_expr, attributes);
            collect_attributes_from_expression(start_expr, attributes);
            collect_attributes_from_expression(len_expr, attributes);
        }
        Expression::ConcatMany(exprs) => {
            for expr in exprs {
                collect_attributes_from_expression(expr, attributes);
            }
        }
        Expression::Lookup(key_expr, _) => {
            collect_attributes_from_expression(key_expr, attributes);
        }
        _ => {} // Number and String literals don't contain attributes
    }
}

fn generate_rust_function(rule: &EnhancedRule, function_name: &str, input_attrs: &[String]) -> Result<String, String> {
    let mut code = String::new();

    // Function signature
    code.push_str(&format!("pub fn {}(", function_name.to_lowercase().replace(" ", "_")));

    // Add input parameters based on data dictionary
    let dictionary = load_data_dictionary().unwrap_or_else(|_| DataDictionary {
        metadata: GrammarMetadata {
            version: "1.0".to_string(),
            description: "".to_string(),
            created: "".to_string(),
            author: "".to_string(),
        },
        attributes: Vec::new(),
        categories: Vec::new(),
    });

    let mut params = Vec::new();
    for attr_name in input_attrs {
        if let Some(attr_def) = dictionary.attributes.iter().find(|a| &a.name == attr_name) {
            let rust_type = match attr_def.data_type.as_str() {
                "String" => "String",
                "Number" => "f64",
                "Boolean" => "bool",
                _ => "String",
            };
            params.push(format!("{}: {}", attr_name, rust_type));
        } else {
            params.push(format!("{}: String", attr_name)); // Default to String
        }
    }
    code.push_str(&params.join(", "));

    // Return type
    let output_type = if let Some(attr_def) = dictionary.attributes.iter().find(|a| a.name == rule.action.target_attribute) {
        match attr_def.data_type.as_str() {
            "String" => "String",
            "Number" => "f64",
            "Boolean" => "bool",
            _ => "String",
        }
    } else {
        "String" // Default
    };

    code.push_str(&format!(") -> Option<{}> {{\n", output_type));

    // Generate condition checks
    code.push_str("    // Condition checks\n");
    for (i, condition) in rule.conditions.iter().enumerate() {
        if i > 0 {
            code.push_str("    && ");
        } else {
            code.push_str("    if ");
        }

        let op_str = match condition.operator {
            Operator::Equals => "==",
            Operator::NotEquals => "!=",
            Operator::GreaterThan => ">",
            Operator::LessThan => "<",
            Operator::GreaterThanOrEqual => ">=",
            Operator::LessThanOrEqual => "<=",
        };

        let value_str = match &condition.value {
            LiteralValue::String(s) => format!("\"{}\"", s),
            LiteralValue::Number(n) => n.to_string(),
            LiteralValue::Boolean(b) => b.to_string(),
            LiteralValue::Null => "None".to_string(),
        };

        code.push_str(&format!("{} {} {}", condition.attribute_name, op_str, value_str));

        if i == rule.conditions.len() - 1 {
            code.push_str(" {\n");
        } else {
            code.push_str("\n");
        }
    }

    // Generate expression evaluation
    code.push_str("        // Expression evaluation\n");
    code.push_str(&format!("        Some({})\n", generate_expression_code(&rule.action.expression)?));
    code.push_str("    } else {\n");
    code.push_str("        None\n");
    code.push_str("    }\n");
    code.push_str("}\n");

    Ok(code)
}

fn generate_expression_code(expr: &Expression) -> Result<String, String> {
    match expr {
        Expression::Attribute(name) => Ok(name.clone()),
        Expression::Number(n) => Ok(n.to_string()),
        Expression::String(s) => Ok(format!("\"{}\"", s)),
        Expression::Add(left, right) => {
            Ok(format!("({} + {})", generate_expression_code(left)?, generate_expression_code(right)?))
        }
        Expression::Subtract(left, right) => {
            Ok(format!("({} - {})", generate_expression_code(left)?, generate_expression_code(right)?))
        }
        Expression::Multiply(left, right) => {
            Ok(format!("({} * {})", generate_expression_code(left)?, generate_expression_code(right)?))
        }
        Expression::Divide(left, right) => {
            Ok(format!("({} / {})", generate_expression_code(left)?, generate_expression_code(right)?))
        }
        Expression::Concat(left, right) => {
            Ok(format!("format!(\"{{}}{{}}\", {}, {})", generate_expression_code(left)?, generate_expression_code(right)?))
        }
        Expression::Substring(str_expr, start_expr, len_expr) => {
            Ok(format!("substring_helper(&{}, {} as usize, {} as usize)",
                generate_expression_code(str_expr)?,
                generate_expression_code(start_expr)?,
                generate_expression_code(len_expr)?))
        }
        Expression::ConcatMany(exprs) => {
            let expr_codes: Result<Vec<String>, String> = exprs.iter().map(generate_expression_code).collect();
            Ok(format!("format!(\"{}\", {})",
                "{} ".repeat(exprs.len()).trim(),
                expr_codes?.join(", ")))
        }
        Expression::Lookup(key_expr, table_name) => {
            Ok(format!("lookup_helper(&{}, \"{}\")", generate_expression_code(key_expr)?, table_name))
        }
    }
}

fn load_compiled_rules() -> Result<Vec<CompiledRule>, String> {
    match fs::read_to_string("compiled_rules.json") {
        Ok(content) => {
            let rules: Vec<CompiledRule> = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse compiled rules JSON: {}", e))?;
            Ok(rules)
        }
        Err(_) => {
            // File doesn't exist, return empty list
            Ok(Vec::new())
        }
    }
}

fn save_compiled_rules(rules: &[CompiledRule]) -> Result<(), String> {
    let json_content = serde_json::to_string_pretty(rules)
        .map_err(|e| format!("Failed to serialize compiled rules: {}", e))?;

    fs::write("compiled_rules.json", json_content)
        .map_err(|e| format!("Failed to save compiled rules file: {}", e))?;

    Ok(())
}

#[tauri::command]
fn get_compiled_rules() -> Result<Vec<CompiledRule>, String> {
    load_compiled_rules()
}

// Runtime execution of compiled rules
#[derive(Serialize, Deserialize)]
struct RuntimeExecutionRequest {
    rule_name: String,
    attribute_values: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct RuntimeExecutionResult {
    rule_name: String,
    output_attribute: String,
    result_value: String,
    execution_time_ms: u64,
    success: bool,
}

#[tauri::command]
fn execute_compiled_rule(request: RuntimeExecutionRequest) -> Result<RuntimeExecutionResult, String> {
    let start_time = std::time::Instant::now();

    // Load compiled rules to find the requested one
    let compiled_rules = load_compiled_rules()?;
    let rule = compiled_rules.iter()
        .find(|r| r.rule_name == request.rule_name)
        .ok_or_else(|| format!("Compiled rule '{}' not found", request.rule_name))?;

    // Execute using Rhai interpreter for real rule execution
    let result_value = execute_rhai_rule(rule, &request.attribute_values)?;
    let execution_time = start_time.elapsed().as_millis() as u64;

    Ok(RuntimeExecutionResult {
        rule_name: request.rule_name,
        output_attribute: rule.output_attribute.clone(),
        result_value,
        execution_time_ms: execution_time,
        success: true,
    })
}

fn generate_rhai_script(rule: &EnhancedRule, input_attrs: &[String]) -> Result<String, String> {
    // Convert the rule to Rhai script
    let condition_code = generate_rhai_condition(&rule.conditions)?;
    let expression_code = generate_rhai_expression(&rule.action.expression)?;

    let script = format!(
r#"// Rhai script for rule: {}
fn execute_rule(params) {{
    {}
    if {} {{
        {}
    }} else {{
        throw "Condition not met";
    }}
}}"#,
        rule.id,
        // Extract variables from params
        input_attrs.iter()
            .map(|attr| format!("let {} = params.get(\"{}\");", attr, attr))
            .collect::<Vec<_>>()
            .join("\n    "),
        condition_code,
        expression_code
    );

    Ok(script)
}

fn generate_rhai_condition(conditions: &[Condition]) -> Result<String, String> {
    if conditions.is_empty() {
        return Ok("true".to_string());
    }

    let condition_strings: Result<Vec<String>, String> = conditions.iter()
        .map(|cond| {
            let op = match cond.operator {
                Operator::Equals => "==",
                Operator::NotEquals => "!=",
                Operator::GreaterThan => ">",
                Operator::LessThan => "<",
                Operator::GreaterThanOrEqual => ">=",
                Operator::LessThanOrEqual => "<=",
            };

            let value = match &cond.value {
                LiteralValue::String(s) => format!("\"{}\"", s),
                LiteralValue::Number(n) => n.to_string(),
                LiteralValue::Boolean(b) => b.to_string(),
                LiteralValue::Null => "null".to_string(),
            };

            Ok(format!("{} {} {}", cond.attribute_name, op, value))
        })
        .collect();

    let conditions = condition_strings?;
    Ok(conditions.join(" && "))
}

fn generate_rhai_expression(expr: &Expression) -> Result<String, String> {
    match expr {
        Expression::Attribute(name) => Ok(name.clone()),
        Expression::Number(n) => Ok(n.to_string()),
        Expression::String(s) => Ok(format!("\"{}\"", s)),
        Expression::Add(left, right) => {
            let left_code = generate_rhai_expression(left)?;
            let right_code = generate_rhai_expression(right)?;
            Ok(format!("({} + {})", left_code, right_code))
        },
        Expression::Subtract(left, right) => {
            let left_code = generate_rhai_expression(left)?;
            let right_code = generate_rhai_expression(right)?;
            Ok(format!("({} - {})", left_code, right_code))
        },
        Expression::Multiply(left, right) => {
            let left_code = generate_rhai_expression(left)?;
            let right_code = generate_rhai_expression(right)?;
            Ok(format!("({} * {})", left_code, right_code))
        },
        Expression::Divide(left, right) => {
            let left_code = generate_rhai_expression(left)?;
            let right_code = generate_rhai_expression(right)?;
            Ok(format!("({} / {})", left_code, right_code))
        },
        Expression::Concat(left, right) => {
            let left_code = generate_rhai_expression(left)?;
            let right_code = generate_rhai_expression(right)?;
            Ok(format!("({} + {})", left_code, right_code)) // String concatenation in Rhai
        },
        Expression::Substring(str_expr, start_expr, len_expr) => {
            let string_code = generate_rhai_expression(str_expr)?;
            let start_code = generate_rhai_expression(start_expr)?;
            let len_code = generate_rhai_expression(len_expr)?;
            Ok(format!("{}.sub_string({}, {})", string_code, start_code, len_code))
        },
        Expression::ConcatMany(exprs) => {
            let expr_codes: Result<Vec<String>, String> = exprs.iter().map(generate_rhai_expression).collect();
            let codes = expr_codes?;
            if codes.is_empty() {
                Ok("\"\"".to_string())
            } else {
                Ok(format!("({})", codes.join(" + ")))
            }
        },
        Expression::Lookup(key_expr, table_name) => {
            let key_code = generate_rhai_expression(key_expr)?;
            // For now, return a placeholder - real lookup would need external data
            Ok(format!("lookup({}, \"{}\")", key_code, table_name))
        },
    }
}

fn execute_rhai_rule(rule: &CompiledRule, values: &std::collections::HashMap<String, String>) -> Result<String, String> {
    if let Some(rhai_script) = &rule.rhai_script {
        // Create Rhai engine
        let engine = rhai::Engine::new();

        // Create a map of input parameters
        let mut params = rhai::Map::new();
        for (key, value) in values {
            // Try to parse as number first, fallback to string
            if let Ok(num) = value.parse::<i64>() {
                params.insert(key.clone().into(), rhai::Dynamic::from(num));
            } else {
                params.insert(key.clone().into(), rhai::Dynamic::from(value.clone()));
            }
        }

        // Execute the script
        let scope = &mut rhai::Scope::new();
        scope.push("params", params);

        let result: Result<rhai::Dynamic, Box<rhai::EvalAltResult>> =
            engine.eval_with_scope(scope, &format!("{}\nexecute_rule(params)", rhai_script));

        match result {
            Ok(value) => Ok(value.to_string()),
            Err(e) => Err(format!("Rhai execution error: {}", e)),
        }
    } else {
        Err("No Rhai script available for this rule".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        save_rules,
        get_test_rules,
        run_test_rule,
        load_grammar,
        save_grammar,
        get_grammar_rules,
        update_grammar_rule,
        generate_pest_grammar,
        validate_grammar,
        load_data_dictionary,
        save_data_dictionary,
        get_attributes,
        add_attribute,
        update_attribute,
        delete_attribute,
        compile_rule_to_rust,
        get_compiled_rules,
        execute_compiled_rule
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}