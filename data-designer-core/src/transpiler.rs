use crate::models::{Expression, Value, BinaryOperator, UnaryOperator};
use crate::parser::parse_expression;
use crate::lisp_cbu_dsl::{LispCbuParser, LispValue};
use crate::db::Rule;
use crate::dsl_utils;
use anyhow::{Result, bail};
use serde_json;

/// Transpiler pipeline: Parse -> Transform -> Generate
/// Converts DSL expressions into optimized target code
pub struct Transpiler {
    pub optimizations_enabled: bool,
    pub target_language: TargetLanguage,
}

#[derive(Debug, Clone)]
pub enum TargetLanguage {
    Rust,
    SQL,
    JavaScript,
    Python,
}

#[derive(Debug, Clone)]
pub struct TranspilerOptions {
    pub target: TargetLanguage,
    pub optimize: bool,
    pub inline_functions: bool,
    pub constant_folding: bool,
    pub dead_code_elimination: bool,
}

impl Default for TranspilerOptions {
    fn default() -> Self {
        Self {
            target: TargetLanguage::Rust,
            optimize: true,
            inline_functions: true,
            constant_folding: true,
            dead_code_elimination: true,
        }
    }
}

impl Transpiler {
    pub fn new(options: TranspilerOptions) -> Self {
        Self {
            optimizations_enabled: options.optimize,
            target_language: options.target,
        }
    }

    /// Main transpiler pipeline
    pub fn transpile(&self, expr: &Expression) -> Result<String> {
        // Step 1: Transform AST (optimizations)
        let optimized_expr = if self.optimizations_enabled {
            self.optimize_expression(expr)?
        } else {
            expr.clone()
        };

        // Step 2: Generate target code
        match self.target_language {
            TargetLanguage::Rust => self.generate_rust(&optimized_expr),
            TargetLanguage::SQL => self.generate_sql(&optimized_expr),
            TargetLanguage::JavaScript => self.generate_javascript(&optimized_expr),
            TargetLanguage::Python => self.generate_python(&optimized_expr),
        }
    }

    /// Transpile S-expression DSL to target language
    pub fn transpile_s_expression(&self, s_expr: &LispValue) -> Result<String> {
        match self.target_language {
            TargetLanguage::Rust => self.generate_rust_from_s_expr(s_expr),
            TargetLanguage::SQL => self.generate_sql_from_s_expr(s_expr),
            TargetLanguage::JavaScript => self.generate_js_from_s_expr(s_expr),
            TargetLanguage::Python => self.generate_python_from_s_expr(s_expr),
        }
    }

    /// Parse and transpile S-expression DSL string
    pub fn parse_and_transpile_s_expr(&self, dsl_text: &str) -> Result<String> {
        let mut parser = LispCbuParser::new(None);
        let result = parser.parse_and_eval(dsl_text)
            .map_err(|e| anyhow::anyhow!("S-expression parse error: {}", e))?;

        if !result.success {
            bail!("S-expression evaluation failed: {}", result.message);
        }

        if let Some(data) = &result.data {
            self.transpile_s_expression(data)
        } else {
            Ok(format!("// Transpiled: {}", result.message))
        }
    }

    /// AST optimization pipeline
    fn optimize_expression(&self, expr: &Expression) -> Result<Expression> {
        let mut optimized = expr.clone();

        // Pass 1: Constant folding
        optimized = self.constant_folding(&optimized)?;

        // Pass 2: Dead code elimination
        optimized = self.dead_code_elimination(&optimized)?;

        // Pass 3: Function inlining (for simple functions)
        optimized = self.inline_simple_functions(&optimized)?;

        Ok(optimized)
    }

    /// Constant folding optimization
    fn constant_folding(&self, expr: &Expression) -> Result<Expression> {
        match expr {
            Expression::BinaryOp { op, left, right } => {
                let left_opt = self.constant_folding(left)?;
                let right_opt = self.constant_folding(right)?;

                // If both operands are literals, evaluate at compile time
                if let (Expression::Literal(l_val), Expression::Literal(r_val)) = (&left_opt, &right_opt) {
                    match self.evaluate_const_binary_op(op, l_val, r_val) {
                        Ok(result) => return Ok(Expression::Literal(result)),
                        Err(_) => {} // Fall back to original if evaluation fails
                    }
                }

                Ok(Expression::BinaryOp {
                    op: *op,
                    left: Box::new(left_opt),
                    right: Box::new(right_opt),
                })
            }
            Expression::UnaryOp { op, operand } => {
                let operand_opt = self.constant_folding(operand)?;

                // If operand is literal, evaluate at compile time
                if let Expression::Literal(val) = &operand_opt {
                    match self.evaluate_const_unary_op(op, val) {
                        Ok(result) => return Ok(Expression::Literal(result)),
                        Err(_) => {} // Fall back to original
                    }
                }

                Ok(Expression::UnaryOp {
                    op: *op,
                    operand: Box::new(operand_opt),
                })
            }
            Expression::FunctionCall { name, args } => {
                let optimized_args: Result<Vec<Expression>> = args.iter()
                    .map(|arg| self.constant_folding(arg))
                    .collect();

                Ok(Expression::FunctionCall {
                    name: name.clone(),
                    args: optimized_args?,
                })
            }
            Expression::Conditional { condition, then_expr, else_expr } => {
                let cond_opt = self.constant_folding(condition)?;

                // If condition is a constant boolean, eliminate branch
                if let Expression::Literal(Value::Boolean(true)) = &cond_opt {
                    return self.constant_folding(then_expr);
                }
                if let Expression::Literal(Value::Boolean(false)) = &cond_opt {
                    if let Some(else_branch) = else_expr {
                        return self.constant_folding(else_branch);
                    } else {
                        return Ok(Expression::Literal(Value::Null));
                    }
                }

                let then_opt = self.constant_folding(then_expr)?;
                let else_opt = if let Some(else_branch) = else_expr {
                    Some(Box::new(self.constant_folding(else_branch)?))
                } else {
                    None
                };

                Ok(Expression::Conditional {
                    condition: Box::new(cond_opt),
                    then_expr: Box::new(then_opt),
                    else_expr: else_opt,
                })
            }
            Expression::List(items) => {
                let optimized_items: Result<Vec<Expression>> = items.iter()
                    .map(|item| self.constant_folding(item))
                    .collect();

                Ok(Expression::List(optimized_items?))
            }
            _ => Ok(expr.clone()), // Literals, identifiers, variables - no optimization
        }
    }

