use crate::models::{Expression, Value, BinaryOperator, UnaryOperator};
use anyhow::{Result, bail};

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
}