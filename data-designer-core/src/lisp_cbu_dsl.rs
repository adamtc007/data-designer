//! LISP-based CBU DSL Parser
//! Elegant list processing for financial entity management
//!
//! Example syntax:
//! (create-cbu "Growth Fund Alpha" "Diversified growth fund"
//!   (entities
//!     (entity "AC001" "Alpha Corp" asset-owner)
//!     (entity "BM002" "Beta Management" investment-manager)))

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sqlx::PgPool;
use crate::dsl_utils;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LispValue {
    Symbol(String),
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<LispValue>),
    Nil,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LispCbuEntity {
    pub id: String,
    pub name: String,
    pub role: LispEntityRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LispEntityRole {
    AssetOwner,
    InvestmentManager,
    ManagingCompany,
    GeneralPartner,
    LimitedPartner,
    PrimeBroker,
    Administrator,
    Custodian,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LispCbuCommand {
    pub operation: LispCbuOperation,
    pub cbu_name: Option<String>,
    pub cbu_id: Option<String>,
    pub description: Option<String>,
    pub entities: Vec<LispCbuEntity>,
    pub metadata: HashMap<String, LispValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LispCbuOperation {
    Create,
    Update,
    Delete,
    Query,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LispCbuResult {
    pub success: bool,
    pub message: String,
    pub cbu_id: Option<String>,
    pub data: Option<LispValue>,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub enum LispDslError {
    ParseError(String),
    EvalError(String),
    ValidationError(String),
    DatabaseError(String),
    UnboundVariable(String),
    UnknownFunction(String),
    ArityMismatch { expected: usize, got: usize },
    TypeError(String),
}

impl std::fmt::Display for LispDslError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispDslError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            LispDslError::EvalError(msg) => write!(f, "Evaluation Error: {}", msg),
            LispDslError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            LispDslError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            LispDslError::UnboundVariable(var) => write!(f, "Unbound variable: {}", var),
            LispDslError::UnknownFunction(func) => write!(f, "Unknown function: {}", func),
            LispDslError::ArityMismatch { expected, got } => {
                write!(f, "Function expects {} arguments, got {}", expected, got)
            }
            LispDslError::TypeError(msg) => write!(f, "Type Error: {}", msg),
        }
    }
}

impl std::error::Error for LispDslError {}

pub struct LispCbuParser {
    pub pool: Option<PgPool>,
    environment: HashMap<String, LispValue>,
}

impl LispCbuParser {
    pub fn new(pool: Option<PgPool>) -> Self {
        let mut parser = Self {
            pool,
            environment: HashMap::new(),
        };
        parser.initialize_environment();
        parser
    }

    fn initialize_environment(&mut self) {
        // Built-in symbols
        self.environment.insert("nil".to_string(), LispValue::Nil);
        self.environment.insert("true".to_string(), LispValue::Boolean(true));
        self.environment.insert("false".to_string(), LispValue::Boolean(false));

        // Entity roles as symbols
        self.environment.insert("asset-owner".to_string(), LispValue::Symbol("asset-owner".to_string()));
        self.environment.insert("investment-manager".to_string(), LispValue::Symbol("investment-manager".to_string()));
        self.environment.insert("managing-company".to_string(), LispValue::Symbol("managing-company".to_string()));
        self.environment.insert("general-partner".to_string(), LispValue::Symbol("general-partner".to_string()));
        self.environment.insert("limited-partner".to_string(), LispValue::Symbol("limited-partner".to_string()));
        self.environment.insert("prime-broker".to_string(), LispValue::Symbol("prime-broker".to_string()));
        self.environment.insert("administrator".to_string(), LispValue::Symbol("administrator".to_string()));
        self.environment.insert("custodian".to_string(), LispValue::Symbol("custodian".to_string()));
    }

    /// Parse and execute LISP CBU DSL
    pub fn parse_and_eval(&mut self, input: &str) -> Result<LispCbuResult, LispDslError> {
        // Strip comments first
        let cleaned_input = dsl_utils::strip_comments(input);

        // Parse into S-expressions
        let expressions = self.parse(&cleaned_input)?;

        // Evaluate each expression
        let mut results = Vec::new();
        for expr in expressions {
            let result = self.eval(&expr)?;
            results.push(result);
        }

        // Convert LISP evaluation result to CBU result
        if let Some(last_result) = results.last() {
            self.lisp_value_to_cbu_result(last_result)
        } else {
            Ok(LispCbuResult {
                success: false,
                message: "No expressions to evaluate".to_string(),
                cbu_id: None,
                data: None,
                errors: vec!["Empty input".to_string()],
            })
        }
    }

    /// Parse LISP S-expressions
    fn parse(&self, input: &str) -> Result<Vec<LispValue>, LispDslError> {
        let tokens = self.tokenize(input)?;
        self.parse_tokens(&tokens)
    }

    /// Tokenize input string
    fn tokenize(&self, input: &str) -> Result<Vec<String>, LispDslError> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    chars.next(); // Skip whitespace
                }
                '(' | ')' => {
                    tokens.push(chars.next().unwrap().to_string());
                }
                '"' => {
                    // Parse string literal
                    chars.next(); // Skip opening quote
                    let mut string_content = String::new();
                    while let Some(ch) = chars.next() {
                        if ch == '"' {
                            break;
                        }
                        if ch == '\\' {
                            // Handle escape sequences
                            if let Some(escaped) = chars.next() {
                                match escaped {
                                    'n' => string_content.push('\n'),
                                    't' => string_content.push('\t'),
                                    'r' => string_content.push('\r'),
                                    '\\' => string_content.push('\\'),
                                    '"' => string_content.push('"'),
                                    _ => {
                                        string_content.push('\\');
                                        string_content.push(escaped);
                                    }
                                }
                            }
                        } else {
                            string_content.push(ch);
                        }
                    }
                    tokens.push(format!("\"{}\"", string_content));
                }
                ';' => {
                    // Skip line comments
                    while let Some(ch) = chars.next() {
                        if ch == '\n' {
                            break;
                        }
                    }
                }
                _ => {
                    // Parse symbol or number
                    let mut token = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_whitespace() || ch == '(' || ch == ')' || ch == '"' || ch == ';' {
                            break;
                        }
                        token.push(chars.next().unwrap());
                    }
                    if !token.is_empty() {
                        tokens.push(token);
                    }
                }
            }
        }

        Ok(tokens)
    }

    /// Parse tokens into S-expressions
    fn parse_tokens(&self, tokens: &[String]) -> Result<Vec<LispValue>, LispDslError> {
        let mut expressions = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            let (expr, consumed) = self.parse_expression(tokens, i)?;
            expressions.push(expr);
            i += consumed;
        }

        Ok(expressions)
    }

    /// Parse a single expression
    fn parse_expression(&self, tokens: &[String], start: usize) -> Result<(LispValue, usize), LispDslError> {
        if start >= tokens.len() {
            return Err(LispDslError::ParseError("Unexpected end of input".to_string()));
        }

        let token = &tokens[start];

        match token.as_str() {
            "(" => {
                // Parse list
                let mut list = Vec::new();
                let mut i = start + 1;

                while i < tokens.len() && tokens[i] != ")" {
                    let (expr, consumed) = self.parse_expression(tokens, i)?;
                    list.push(expr);
                    i += consumed;
                }

                if i >= tokens.len() {
                    return Err(LispDslError::ParseError("Unmatched opening parenthesis".to_string()));
                }

                Ok((LispValue::List(list), i - start + 1))
            }
            ")" => {
                Err(LispDslError::ParseError("Unexpected closing parenthesis".to_string()))
            }
            _ => {
                // Parse atom
                let value = self.parse_atom(token)?;
                Ok((value, 1))
            }
        }
    }

    /// Parse atomic values
    fn parse_atom(&self, token: &str) -> Result<LispValue, LispDslError> {
        // String literal
        if token.starts_with('"') && token.ends_with('"') && token.len() >= 2 {
            return Ok(LispValue::String(token[1..token.len()-1].to_string()));
        }

        // Number
        if let Ok(num) = token.parse::<f64>() {
            return Ok(LispValue::Number(num));
        }

        // Boolean
        match token {
            "true" => return Ok(LispValue::Boolean(true)),
            "false" => return Ok(LispValue::Boolean(false)),
            "nil" => return Ok(LispValue::Nil),
            _ => {}
        }

        // Symbol
        Ok(LispValue::Symbol(token.to_string()))
    }

    /// Evaluate LISP expression
    fn eval(&mut self, expr: &LispValue) -> Result<LispValue, LispDslError> {
        match expr {
            LispValue::Symbol(name) => {
                // Variable lookup
                self.environment.get(name)
                    .cloned()
                    .ok_or_else(|| LispDslError::UnboundVariable(name.clone()))
            }
            LispValue::List(items) if !items.is_empty() => {
                // Function call
                if let LispValue::Symbol(func_name) = &items[0] {
                    let args = &items[1..];
                    self.call_function(func_name, args)
                } else {
                    Err(LispDslError::EvalError("First element of list must be a function name".to_string()))
                }
            }
            LispValue::List(_) => {
                Ok(LispValue::Nil) // Empty list evaluates to nil
            }
            _ => {
                // Self-evaluating forms (strings, numbers, booleans, nil)
                Ok(expr.clone())
            }
        }
    }

    /// Call built-in functions
    fn call_function(&mut self, name: &str, args: &[LispValue]) -> Result<LispValue, LispDslError> {
        match name {
            "create-cbu" => self.eval_create_cbu(args),
            "update-cbu" => self.eval_update_cbu(args),
            "delete-cbu" => self.eval_delete_cbu(args),
            "query-cbu" => self.eval_query_cbu(args),
            "entities" => self.eval_entities(args),
            "entity" => self.eval_entity(args),
            "list" => Ok(LispValue::List(args.iter().map(|arg| self.eval(arg)).collect::<Result<Vec<_>, _>>()?)),
            "quote" => {
                if args.len() != 1 {
                    return Err(LispDslError::ArityMismatch { expected: 1, got: args.len() });
                }
                Ok(args[0].clone()) // Return unevaluated
            }
            _ => Err(LispDslError::UnknownFunction(name.to_string())),
        }
    }

    /// Evaluate create-cbu function
    fn eval_create_cbu(&mut self, args: &[LispValue]) -> Result<LispValue, LispDslError> {
        if args.len() < 2 {
            return Err(LispDslError::ArityMismatch {
                expected: 2,
                got: args.len()
            });
        }

        // Extract name and description
        let name_val = self.eval(&args[0])?;
        let desc_val = self.eval(&args[1])?;
        let name = self.extract_string(&name_val)?;
        let description = self.extract_string(&desc_val)?;

        // Extract entities (if provided)
        let mut entities = Vec::new();
        if args.len() > 2 {
            let entities_expr = self.eval(&args[2])?;
            entities = self.extract_entities(&entities_expr)?;
        }

        // Create CBU command structure
        let command = LispCbuCommand {
            operation: LispCbuOperation::Create,
            cbu_name: Some(name),
            cbu_id: None,
            description: Some(description),
            entities,
            metadata: HashMap::new(),
        };

        // Return success result
        Ok(LispValue::List(vec![
            LispValue::Symbol("create-cbu-result".to_string()),
            LispValue::String(command.cbu_name.unwrap_or_default()),
            LispValue::String(command.description.unwrap_or_default()),
            LispValue::Number(command.entities.len() as f64),
        ]))
    }

    /// Evaluate update-cbu function
    fn eval_update_cbu(&mut self, args: &[LispValue]) -> Result<LispValue, LispDslError> {
        if args.is_empty() {
            return Err(LispDslError::ArityMismatch { expected: 1, got: 0 });
        }

        let cbu_id_val = self.eval(&args[0])?;
        let cbu_id = self.extract_string(&cbu_id_val)?;

        Ok(LispValue::List(vec![
            LispValue::Symbol("update-cbu-result".to_string()),
            LispValue::String(cbu_id),
            LispValue::Boolean(true),
        ]))
    }

    /// Evaluate delete-cbu function
    fn eval_delete_cbu(&mut self, args: &[LispValue]) -> Result<LispValue, LispDslError> {
        if args.len() != 1 {
            return Err(LispDslError::ArityMismatch { expected: 1, got: args.len() });
        }

        let cbu_id_val = self.eval(&args[0])?;
        let cbu_id = self.extract_string(&cbu_id_val)?;

        Ok(LispValue::List(vec![
            LispValue::Symbol("delete-cbu-result".to_string()),
            LispValue::String(cbu_id),
            LispValue::Boolean(true),
        ]))
    }

    /// Evaluate query-cbu function
    fn eval_query_cbu(&mut self, _args: &[LispValue]) -> Result<LispValue, LispDslError> {
        Ok(LispValue::List(vec![
            LispValue::Symbol("query-cbu-result".to_string()),
            LispValue::List(vec![]), // Empty result set for now
        ]))
    }

    /// Evaluate entities function
    fn eval_entities(&mut self, args: &[LispValue]) -> Result<LispValue, LispDslError> {
        let mut entities = Vec::new();

        for arg in args {
            let entity_expr = self.eval(arg)?;
            if let LispValue::List(entity_data) = entity_expr {
                if !entity_data.is_empty() {
                    entities.push(LispValue::List(entity_data));
                }
            }
        }

        Ok(LispValue::List(entities))
    }

    /// Evaluate entity function
    fn eval_entity(&mut self, args: &[LispValue]) -> Result<LispValue, LispDslError> {
        if args.len() != 3 {
            return Err(LispDslError::ArityMismatch { expected: 3, got: args.len() });
        }

        let id_val = self.eval(&args[0])?;
        let name_val = self.eval(&args[1])?;
        let id = self.extract_string(&id_val)?;
        let name = self.extract_string(&name_val)?;
        let role_symbol = self.eval(&args[2])?;
        let role = self.extract_entity_role(&role_symbol)?;

        Ok(LispValue::List(vec![
            LispValue::Symbol("entity".to_string()),
            LispValue::String(id),
            LispValue::String(name),
            LispValue::Symbol(format!("{:?}", role).to_lowercase()),
        ]))
    }

    /// Extract string from LISP value
    fn extract_string(&self, value: &LispValue) -> Result<String, LispDslError> {
        match value {
            LispValue::String(s) => Ok(s.clone()),
            LispValue::Symbol(s) => Ok(s.clone()),
            _ => Err(LispDslError::TypeError(format!("Expected string, got {:?}", value))),
        }
    }

    /// Extract entity role from LISP value
    fn extract_entity_role(&self, value: &LispValue) -> Result<LispEntityRole, LispDslError> {
        let role_str = self.extract_string(value)?;
        match role_str.as_str() {
            "asset-owner" => Ok(LispEntityRole::AssetOwner),
            "investment-manager" => Ok(LispEntityRole::InvestmentManager),
            "managing-company" => Ok(LispEntityRole::ManagingCompany),
            "general-partner" => Ok(LispEntityRole::GeneralPartner),
            "limited-partner" => Ok(LispEntityRole::LimitedPartner),
            "prime-broker" => Ok(LispEntityRole::PrimeBroker),
            "administrator" => Ok(LispEntityRole::Administrator),
            "custodian" => Ok(LispEntityRole::Custodian),
            _ => Err(LispDslError::ValidationError(format!("Unknown entity role: {}", role_str))),
        }
    }

    /// Extract entities from LISP value
    fn extract_entities(&self, value: &LispValue) -> Result<Vec<LispCbuEntity>, LispDslError> {
        match value {
            LispValue::List(entities) => {
                let mut result = Vec::new();
                for entity_expr in entities {
                    if let LispValue::List(entity_data) = entity_expr {
                        if entity_data.len() >= 4 {
                            let id = self.extract_string(&entity_data[1])?;
                            let name = self.extract_string(&entity_data[2])?;
                            let role = self.extract_entity_role(&entity_data[3])?;

                            result.push(LispCbuEntity { id, name, role });
                        }
                    }
                }
                Ok(result)
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Convert LISP value to CBU result
    fn lisp_value_to_cbu_result(&self, value: &LispValue) -> Result<LispCbuResult, LispDslError> {
        match value {
            LispValue::List(items) if !items.is_empty() => {
                if let LispValue::Symbol(result_type) = &items[0] {
                    match result_type.as_str() {
                        "create-cbu-result" => {
                            let cbu_name = if items.len() > 1 {
                                self.extract_string(&items[1]).ok()
                            } else {
                                None
                            };

                            Ok(LispCbuResult {
                                success: true,
                                message: format!("CBU '{}' created successfully", cbu_name.unwrap_or_default()),
                                cbu_id: Some(format!("CBU_{}", chrono::Utc::now().timestamp())),
                                data: Some(value.clone()),
                                errors: Vec::new(),
                            })
                        }
                        "update-cbu-result" | "delete-cbu-result" => {
                            Ok(LispCbuResult {
                                success: true,
                                message: "Operation completed successfully".to_string(),
                                cbu_id: None,
                                data: Some(value.clone()),
                                errors: Vec::new(),
                            })
                        }
                        _ => {
                            Ok(LispCbuResult {
                                success: true,
                                message: "Expression evaluated successfully".to_string(),
                                cbu_id: None,
                                data: Some(value.clone()),
                                errors: Vec::new(),
                            })
                        }
                    }
                } else {
                    Ok(LispCbuResult {
                        success: true,
                        message: "List expression evaluated".to_string(),
                        cbu_id: None,
                        data: Some(value.clone()),
                        errors: Vec::new(),
                    })
                }
            }
            _ => {
                Ok(LispCbuResult {
                    success: true,
                    message: "Value evaluated successfully".to_string(),
                    cbu_id: None,
                    data: Some(value.clone()),
                    errors: Vec::new(),
                })
            }
        }
    }

    /// Generate LISP DSL from CBU data (for round-trip)
    pub fn generate_dsl_from_cbu(&self, cbu_name: &str, description: &str, entities: &[LispCbuEntity]) -> String {
        let mut dsl = String::new();

        dsl.push_str(&format!(";; Generated CBU DSL\n"));
        dsl.push_str(&format!("(create-cbu \"{}\" \"{}\"\n", cbu_name, description));

        if !entities.is_empty() {
            dsl.push_str("  (entities\n");
            for entity in entities {
                let role_str = format!("{:?}", entity.role).to_lowercase().replace("_", "-");
                dsl.push_str(&format!("    (entity \"{}\" \"{}\" {})\n",
                    entity.id, entity.name, role_str));
            }
            dsl.push_str("  )");
        }

        dsl.push_str(")");
        dsl
    }
}

impl LispValue {
    /// Pretty print LISP value
    pub fn to_pretty_string(&self) -> String {
        match self {
            LispValue::Symbol(s) => s.clone(),
            LispValue::String(s) => format!("\"{}\"", s),
            LispValue::Number(n) => n.to_string(),
            LispValue::Boolean(b) => b.to_string(),
            LispValue::List(items) => {
                let inner = items.iter()
                    .map(|item| item.to_pretty_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("({})", inner)
            }
            LispValue::Nil => "nil".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lisp_tokenizer() {
        let parser = LispCbuParser::new(None);
        let tokens = parser.tokenize("(create-cbu \"Test Fund\" \"Description\")").unwrap();
        assert_eq!(tokens, vec!["(", "create-cbu", "\"Test Fund\"", "\"Description\"", ")"]);
    }

    #[test]
    fn test_lisp_parser() {
        let parser = LispCbuParser::new(None);
        let expressions = parser.parse("(create-cbu \"Test Fund\" \"Description\")").unwrap();

        assert_eq!(expressions.len(), 1);
        if let LispValue::List(items) = &expressions[0] {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], LispValue::Symbol("create-cbu".to_string()));
            assert_eq!(items[1], LispValue::String("Test Fund".to_string()));
            assert_eq!(items[2], LispValue::String("Description".to_string()));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_create_cbu_evaluation() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.parse_and_eval(r#"
            (create-cbu "Growth Fund Alpha" "Diversified growth fund"
              (entities
                (entity "AC001" "Alpha Corp" asset-owner)
                (entity "BM002" "Beta Management" investment-manager)))
        "#).unwrap();

        assert!(result.success);
        assert!(result.message.contains("Growth Fund Alpha"));
        assert!(result.cbu_id.is_some());
    }

    #[test]
    fn test_entity_parsing() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.eval_entity(&[
            LispValue::String("AC001".to_string()),
            LispValue::String("Alpha Corp".to_string()),
            LispValue::Symbol("asset-owner".to_string()),
        ]).unwrap();

        if let LispValue::List(items) = result {
            assert_eq!(items.len(), 4);
            assert_eq!(items[0], LispValue::Symbol("entity".to_string()));
            assert_eq!(items[1], LispValue::String("AC001".to_string()));
            assert_eq!(items[2], LispValue::String("Alpha Corp".to_string()));
        } else {
            panic!("Expected list result");
        }
    }

    #[test]
    fn test_comment_handling() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.parse_and_eval(r#"
            ;; This is a comment
            (create-cbu "Test Fund" "Test Description") ;; Inline comment
            ;; Another comment
        "#).unwrap();

        assert!(result.success);
        assert!(result.message.contains("Test Fund"));
    }

    #[test]
    fn test_dsl_generation() {
        let parser = LispCbuParser::new(None);
        let entities = vec![
            LispCbuEntity {
                id: "AC001".to_string(),
                name: "Alpha Corp".to_string(),
                role: LispEntityRole::AssetOwner,
            },
            LispCbuEntity {
                id: "BM002".to_string(),
                name: "Beta Management".to_string(),
                role: LispEntityRole::InvestmentManager,
            },
        ];

        let dsl = parser.generate_dsl_from_cbu("Growth Fund Alpha", "Test fund", &entities);

        assert!(dsl.contains("create-cbu"));
        assert!(dsl.contains("Growth Fund Alpha"));
        assert!(dsl.contains("Alpha Corp"));
        assert!(dsl.contains("asset-owner"));
    }

    #[test]
    fn test_comprehensive_cbu_creation() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.parse_and_eval(r#"
            (create-cbu "Goldman Sachs Investment Fund" "Multi-strategy hedge fund operations"
              (entities
                (entity "GS001" "Goldman Sachs Asset Management" asset-owner)
                (entity "GS002" "Goldman Sachs Investment Advisors" investment-manager)
                (entity "GS003" "Goldman Sachs Services" managing-company)
                (entity "BNY001" "BNY Mellon" custodian)
                (entity "PWC001" "PricewaterhouseCoopers" administrator)))
        "#).unwrap();

        assert!(result.success);
        assert!(result.message.contains("Goldman Sachs Investment Fund"));
        assert!(result.cbu_id.is_some());
    }

    #[test]
    fn test_entity_role_variations() {
        let mut parser = LispCbuParser::new(None);

        let roles = [
            ("asset-owner", "Asset Owner"),
            ("investment-manager", "Investment Manager"),
            ("managing-company", "Managing Company"),
            ("custodian", "Custodian"),
            ("administrator", "Administrator"),
            ("prime-broker", "Prime Broker"),
        ];

        for (role_symbol, role_name) in &roles {
            let dsl = format!(r#"(entity "TEST001" "{} Entity" {})"#, role_name, role_symbol);
            let result = parser.eval_entity(&[
                LispValue::String("TEST001".to_string()),
                LispValue::String(format!("{} Entity", role_name)),
                LispValue::Symbol(role_symbol.to_string()),
            ]);

            assert!(result.is_ok(), "Role {} should be valid", role_symbol);
        }
    }

    #[test]
    fn test_error_handling_invalid_function() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.parse_and_eval("(invalid-function \"test\")");

        assert!(result.is_err());
        match result {
            Err(LispDslError::UnknownFunction(func)) => {
                assert_eq!(func, "invalid-function");
            }
            _ => panic!("Expected UnknownFunction error"),
        }
    }

    #[test]
    fn test_error_handling_arity_mismatch() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.parse_and_eval("(create-cbu)"); // Missing required arguments

        assert!(result.is_err());
        match result {
            Err(LispDslError::ArityMismatch { expected, got }) => {
                assert_eq!(expected, 2);
                assert_eq!(got, 0);
            }
            _ => panic!("Expected ArityMismatch error"),
        }
    }

    #[test]
    fn test_error_handling_malformed_s_expression() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.parse_and_eval("(create-cbu \"Test\""); // Missing closing paren

        assert!(result.is_err());
        match result {
            Err(LispDslError::ParseError(_)) => {
                // Expected parse error
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_special_characters_in_names() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.parse_and_eval(r#"
            (create-cbu "Fonds d'Investissement Européen" "European investment fund with special characters"
              (entities
                (entity "EU001" "Société Générale" asset-owner)
                (entity "EU002" "BNP Paribas Asset Management" investment-manager)))
        "#).unwrap();

        assert!(result.success);
        assert!(result.message.contains("Fonds d'Investissement Européen"));
    }

    #[test]
    fn test_nested_expressions_with_entities() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.parse_and_eval(r#"
            (create-cbu "Test Fund" "Test Description"
              (entities
                (entity "E001" "Entity One" asset-owner)
                (entity "E002" "Entity Two" investment-manager)
                (entity "E003" "Entity Three" custodian)))
        "#).unwrap();

        assert!(result.success);
        assert!(result.cbu_id.is_some());
    }

    #[test]
    fn test_cbu_update_operations() {
        let mut parser = LispCbuParser::new(None);

        // Test basic update
        let result = parser.parse_and_eval(r#"(update-cbu "CBU001")"#).unwrap();
        assert!(result.success);

        // Test delete
        let result = parser.parse_and_eval(r#"(delete-cbu "CBU001")"#).unwrap();
        assert!(result.success);

        // Test query
        let result = parser.parse_and_eval(r#"(query-cbu)"#).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_entities_function() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.parse_and_eval(r#"
            (entities
              (entity "E001" "Entity One" asset-owner)
              (entity "E002" "Entity Two" investment-manager))
        "#).unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_quote_function() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.eval(&LispValue::List(vec![
            LispValue::Symbol("quote".to_string()),
            LispValue::Symbol("test-symbol".to_string()),
        ]));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LispValue::Symbol("test-symbol".to_string()));
    }

    #[test]
    fn test_list_function() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.parse_and_eval(r#"(list "item1" "item2" "item3")"#).unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_empty_cbu_creation() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.parse_and_eval(r#"(create-cbu "Empty Fund" "Fund with no entities")"#).unwrap();

        assert!(result.success);
        assert!(result.message.contains("Empty Fund"));
    }

    #[test]
    fn test_invalid_entity_role() {
        let mut parser = LispCbuParser::new(None);
        let result = parser.eval_entity(&[
            LispValue::String("E001".to_string()),
            LispValue::String("Test Entity".to_string()),
            LispValue::Symbol("invalid-role".to_string()),
        ]);

        assert!(result.is_err());
        match result {
            Err(LispDslError::ValidationError(_)) => {
                // Expected validation error
            }
            _ => panic!("Expected ValidationError for invalid role"),
        }
    }

    #[test]
    fn test_round_trip_dsl_generation() {
        let parser = LispCbuParser::new(None);
        let entities = vec![
            LispCbuEntity {
                id: "RT001".to_string(),
                name: "Round Trip Entity".to_string(),
                role: LispEntityRole::AssetOwner,
            },
        ];

        let generated_dsl = parser.generate_dsl_from_cbu("Round Trip Fund", "Test round trip", &entities);

        // Parse the generated DSL back
        let mut parser2 = LispCbuParser::new(None);
        let result = parser2.parse_and_eval(&generated_dsl);

        assert!(result.is_ok());
        let parsed_result = result.unwrap();
        assert!(parsed_result.success);
        assert!(parsed_result.message.contains("Round Trip Fund"));
    }
}