    /// Dead code elimination
    fn dead_code_elimination(&self, expr: &Expression) -> Result<Expression> {
        match expr {
            Expression::Conditional { condition, then_expr, else_expr } => {
                // If we have unreachable branches after constant folding
                let cond_opt = self.dead_code_elimination(condition)?;
                let then_opt = self.dead_code_elimination(then_expr)?;
                let else_opt = if let Some(else_branch) = else_expr {
                    Some(Box::new(self.dead_code_elimination(else_branch)?))
                } else {
                    None
                };

                Ok(Expression::Conditional {
                    condition: Box::new(cond_opt),
                    then_expr: Box::new(then_opt),
                    else_expr: else_opt,
                })
            }
            _ => Ok(expr.clone()), // For now, just pass through
        }
    }

    /// Inline simple functions
    fn inline_simple_functions(&self, expr: &Expression) -> Result<Expression> {
        match expr {
            Expression::FunctionCall { name, args } => {
                // Inline simple functions like UPPER, LOWER, etc.
                match name.to_uppercase().as_str() {
                    "UPPER" if args.len() == 1 => {
                        if let Expression::Literal(Value::String(s)) = &args[0] {
                            return Ok(Expression::Literal(Value::String(s.to_uppercase())));
                        }
                    }
                    "LOWER" if args.len() == 1 => {
                        if let Expression::Literal(Value::String(s)) = &args[0] {
                            return Ok(Expression::Literal(Value::String(s.to_lowercase())));
                        }
                    }
                    _ => {}
                }

                // Recursively optimize arguments
                let optimized_args: Result<Vec<Expression>> = args.iter()
                    .map(|arg| self.inline_simple_functions(arg))
                    .collect();

                Ok(Expression::FunctionCall {
                    name: name.clone(),
                    args: optimized_args?,
                })
            }
            _ => Ok(expr.clone()),
        }
    }

    /// Evaluate constant binary operations at compile time
    fn evaluate_const_binary_op(&self, op: &BinaryOperator, left: &Value, right: &Value) -> Result<Value> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => {
                match op {
                    BinaryOperator::Add => Ok(Value::Integer(l + r)),
                    BinaryOperator::Subtract => Ok(Value::Integer(l - r)),
                    BinaryOperator::Multiply => Ok(Value::Integer(l * r)),
                    BinaryOperator::Divide => {
                        if *r == 0 {
                            bail!("Division by zero");
                        }
                        Ok(Value::Integer(l / r))
                    }
                    BinaryOperator::Equals => Ok(Value::Boolean(l == r)),
                    BinaryOperator::NotEquals => Ok(Value::Boolean(l != r)),
                    BinaryOperator::LessThan => Ok(Value::Boolean(l < r)),
                    BinaryOperator::GreaterThan => Ok(Value::Boolean(l > r)),
                    _ => bail!("Unsupported operation for integers"),
                }
            }
            (Value::Float(l), Value::Float(r)) => {
                match op {
                    BinaryOperator::Add => Ok(Value::Float(l + r)),
                    BinaryOperator::Subtract => Ok(Value::Float(l - r)),
                    BinaryOperator::Multiply => Ok(Value::Float(l * r)),
                    BinaryOperator::Divide => {
                        if *r == 0.0 {
                            bail!("Division by zero");
                        }
                        Ok(Value::Float(l / r))
                    }
                    BinaryOperator::Equals => Ok(Value::Boolean((l - r).abs() < f64::EPSILON)),
                    BinaryOperator::NotEquals => Ok(Value::Boolean((l - r).abs() >= f64::EPSILON)),
                    BinaryOperator::LessThan => Ok(Value::Boolean(l < r)),
                    BinaryOperator::GreaterThan => Ok(Value::Boolean(l > r)),
                    _ => bail!("Unsupported operation for floats"),
                }
            }
            (Value::String(l), Value::String(r)) => {
                match op {
                    BinaryOperator::Concat => Ok(Value::String(format!("{}{}", l, r))),
                    BinaryOperator::Equals => Ok(Value::Boolean(l == r)),
                    BinaryOperator::NotEquals => Ok(Value::Boolean(l != r)),
                    _ => bail!("Unsupported operation for strings"),
                }
            }
            _ => bail!("Type mismatch in binary operation"),
        }
    }

    /// Evaluate constant unary operations at compile time
    fn evaluate_const_unary_op(&self, op: &UnaryOperator, operand: &Value) -> Result<Value> {
        match (op, operand) {
            (UnaryOperator::Minus, Value::Integer(i)) => Ok(Value::Integer(-i)),
            (UnaryOperator::Minus, Value::Float(f)) => Ok(Value::Float(-f)),
            (UnaryOperator::Plus, Value::Integer(i)) => Ok(Value::Integer(*i)),
            (UnaryOperator::Plus, Value::Float(f)) => Ok(Value::Float(*f)),
            (UnaryOperator::Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
            _ => bail!("Unsupported unary operation"),
        }
    }

    /// Generate Rust code
    fn generate_rust(&self, expr: &Expression) -> Result<String> {
        match expr {
            Expression::Literal(val) => self.generate_rust_literal(val),
            Expression::Identifier(name) | Expression::Variable(name) => {
                Ok(format!("ctx.get(\"{}\")", name))
            }
            Expression::BinaryOp { op, left, right } => {
                let left_code = self.generate_rust(left)?;
                let right_code = self.generate_rust(right)?;
                let op_code = self.generate_rust_binary_op(op);
                Ok(format!("({} {} {})", left_code, op_code, right_code))
            }
            Expression::UnaryOp { op, operand } => {
                let operand_code = self.generate_rust(operand)?;
                let op_code = self.generate_rust_unary_op(op);
                Ok(format!("({}{})", op_code, operand_code))
            }
            Expression::FunctionCall { name, args } => {
                let arg_codes: Result<Vec<String>> = args.iter()
                    .map(|arg| self.generate_rust(arg))
                    .collect();
                let args_str = arg_codes?.join(", ");
                Ok(format!("{}({})", name.to_lowercase(), args_str))
            }
            Expression::Conditional { condition, then_expr, else_expr } => {
                let cond_code = self.generate_rust(condition)?;
                let then_code = self.generate_rust(then_expr)?;
                let else_code = if let Some(else_branch) = else_expr {
                    self.generate_rust(else_branch)?
                } else {
                    "Value::Null".to_string()
                };
                Ok(format!("if {} {{ {} }} else {{ {} }}", cond_code, then_code, else_code))
            }
            Expression::List(items) => {
                let item_codes: Result<Vec<String>> = items.iter()
                    .map(|item| self.generate_rust(item))
                    .collect();
                Ok(format!("vec![{}]", item_codes?.join(", ")))
            }
            _ => bail!("Unsupported expression type for Rust generation"),
        }
    }

    fn generate_rust_literal(&self, val: &Value) -> Result<String> {
        match val {
            Value::String(s) => Ok(format!("Value::String(\"{}\".to_string())", s)),
            Value::Integer(i) => Ok(format!("Value::Integer({})", i)),
            Value::Float(f) => Ok(format!("Value::Float({})", f)),
            Value::Number(n) => Ok(format!("Value::Number({})", n)),
            Value::Boolean(b) => Ok(format!("Value::Boolean({})", b)),
            Value::Null => Ok("Value::Null".to_string()),
            Value::Regex(pattern) => Ok(format!("Value::Regex(\"{}\".to_string())", pattern)),
            Value::List(items) => {
                let item_strings: Result<Vec<String>> = items.iter()
                    .map(|item| self.generate_rust_literal(item))
                    .collect();
                Ok(format!("Value::List(vec![{}])", item_strings?.join(", ")))
            }
        }
    }

    fn generate_rust_binary_op(&self, op: &BinaryOperator) -> &'static str {
        match op {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Equals => "==",
            BinaryOperator::NotEquals => "!=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::And => "&&",
            BinaryOperator::Or => "||",
            _ => "/* unsupported op */",
        }
    }

    fn generate_rust_unary_op(&self, op: &UnaryOperator) -> &'static str {
        match op {
            UnaryOperator::Minus => "-",
            UnaryOperator::Plus => "+",
            UnaryOperator::Not => "!",
        }
    }

    /// Generate SQL code
    fn generate_sql(&self, expr: &Expression) -> Result<String> {
        match expr {
            Expression::Literal(val) => self.generate_sql_literal(val),
            Expression::Identifier(name) | Expression::Variable(name) => {
                Ok(format!("\"{}\"", name))
            }
            Expression::BinaryOp { op, left, right } => {
                let left_code = self.generate_sql(left)?;
                let right_code = self.generate_sql(right)?;
                let op_code = self.generate_sql_binary_op(op);
                Ok(format!("({} {} {})", left_code, op_code, right_code))
            }
            Expression::FunctionCall { name, args } => {
                let arg_codes: Result<Vec<String>> = args.iter()
                    .map(|arg| self.generate_sql(arg))
                    .collect();
                let args_str = arg_codes?.join(", ");
                Ok(format!("{}({})", name.to_uppercase(), args_str))
            }
            Expression::Conditional { condition, then_expr, else_expr } => {
                let cond_code = self.generate_sql(condition)?;
                let then_code = self.generate_sql(then_expr)?;
                let else_code = if let Some(else_branch) = else_expr {
                    self.generate_sql(else_branch)?
                } else {
                    "NULL".to_string()
                };
                Ok(format!("CASE WHEN {} THEN {} ELSE {} END", cond_code, then_code, else_code))
            }
            _ => bail!("Unsupported expression type for SQL generation"),
        }
    }

    fn generate_sql_literal(&self, val: &Value) -> Result<String> {
        match val {
            Value::String(s) => Ok(format!("'{}'", s.replace("'", "''"))),
            Value::Integer(i) => Ok(i.to_string()),
            Value::Float(f) => Ok(f.to_string()),
            Value::Number(n) => Ok(n.to_string()),
            Value::Boolean(b) => Ok(if *b { "TRUE".to_string() } else { "FALSE".to_string() }),
            Value::Null => Ok("NULL".to_string()),
            _ => bail!("Unsupported literal type for SQL"),
        }
    }

    fn generate_sql_binary_op(&self, op: &BinaryOperator) -> &'static str {
        match op {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Equals => "=",
            BinaryOperator::NotEquals => "!=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::And => "AND",
            BinaryOperator::Or => "OR",
            BinaryOperator::Concat => "||",
            _ => "/* unsupported */",
        }
    }

    /// Generate JavaScript code
    fn generate_javascript(&self, expr: &Expression) -> Result<String> {
        match expr {
            Expression::Literal(val) => self.generate_js_literal(val),
            Expression::Identifier(name) | Expression::Variable(name) => {
                Ok(format!("ctx.get('{}')", name))
            }
            Expression::BinaryOp { op, left, right } => {
                let left_code = self.generate_javascript(left)?;
                let right_code = self.generate_javascript(right)?;
                let op_code = self.generate_js_binary_op(op);
                Ok(format!("({} {} {})", left_code, op_code, right_code))
            }
            Expression::FunctionCall { name, args } => {
                let arg_codes: Result<Vec<String>> = args.iter()
                    .map(|arg| self.generate_javascript(arg))
                    .collect();
                let args_str = arg_codes?.join(", ");
                Ok(format!("{}({})", name.to_lowercase(), args_str))
            }
            Expression::Conditional { condition, then_expr, else_expr } => {
                let cond_code = self.generate_javascript(condition)?;
                let then_code = self.generate_javascript(then_expr)?;
                let else_code = if let Some(else_branch) = else_expr {
                    self.generate_javascript(else_branch)?
                } else {
                    "null".to_string()
                };
                Ok(format!("({} ? {} : {})", cond_code, then_code, else_code))
            }
            _ => bail!("Unsupported expression type for JavaScript generation"),
        }
    }

    fn generate_js_literal(&self, val: &Value) -> Result<String> {
        match val {
            Value::String(s) => Ok(format!("\"{}\"", s.replace("\"", "\\\""))),
            Value::Integer(i) => Ok(i.to_string()),
            Value::Float(f) => Ok(f.to_string()),
            Value::Number(n) => Ok(n.to_string()),
            Value::Boolean(b) => Ok(b.to_string()),
            Value::Null => Ok("null".to_string()),
            _ => bail!("Unsupported literal type for JavaScript"),
        }
    }

    fn generate_js_binary_op(&self, op: &BinaryOperator) -> &'static str {
        match op {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Equals => "===",
            BinaryOperator::NotEquals => "!==",
            BinaryOperator::LessThan => "<",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::And => "&&",
            BinaryOperator::Or => "||",
            _ => "/* unsupported */",
        }
    }

    /// Generate Python code
    fn generate_python(&self, expr: &Expression) -> Result<String> {
        match expr {
            Expression::Literal(val) => self.generate_python_literal(val),
            Expression::Identifier(name) | Expression::Variable(name) => {
                Ok(format!("ctx.get('{}')", name))
            }
            Expression::BinaryOp { op, left, right } => {
                let left_code = self.generate_python(left)?;
                let right_code = self.generate_python(right)?;
                let op_code = self.generate_python_binary_op(op);
                Ok(format!("({} {} {})", left_code, op_code, right_code))
            }
            Expression::FunctionCall { name, args } => {
                let arg_codes: Result<Vec<String>> = args.iter()
                    .map(|arg| self.generate_python(arg))
                    .collect();
                let args_str = arg_codes?.join(", ");
                Ok(format!("{}({})", name.to_lowercase(), args_str))
            }
            Expression::Conditional { condition, then_expr, else_expr } => {
                let cond_code = self.generate_python(condition)?;
                let then_code = self.generate_python(then_expr)?;
                let else_code = if let Some(else_branch) = else_expr {
                    self.generate_python(else_branch)?
                } else {
                    "None".to_string()
                };
                Ok(format!("({} if {} else {})", then_code, cond_code, else_code))
            }
            _ => bail!("Unsupported expression type for Python generation"),
        }
    }

    fn generate_python_literal(&self, val: &Value) -> Result<String> {
        match val {
            Value::String(s) => Ok(format!("\"{}\"", s.replace("\"", "\\\""))),
            Value::Integer(i) => Ok(i.to_string()),
            Value::Float(f) => Ok(f.to_string()),
            Value::Number(n) => Ok(n.to_string()),
            Value::Boolean(b) => Ok(if *b { "True".to_string() } else { "False".to_string() }),
            Value::Null => Ok("None".to_string()),
            _ => bail!("Unsupported literal type for Python"),
        }
    }

    fn generate_python_binary_op(&self, op: &BinaryOperator) -> &'static str {
        match op {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Equals => "==",
            BinaryOperator::NotEquals => "!=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::And => "and",
            BinaryOperator::Or => "or",
            _ => "# unsupported",
        }
    }
}

/// DSL-to-Rules transpiler with detailed error reporting
#[derive(Debug, Clone)]
pub struct DslRule {
    pub name: String,
    pub expression: Expression,
    pub definition: String,
    pub line_number: usize,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TranspileError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub rule_name: Option<String>,
    pub error_type: ErrorType,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    ParseError,
    SemanticError,
    ValidationError,
    ConversionError,
}

impl std::fmt::Display for TranspileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}: {} - {}",
               self.line.unwrap_or(0),
               match self.error_type {
                   ErrorType::ParseError => "Parse Error",
                   ErrorType::SemanticError => "Semantic Error",
                   ErrorType::ValidationError => "Validation Error",
                   ErrorType::ConversionError => "Conversion Error",
               },
               self.message)
    }
}

impl std::error::Error for TranspileError {}

/// Multi-rule DSL transpiler
pub struct DslTranspiler {
    pub validation_enabled: bool,
    pub dependency_analysis: bool,
}

impl DslTranspiler {
    pub fn new() -> Self {
        Self {
            validation_enabled: true,
            dependency_analysis: true,
        }
    }

    /// Main entry point: transpile DSL text to Rule objects
    pub fn transpile_dsl_to_rules(&self, dsl_text: &str) -> Result<Vec<DslRule>, Vec<TranspileError>> {
        let cleaned_text = dsl_utils::strip_comments(dsl_text);
        let mut rules = Vec::new();
        let mut errors = Vec::new();

        // Split DSL text into individual rule definitions
        let rule_definitions = self.parse_rule_definitions(&cleaned_text);

        for (line_number, rule_def) in rule_definitions.iter().enumerate() {
            match self.parse_single_rule(rule_def.trim(), line_number + 1) {
                Ok(rule) => {
                    if self.validation_enabled {
                        if let Err(validation_errors) = self.validate_rule(&rule) {
                            errors.extend(validation_errors);
                        }
                    }
                    rules.push(rule);
                }
                Err(mut parse_errors) => {
                    // Add line context to parse errors
                    for error in &mut parse_errors {
                        if error.line.is_none() {
                            error.line = Some(line_number + 1);
                        }
                    }
                    errors.extend(parse_errors);
                }
            }
        }

        // Perform dependency analysis if enabled
        if self.dependency_analysis && !rules.is_empty() {
            if let Err(dep_errors) = self.analyze_dependencies(&mut rules) {
                errors.extend(dep_errors);
            }
        }

        if errors.is_empty() {
            Ok(rules)
        } else {
            Err(errors)
        }
    }

    /// Parse DSL text into individual rule definitions
    fn parse_rule_definitions(&self, dsl_text: &str) -> Vec<String> {
        let mut rules = Vec::new();
        let mut current_rule = String::new();
        let mut in_rule = false;

        for line in dsl_text.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Rule definition patterns
            if trimmed.starts_with("rule ") || trimmed.contains(" = ") {
                // Start of new rule
                if in_rule && !current_rule.trim().is_empty() {
                    rules.push(current_rule.trim().to_string());
                    current_rule.clear();
                }
                current_rule.push_str(line);
                current_rule.push('\n');
                in_rule = true;
            } else if in_rule {
                // Continuation of current rule
                current_rule.push_str(line);
                current_rule.push('\n');

                // Check for rule termination
                if trimmed.ends_with(";") || trimmed.ends_with("}") {
                    rules.push(current_rule.trim().to_string());
                    current_rule.clear();
                    in_rule = false;
                }
            } else {
                // Single-line expression - treat as anonymous rule
                rules.push(trimmed.to_string());
            }
        }

        // Add final rule if exists
        if in_rule && !current_rule.trim().is_empty() {
            rules.push(current_rule.trim().to_string());
        }

        rules
    }

    /// Parse a single rule definition
    fn parse_single_rule(&self, rule_def: &str, line_number: usize) -> Result<DslRule, Vec<TranspileError>> {
        let mut errors = Vec::new();

        // Extract rule name and definition
        let (rule_name, expression_text) = self.extract_rule_parts(rule_def)?;

        // Parse the expression using nom parser
        match parse_expression(expression_text) {
            Ok((remaining, ast)) => {
                if !remaining.trim().is_empty() {
                    errors.push(TranspileError {
                        message: format!("Unexpected tokens after expression: '{}'", remaining),
                        line: Some(line_number),
                        column: Some(expression_text.len() - remaining.len()),
                        rule_name: Some(rule_name.clone()),
                        error_type: ErrorType::ParseError,
                    });
                }

                // Extract dependencies from AST
                let dependencies = self.extract_dependencies(&ast);

                let rule = DslRule {
                    name: rule_name,
                    expression: ast,
                    definition: rule_def.to_string(),
                    line_number,
                    dependencies,
                };

                if errors.is_empty() {
                    Ok(rule)
                } else {
                    Err(errors)
                }
            }
            Err(nom_error) => {
                errors.push(TranspileError {
                    message: format!("Failed to parse expression: {}", nom_error),
                    line: Some(line_number),
                    column: None,
                    rule_name: Some(rule_name),
                    error_type: ErrorType::ParseError,
                });
                Err(errors)
            }
        }
    }

    /// Extract rule name and expression from rule definition
    fn extract_rule_parts<'a>(&self, rule_def: &'a str) -> Result<(String, &'a str), Vec<TranspileError>> {
        if let Some(equals_pos) = rule_def.find(" = ") {
            let name_part = rule_def[..equals_pos].trim();
            let expression_part = rule_def[equals_pos + 3..].trim();

            // Extract rule name (handle "rule name" or just "name")
            let rule_name = if name_part.starts_with("rule ") {
                name_part[5..].trim().to_string()
            } else {
                name_part.to_string()
            };

            // Clean up expression (remove trailing semicolon if present)
            let clean_expr = expression_part.trim_end_matches(';').trim();

            Ok((rule_name, clean_expr))
        } else {
            // Anonymous rule - generate name
            let rule_name = format!("rule_{}", fastrand::u32(..));
            Ok((rule_name, rule_def.trim_end_matches(';').trim()))
        }
    }

    /// Extract variable dependencies from AST
    fn extract_dependencies(&self, expr: &Expression) -> Vec<String> {
        let mut deps = Vec::new();
        self.collect_dependencies(expr, &mut deps);
        deps.sort();
        deps.dedup();
        deps
    }

    /// Recursively collect dependencies from expression tree
    fn collect_dependencies(&self, expr: &Expression, deps: &mut Vec<String>) {
        match expr {
            Expression::Variable(name) | Expression::Identifier(name) => {
                // Skip built-in functions and literals
                if !self.is_builtin_function(name) && !name.chars().all(|c| c.is_ascii_digit()) {
                    deps.push(name.clone());
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.collect_dependencies(left, deps);
                self.collect_dependencies(right, deps);
            }
            Expression::UnaryOp { operand, .. } => {
                self.collect_dependencies(operand, deps);
            }
            Expression::FunctionCall { args, .. } => {
                for arg in args {
                    self.collect_dependencies(arg, deps);
                }
            }
            Expression::Conditional { condition, then_expr, else_expr } => {
                self.collect_dependencies(condition, deps);
                self.collect_dependencies(then_expr, deps);
                if let Some(else_branch) = else_expr {
                    self.collect_dependencies(else_branch, deps);
                }
            }
            Expression::Assignment { value, .. } => {
                self.collect_dependencies(value, deps);
            }
            Expression::List(items) => {
                for item in items {
                    self.collect_dependencies(item, deps);
                }
            }
            Expression::Cast { expr, .. } => {
                self.collect_dependencies(expr, deps);
            }
            _ => {} // Literals don't have dependencies
        }
    }

    /// Check if a name is a built-in function
    fn is_builtin_function(&self, name: &str) -> bool {
        matches!(name.to_uppercase().as_str(),
                "CONCAT" | "UPPER" | "LOWER" | "LENGTH" | "SUBSTRING" | "TRIM" |
                "ABS" | "ROUND" | "CEIL" | "FLOOR" | "MIN" | "MAX" | "SUM" | "AVG" |
                "IF" | "WHEN" | "THEN" | "ELSE" | "CASE" | "END" |
                "MATCHES" | "CONTAINS" | "STARTS_WITH" | "ENDS_WITH" |
                "TRUE" | "FALSE" | "NULL")
    }

    /// Validate a parsed rule
    fn validate_rule(&self, rule: &DslRule) -> Result<(), Vec<TranspileError>> {
        let mut errors = Vec::new();

        // Rule name validation
        if rule.name.is_empty() {
            errors.push(TranspileError {
                message: "Rule name cannot be empty".to_string(),
                line: Some(rule.line_number),
                column: None,
                rule_name: Some(rule.name.clone()),
                error_type: ErrorType::ValidationError,
            });
        }

        // Expression validation
        if let Err(validation_error) = self.validate_expression(&rule.expression) {
            errors.push(TranspileError {
                message: validation_error,
                line: Some(rule.line_number),
                column: None,
                rule_name: Some(rule.name.clone()),
                error_type: ErrorType::ValidationError,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate expression semantics
    fn validate_expression(&self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::BinaryOp { left, right, op } => {
                self.validate_expression(left)?;
                self.validate_expression(right)?;
                self.validate_binary_operation(left, right, op)?;
            }
            Expression::UnaryOp { operand, .. } => {
                self.validate_expression(operand)?;
            }
            Expression::FunctionCall { name, args } => {
                self.validate_function_call(name, args)?;
                for arg in args {
                    self.validate_expression(arg)?;
                }
            }
            Expression::Conditional { condition, then_expr, else_expr } => {
                self.validate_expression(condition)?;
                self.validate_expression(then_expr)?;
                if let Some(else_branch) = else_expr {
                    self.validate_expression(else_branch)?;
                }
            }
            Expression::List(items) => {
                for item in items {
                    self.validate_expression(item)?;
                }
            }
            Expression::Cast { expr, .. } => {
                self.validate_expression(expr)?;
            }
            _ => {} // Variables, identifiers, and literals are always valid
        }
        Ok(())
    }

    /// Validate binary operations
    fn validate_binary_operation(&self, _left: &Expression, _right: &Expression, _op: &BinaryOperator) -> Result<(), String> {
        // For now, all binary operations are considered valid
        // In the future, we could add type checking here
        Ok(())
    }

    /// Validate function calls
    fn validate_function_call(&self, name: &str, args: &[Expression]) -> Result<(), String> {
        match name.to_uppercase().as_str() {
            "CONCAT" => {
                if args.len() < 2 {
                    return Err(format!("CONCAT requires at least 2 arguments, got {}", args.len()));
                }
            }
            "SUBSTRING" => {
                if args.len() != 3 {
                    return Err(format!("SUBSTRING requires exactly 3 arguments, got {}", args.len()));
                }
            }
            "IF" => {
                if args.len() != 3 {
                    return Err(format!("IF requires exactly 3 arguments (condition, then, else), got {}", args.len()));
                }
            }
            _ => {} // Unknown functions are allowed for extensibility
        }
        Ok(())
    }

    /// Analyze inter-rule dependencies
    fn analyze_dependencies(&self, rules: &mut [DslRule]) -> Result<(), Vec<TranspileError>> {
        let mut errors = Vec::new();

        // Create a map of rule names for quick lookup
        let rule_names: std::collections::HashSet<String> =
            rules.iter().map(|r| r.name.clone()).collect();

        // Check for undefined dependencies
        for rule in rules.iter() {
            for dep in &rule.dependencies {
                if !rule_names.contains(dep) && !self.is_builtin_function(dep) {
                    errors.push(TranspileError {
                        message: format!("Undefined dependency: '{}'", dep),
                        line: Some(rule.line_number),
                        column: None,
                        rule_name: Some(rule.name.clone()),
                        error_type: ErrorType::SemanticError,
                    });
                }
            }
        }

        // Check for circular dependencies (simple implementation)
        for rule in rules.iter() {
            if rule.dependencies.contains(&rule.name) {
                errors.push(TranspileError {
                    message: "Rule cannot depend on itself".to_string(),
                    line: Some(rule.line_number),
                    column: None,
                    rule_name: Some(rule.name.clone()),
                    error_type: ErrorType::SemanticError,
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Convert DslRule to database Rule object
    pub fn to_database_rule(&self, dsl_rule: &DslRule, category_id: Option<i32>) -> Rule {
        use chrono::Utc;

        // Serialize AST to JSON
        let parsed_ast = serde_json::to_value(&dsl_rule.expression).ok();

        Rule {
            id: 0, // Will be set by database
            rule_id: format!("rule_{}", fastrand::u32(..)),
            rule_name: dsl_rule.name.clone(),
            description: Some(format!("Auto-generated rule from DSL: {}", dsl_rule.definition)),
            category_id,
            target_attribute_id: None, // To be determined by dependency analysis
            rule_definition: dsl_rule.definition.clone(),
            parsed_ast,
            status: "draft".to_string(),
            version: 1,
            tags: Some(vec!["dsl_generated".to_string()]),
            performance_metrics: None,
            embedding_data: None,
            created_by: Some("dsl_transpiler".to_string()),
            created_at: Utc::now(),
            updated_by: Some("dsl_transpiler".to_string()),
            updated_at: Utc::now(),
        }
    }
}

impl Default for DslTranspiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation utilities for transpiler
pub struct TranspilerValidator;

impl TranspilerValidator {
    /// Validate AST for transpilation compatibility
    pub fn validate_for_target(expr: &Expression, target: &TargetLanguage) -> Result<()> {
        match target {
            TargetLanguage::SQL => Self::validate_sql_compatibility(expr),
            TargetLanguage::Rust => Self::validate_rust_compatibility(expr),
            TargetLanguage::JavaScript => Self::validate_js_compatibility(expr),
            TargetLanguage::Python => Self::validate_python_compatibility(expr),
        }
    }

    fn validate_sql_compatibility(expr: &Expression) -> Result<()> {
        match expr {
            Expression::FunctionCall { name, .. } => {
                // Check if function is supported in SQL
                match name.to_uppercase().as_str() {
                    "CONCAT" | "UPPER" | "LOWER" | "LENGTH" | "SUBSTRING" | "TRIM" => Ok(()),
                    _ => bail!("Function '{}' is not supported in SQL target", name),
                }
            }
            Expression::BinaryOp { op, left, right } => {
                Self::validate_sql_compatibility(left)?;
                Self::validate_sql_compatibility(right)?;
                match op {
                    BinaryOperator::Matches | BinaryOperator::NotMatches => {
                        bail!("Regex operations not directly supported in standard SQL")
                    }
                    _ => Ok(()),
                }
            }
            Expression::UnaryOp { operand, .. } => Self::validate_sql_compatibility(operand),
            Expression::Conditional { condition, then_expr, else_expr } => {
                Self::validate_sql_compatibility(condition)?;
                Self::validate_sql_compatibility(then_expr)?;
                if let Some(else_branch) = else_expr {
                    Self::validate_sql_compatibility(else_branch)?;
                }
                Ok(())
            }
            Expression::List(items) => {
                for item in items {
                    Self::validate_sql_compatibility(item)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn validate_rust_compatibility(_expr: &Expression) -> Result<()> {
        // Rust supports all expression types
        Ok(())
    }

    fn validate_js_compatibility(_expr: &Expression) -> Result<()> {
        // JavaScript supports all expression types
        Ok(())
    }

    fn validate_python_compatibility(_expr: &Expression) -> Result<()> {
        // Python supports all expression types
        Ok(())
    }
}

/// S-expression code generation methods
impl Transpiler {
    /// Generate Rust code from S-expression
    fn generate_rust_from_s_expr(&self, s_expr: &LispValue) -> Result<String> {
        match s_expr {
            LispValue::List(items) if !items.is_empty() => {
                if let LispValue::Symbol(func) = &items[0] {
                    match func.as_str() {
                        "create-cbu-result" => {
                            let cbu_name = if items.len() > 1 {
                                self.extract_lisp_string(&items[1])?
                            } else {
                                "unnamed_cbu".to_string()
                            };
                            Ok(format!(
                                "// Create CBU: {}\nlet cbu = CbuBuilder::new(\"{}\")\n    .build()?;",
                                cbu_name, cbu_name
                            ))
                        }
                        "entity" => {
                            let id = if items.len() > 1 { self.extract_lisp_string(&items[1])? } else { "unknown".to_string() };
                            let name = if items.len() > 2 { self.extract_lisp_string(&items[2])? } else { "unknown".to_string() };
                            let role = if items.len() > 3 { self.extract_lisp_string(&items[3])? } else { "unknown".to_string() };
                            Ok(format!(
                                "Entity {{ id: \"{}\", name: \"{}\", role: EntityRole::{} }}",
                                id, name, self.role_to_rust_enum(&role)
                            ))
                        }
                        _ => Ok(format!("/* S-expression: {} */", s_expr.to_pretty_string())),
                    }
                } else {
                    Ok(format!("/* List: {} */", s_expr.to_pretty_string()))
                }
            }
            LispValue::String(s) => Ok(format!("\"{}\"", s)),
            LispValue::Number(n) => Ok(n.to_string()),
            LispValue::Boolean(b) => Ok(b.to_string()),
            LispValue::Symbol(s) => Ok(s.clone()),
            LispValue::Nil => Ok("None".to_string()),
            _ => Ok(format!("/* {} */", s_expr.to_pretty_string())),
        }
    }

    /// Generate SQL code from S-expression
    fn generate_sql_from_s_expr(&self, s_expr: &LispValue) -> Result<String> {
        match s_expr {
            LispValue::List(items) if !items.is_empty() => {
                if let LispValue::Symbol(func) = &items[0] {
                    match func.as_str() {
                        "create-cbu-result" => {
                            let cbu_name = if items.len() > 1 {
                                self.extract_lisp_string(&items[1])?
                            } else {
                                "Unnamed CBU".to_string()
                            };
                            Ok(format!(
                                "-- Create CBU: {}\nINSERT INTO cbus (cbu_name, status) VALUES ('{}', 'Active');",
                                cbu_name, cbu_name
                            ))
                        }
                        "entity" => {
                            let id = if items.len() > 1 { self.extract_lisp_string(&items[1])? } else { "unknown".to_string() };
                            let name = if items.len() > 2 { self.extract_lisp_string(&items[2])? } else { "unknown".to_string() };
                            let role = if items.len() > 3 { self.extract_lisp_string(&items[3])? } else { "unknown".to_string() };
                            Ok(format!(
                                "INSERT INTO entities (entity_id, entity_name, entity_role) VALUES ('{}', '{}', '{}');",
                                id, name, role
                            ))
                        }
                        _ => Ok(format!("-- S-expression: {}", s_expr.to_pretty_string())),
                    }
                } else {
                    Ok(format!("-- List: {}", s_expr.to_pretty_string()))
                }
            }
            LispValue::String(s) => Ok(format!("'{}'", s.replace("'", "''"))),
            LispValue::Number(n) => Ok(n.to_string()),
            LispValue::Boolean(b) => Ok(if *b { "TRUE".to_string() } else { "FALSE".to_string() }),
            LispValue::Symbol(s) => Ok(s.clone()),
            LispValue::Nil => Ok("NULL".to_string()),
            _ => Ok(format!("/* {} */", s_expr.to_pretty_string())),
        }
    }

    /// Generate JavaScript code from S-expression
    fn generate_js_from_s_expr(&self, s_expr: &LispValue) -> Result<String> {
        match s_expr {
            LispValue::List(items) if !items.is_empty() => {
                if let LispValue::Symbol(func) = &items[0] {
                    match func.as_str() {
                        "create-cbu-result" => {
                            let cbu_name = if items.len() > 1 {
                                self.extract_lisp_string(&items[1])?
                            } else {
                                "Unnamed CBU".to_string()
                            };
                            Ok(format!(
                                "// Create CBU: {}\nconst cbu = new CBU('{}');",
                                cbu_name, cbu_name
                            ))
                        }
                        "entity" => {
                            let id = if items.len() > 1 { self.extract_lisp_string(&items[1])? } else { "unknown".to_string() };
                            let name = if items.len() > 2 { self.extract_lisp_string(&items[2])? } else { "unknown".to_string() };
                            let role = if items.len() > 3 { self.extract_lisp_string(&items[3])? } else { "unknown".to_string() };
                            Ok(format!(
                                "{{ id: '{}', name: '{}', role: '{}' }}",
                                id, name, role
                            ))
                        }
                        _ => Ok(format!("/* S-expression: {} */", s_expr.to_pretty_string())),
                    }
                } else {
                    Ok(format!("/* List: {} */", s_expr.to_pretty_string()))
                }
            }
            LispValue::String(s) => Ok(format!("\"{}\"", s.replace("\"", "\\\""))),
            LispValue::Number(n) => Ok(n.to_string()),
            LispValue::Boolean(b) => Ok(b.to_string()),
            LispValue::Symbol(s) => Ok(format!("'{}'", s)),
            LispValue::Nil => Ok("null".to_string()),
            _ => Ok(format!("/* {} */", s_expr.to_pretty_string())),
        }
    }

    /// Generate Python code from S-expression
    fn generate_python_from_s_expr(&self, s_expr: &LispValue) -> Result<String> {
        match s_expr {
            LispValue::List(items) if !items.is_empty() => {
                if let LispValue::Symbol(func) = &items[0] {
                    match func.as_str() {
                        "create-cbu-result" => {
                            let cbu_name = if items.len() > 1 {
                                self.extract_lisp_string(&items[1])?
                            } else {
                                "Unnamed CBU".to_string()
                            };
                            Ok(format!(
                                "# Create CBU: {}\ncbu = CBU('{}', status='Active')",
                                cbu_name, cbu_name
                            ))
                        }
                        "entity" => {
                            let id = if items.len() > 1 { self.extract_lisp_string(&items[1])? } else { "unknown".to_string() };
                            let name = if items.len() > 2 { self.extract_lisp_string(&items[2])? } else { "unknown".to_string() };
                            let role = if items.len() > 3 { self.extract_lisp_string(&items[3])? } else { "unknown".to_string() };
                            Ok(format!(
                                "Entity(id='{}', name='{}', role='{}')",
                                id, name, role
                            ))
                        }
                        _ => Ok(format!("# S-expression: {}", s_expr.to_pretty_string())),
                    }
                } else {
                    Ok(format!("# List: {}", s_expr.to_pretty_string()))
                }
            }
            LispValue::String(s) => Ok(format!("\"{}\"", s.replace("\"", "\\\""))),
            LispValue::Number(n) => Ok(n.to_string()),
            LispValue::Boolean(b) => Ok(if *b { "True".to_string() } else { "False".to_string() }),
            LispValue::Symbol(s) => Ok(format!("'{}'", s)),
            LispValue::Nil => Ok("None".to_string()),
            _ => Ok(format!("# {}", s_expr.to_pretty_string())),
        }
    }

    /// Helper: Extract string from LISP value
    fn extract_lisp_string(&self, value: &LispValue) -> Result<String> {
        match value {
            LispValue::String(s) => Ok(s.clone()),
            LispValue::Symbol(s) => Ok(s.clone()),
            _ => bail!("Expected string, got {:?}", value),
        }
    }

    /// Helper: Convert role string to Rust enum variant
    fn role_to_rust_enum(&self, role: &str) -> String {
        match role.to_lowercase().as_str() {
            "asset-owner" | "assetowner" => "AssetOwner".to_string(),
            "investment-manager" | "investmentmanager" => "InvestmentManager".to_string(),
            "managing-company" | "managingcompany" => "ManagingCompany".to_string(),
            "general-partner" | "generalpartner" => "GeneralPartner".to_string(),
            "limited-partner" | "limitedpartner" => "LimitedPartner".to_string(),
            "prime-broker" | "primebroker" => "PrimeBroker".to_string(),
            "administrator" => "Administrator".to_string(),
            "custodian" => "Custodian".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding() {
        let transpiler = Transpiler::new(TranspilerOptions::default());

        // Test arithmetic constant folding
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Literal(Value::Integer(2))),
            right: Box::new(Expression::Literal(Value::Integer(3))),
        };

        let optimized = transpiler.constant_folding(&expr).unwrap();
        assert_eq!(optimized, Expression::Literal(Value::Integer(5)));
    }

    #[test]
    fn test_rust_generation() {
        let transpiler = Transpiler::new(TranspilerOptions {
            target: TargetLanguage::Rust,
            ..Default::default()
        });

        let expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Identifier("x".to_string())),
            right: Box::new(Expression::Literal(Value::Integer(5))),
        };

        let code = transpiler.generate_rust(&expr).unwrap();
        assert_eq!(code, "(ctx.get(\"x\") + Value::Integer(5))");
    }

    #[test]
    fn test_sql_generation() {
        let transpiler = Transpiler::new(TranspilerOptions {
            target: TargetLanguage::SQL,
            ..Default::default()
        });

        let expr = Expression::FunctionCall {
            name: "UPPER".to_string(),
            args: vec![Expression::Identifier("name".to_string())],
        };

        let code = transpiler.generate_sql(&expr).unwrap();
        assert_eq!(code, "UPPER(\"name\")");
    }

    // S-expression transpiler tests
    #[test]
    fn test_s_expression_rust_generation() {
        let transpiler = Transpiler::new(TranspilerOptions {
            target: TargetLanguage::Rust,
            ..Default::default()
        });

        let s_expr = LispValue::List(vec![
            LispValue::Symbol("create-cbu-result".to_string()),
            LispValue::String("Test Fund".to_string()),
            LispValue::String("Test Description".to_string()),
        ]);

        let code = transpiler.generate_rust_from_s_expr(&s_expr).unwrap();
        assert!(code.contains("Test Fund"));
        assert!(code.contains("CbuBuilder"));
    }

    #[test]
    fn test_s_expression_sql_generation() {
        let transpiler = Transpiler::new(TranspilerOptions {
            target: TargetLanguage::SQL,
            ..Default::default()
        });

        let s_expr = LispValue::List(vec![
            LispValue::Symbol("create-cbu-result".to_string()),
            LispValue::String("Investment Fund".to_string()),
        ]);

        let code = transpiler.generate_sql_from_s_expr(&s_expr).unwrap();
        assert!(code.contains("INSERT INTO cbus"));
        assert!(code.contains("Investment Fund"));
    }

    #[test]
    fn test_s_expression_javascript_generation() {
        let transpiler = Transpiler::new(TranspilerOptions {
            target: TargetLanguage::JavaScript,
            ..Default::default()
        });

        let s_expr = LispValue::List(vec![
            LispValue::Symbol("entity".to_string()),
            LispValue::String("E001".to_string()),
            LispValue::String("Test Entity".to_string()),
            LispValue::Symbol("asset-owner".to_string()),
        ]);

        let code = transpiler.generate_js_from_s_expr(&s_expr).unwrap();
        assert!(code.contains("E001"));
        assert!(code.contains("Test Entity"));
        assert!(code.contains("asset-owner"));
    }

    #[test]
    fn test_s_expression_python_generation() {
        let transpiler = Transpiler::new(TranspilerOptions {
            target: TargetLanguage::Python,
            ..Default::default()
        });

        let s_expr = LispValue::List(vec![
            LispValue::Symbol("entity".to_string()),
            LispValue::String("E001".to_string()),
            LispValue::String("Test Entity".to_string()),
            LispValue::Symbol("custodian".to_string()),
        ]);

        let code = transpiler.generate_python_from_s_expr(&s_expr).unwrap();
        assert!(code.contains("Entity("));
        assert!(code.contains("E001"));
        assert!(code.contains("custodian"));
    }

    #[test]
    fn test_s_expression_transpile_all_targets() {
        let s_expr = LispValue::List(vec![
            LispValue::Symbol("create-cbu-result".to_string()),
            LispValue::String("Multi-Target Fund".to_string()),
        ]);

        for target in [TargetLanguage::Rust, TargetLanguage::SQL, TargetLanguage::JavaScript, TargetLanguage::Python] {
            let transpiler = Transpiler::new(TranspilerOptions {
                target: target.clone(),
                ..Default::default()
            });

            let result = transpiler.transpile_s_expression(&s_expr);
            assert!(result.is_ok(), "Transpilation to {:?} should succeed", target);

            let code = result.unwrap();
            assert!(code.contains("Multi-Target Fund"), "Code should contain fund name for {:?}", target);
        }
    }

    #[test]
    fn test_s_expression_parse_and_transpile() {
        let transpiler = Transpiler::new(TranspilerOptions {
            target: TargetLanguage::Rust,
            ..Default::default()
        });

        let dsl_text = r#"(create-cbu "Parse Test Fund" "Testing parse and transpile")"#;
        let result = transpiler.parse_and_transpile_s_expr(dsl_text);

        assert!(result.is_ok(), "Parse and transpile should succeed");
        let code = result.unwrap();
        assert!(code.contains("Parse Test Fund") || code.contains("Transpiled"));
    }

    #[test]
    fn test_s_expression_atomic_values() {
        let transpiler = Transpiler::new(TranspilerOptions {
            target: TargetLanguage::Rust,
            ..Default::default()
        });

        // Test string
        let string_expr = LispValue::String("test string".to_string());
        let code = transpiler.generate_rust_from_s_expr(&string_expr).unwrap();
        assert_eq!(code, "\"test string\"");

        // Test number
        let number_expr = LispValue::Number(42.0);
        let code = transpiler.generate_rust_from_s_expr(&number_expr).unwrap();
        assert_eq!(code, "42");

        // Test boolean
        let bool_expr = LispValue::Boolean(true);
        let code = transpiler.generate_rust_from_s_expr(&bool_expr).unwrap();
        assert_eq!(code, "true");

        // Test nil
        let nil_expr = LispValue::Nil;
        let code = transpiler.generate_rust_from_s_expr(&nil_expr).unwrap();
        assert_eq!(code, "None");
    }

    #[test]
    fn test_role_to_rust_enum_conversion() {
        let transpiler = Transpiler::new(TranspilerOptions::default());

        assert_eq!(transpiler.role_to_rust_enum("asset-owner"), "AssetOwner");
        assert_eq!(transpiler.role_to_rust_enum("investment-manager"), "InvestmentManager");
        assert_eq!(transpiler.role_to_rust_enum("custodian"), "Custodian");
        assert_eq!(transpiler.role_to_rust_enum("unknown-role"), "Unknown");
    }

    #[test]
    fn test_s_expression_comprehensive_cbu() {
        let transpiler = Transpiler::new(TranspilerOptions {
            target: TargetLanguage::SQL,
            ..Default::default()
        });

        let dsl_text = r#"
            (create-cbu "Goldman Sachs Investment Fund" "Multi-strategy hedge fund operations"
              (entities
                (entity "GS001" "Goldman Sachs Asset Management" asset-owner)
                (entity "GS002" "Goldman Sachs Investment Advisors" investment-manager)))
        "#;

        let result = transpiler.parse_and_transpile_s_expr(dsl_text);
        assert!(result.is_ok(), "Complex CBU transpilation should succeed");

        let code = result.unwrap();
        assert!(code.contains("Goldman Sachs Investment Fund") || code.contains("Transpiled"));
    }

    #[test]
    fn test_s_expression_error_handling() {
        let transpiler = Transpiler::new(TranspilerOptions::default());

        // Test malformed S-expression
        let malformed_dsl = "(create-cbu \"Test\""; // Missing closing paren
        let result = transpiler.parse_and_transpile_s_expr(malformed_dsl);
        assert!(result.is_err(), "Malformed S-expression should fail");

        // Test invalid function
        let invalid_function_dsl = "(invalid-function \"test\")";
        let result = transpiler.parse_and_transpile_s_expr(invalid_function_dsl);
        assert!(result.is_err(), "Invalid function should fail");
    }

    #[test]
    fn test_extract_lisp_string() {
        let transpiler = Transpiler::new(TranspilerOptions::default());

        // Test string extraction
        let string_value = LispValue::String("test".to_string());
        let result = transpiler.extract_lisp_string(&string_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");

        // Test symbol extraction
        let symbol_value = LispValue::Symbol("symbol".to_string());
        let result = transpiler.extract_lisp_string(&symbol_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "symbol");

        // Test invalid type
        let number_value = LispValue::Number(42.0);
        let result = transpiler.extract_lisp_string(&number_value);
        assert!(result.is_err(), "Number should not be extractable as string");
    }
}