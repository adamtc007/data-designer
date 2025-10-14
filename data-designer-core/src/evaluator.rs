use crate::models::{Expression, Value, BinaryOperator, UnaryOperator};
use anyhow::{Result, bail};
use std::collections::HashMap;
use regex::Regex;

pub type Facts = HashMap<String, Value>;

/// Comprehensive function library for DSL evaluation
pub struct FunctionLibrary {
    pub lookup_tables: HashMap<String, HashMap<String, String>>,
}

impl FunctionLibrary {
    pub fn new() -> Self {
        Self {
            lookup_tables: HashMap::new(),
        }
    }

    pub fn add_lookup_table(&mut self, name: String, table: HashMap<String, String>) {
        self.lookup_tables.insert(name, table);
    }

    pub fn call_function(&self, name: &str, args: &[Value]) -> Result<Value> {
        match name.to_uppercase().as_str() {
            "CONCAT" => self.concat(args),
            "SUBSTRING" => self.substring(args),
            "UPPER" => self.upper(args),
            "LOWER" => self.lower(args),
            "LENGTH" => self.length(args),
            "TRIM" => self.trim(args),
            "LOOKUP" => self.lookup(args),
            "ABS" => self.abs(args),
            "ROUND" => self.round(args),
            "FLOOR" => self.floor(args),
            "CEIL" => self.ceil(args),
            "MIN" => self.min(args),
            "MAX" => self.max(args),
            "SUM" => self.sum(args),
            "AVG" => self.avg(args),
            "COUNT" => self.count(args),
            "HAS" => self.has(args),
            "IS_NULL" => self.is_null(args),
            "IS_EMPTY" => self.is_empty(args),
            "TO_STRING" => self.to_string(args),
            "TO_NUMBER" => self.to_number(args),
            "TO_BOOLEAN" => self.to_boolean(args),
            "FIRST" => self.first(args),
            "LAST" => self.last(args),
            "GET" => self.get(args),
            _ => bail!("Unknown function '{}'", name),
        }
    }

    // String functions
    fn concat(&self, args: &[Value]) -> Result<Value> {
        let result = args
            .iter()
            .map(|v| value_to_string(v))
            .collect::<Vec<_>>()
            .join("");
        Ok(Value::String(result))
    }

    fn substring(&self, args: &[Value]) -> Result<Value> {
        if args.len() < 2 || args.len() > 3 {
            bail!("SUBSTRING requires 2 or 3 arguments");
        }

        let text = value_to_string(&args[0]);
        let start = match &args[1] {
            Value::Integer(i) => *i as usize,
            _ => bail!("SUBSTRING start position must be an integer"),
        };

        let result = if args.len() == 3 {
            let length = match &args[2] {
                Value::Integer(i) => *i as usize,
                _ => bail!("SUBSTRING length must be an integer"),
            };
            text.chars().skip(start).take(length).collect()
        } else {
            text.chars().skip(start).collect()
        };

        Ok(Value::String(result))
    }

    fn upper(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("UPPER requires exactly 1 argument");
        }
        Ok(Value::String(value_to_string(&args[0]).to_uppercase()))
    }

    fn lower(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("LOWER requires exactly 1 argument");
        }
        Ok(Value::String(value_to_string(&args[0]).to_lowercase()))
    }

    fn length(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("LENGTH requires exactly 1 argument");
        }
        let len = match &args[0] {
            Value::String(s) => s.len(),
            Value::List(l) => l.len(),
            _ => value_to_string(&args[0]).len(),
        };
        Ok(Value::Integer(len as i64))
    }

    fn trim(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("TRIM requires exactly 1 argument");
        }
        Ok(Value::String(value_to_string(&args[0]).trim().to_string()))
    }

    // Lookup function
    fn lookup(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            bail!("LOOKUP requires exactly 2 arguments");
        }

        let key = value_to_string(&args[0]);
        let table_name = value_to_string(&args[1]);

        if let Some(table) = self.lookup_tables.get(&table_name) {
            if let Some(value) = table.get(&key) {
                Ok(Value::String(value.clone()))
            } else {
                Ok(Value::Null)
            }
        } else {
            bail!("Lookup table '{}' not found", table_name);
        }
    }

    // Math functions
    fn abs(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("ABS requires exactly 1 argument");
        }
        match &args[0] {
            Value::Integer(i) => Ok(Value::Integer(i.abs())),
            Value::Float(f) => Ok(Value::Float(f.abs())),
            _ => bail!("ABS requires a numeric argument"),
        }
    }

    fn round(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("ROUND requires exactly 1 argument");
        }
        match &args[0] {
            Value::Float(f) => Ok(Value::Integer(f.round() as i64)),
            Value::Integer(i) => Ok(Value::Integer(*i)),
            _ => bail!("ROUND requires a numeric argument"),
        }
    }

    fn floor(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("FLOOR requires exactly 1 argument");
        }
        match &args[0] {
            Value::Float(f) => Ok(Value::Integer(f.floor() as i64)),
            Value::Integer(i) => Ok(Value::Integer(*i)),
            _ => bail!("FLOOR requires a numeric argument"),
        }
    }

    fn ceil(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("CEIL requires exactly 1 argument");
        }
        match &args[0] {
            Value::Float(f) => Ok(Value::Integer(f.ceil() as i64)),
            Value::Integer(i) => Ok(Value::Integer(*i)),
            _ => bail!("CEIL requires a numeric argument"),
        }
    }

    // Aggregate functions
    fn min(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            bail!("MIN requires at least 1 argument");
        }

        let mut min_val = &args[0];
        for val in &args[1..] {
            if compare_values(val, min_val)? < 0 {
                min_val = val;
            }
        }
        Ok(min_val.clone())
    }

    fn max(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            bail!("MAX requires at least 1 argument");
        }

        let mut max_val = &args[0];
        for val in &args[1..] {
            if compare_values(val, max_val)? > 0 {
                max_val = val;
            }
        }
        Ok(max_val.clone())
    }

    fn sum(&self, args: &[Value]) -> Result<Value> {
        let mut sum = 0.0;
        let mut is_float = false;

        for arg in args {
            match arg {
                Value::Integer(i) => sum += *i as f64,
                Value::Float(f) => {
                    sum += f;
                    is_float = true;
                }
                Value::List(list) => {
                    for item in list {
                        match item {
                            Value::Integer(i) => sum += *i as f64,
                            Value::Float(f) => {
                                sum += f;
                                is_float = true;
                            }
                            _ => bail!("SUM requires numeric values"),
                        }
                    }
                }
                _ => bail!("SUM requires numeric values"),
            }
        }

        if is_float {
            Ok(Value::Float(sum))
        } else {
            Ok(Value::Integer(sum as i64))
        }
    }

    fn avg(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            bail!("AVG requires at least 1 argument");
        }

        let sum_result = self.sum(args)?;
        let count = args.len() as f64;

        match sum_result {
            Value::Integer(i) => Ok(Value::Float(i as f64 / count)),
            Value::Float(f) => Ok(Value::Float(f / count)),
            _ => bail!("AVG calculation failed"),
        }
    }

    fn count(&self, args: &[Value]) -> Result<Value> {
        let mut count = 0;
        for arg in args {
            match arg {
                Value::List(list) => count += list.len(),
                Value::Null => {} // Don't count nulls
                _ => count += 1,
            }
        }
        Ok(Value::Integer(count as i64))
    }

    // Utility functions
    fn has(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("HAS requires exactly 1 argument");
        }
        Ok(Value::Boolean(!matches!(args[0], Value::Null)))
    }

    fn is_null(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("IS_NULL requires exactly 1 argument");
        }
        Ok(Value::Boolean(matches!(args[0], Value::Null)))
    }

    fn is_empty(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("IS_EMPTY requires exactly 1 argument");
        }
        let empty = match &args[0] {
            Value::String(s) => s.is_empty(),
            Value::List(l) => l.is_empty(),
            Value::Null => true,
            _ => false,
        };
        Ok(Value::Boolean(empty))
    }

    // Type conversion functions
    fn to_string(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("TO_STRING requires exactly 1 argument");
        }
        Ok(Value::String(value_to_string(&args[0])))
    }

    fn to_number(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("TO_NUMBER requires exactly 1 argument");
        }
        match &args[0] {
            Value::String(s) => {
                if let Ok(i) = s.parse::<i64>() {
                    Ok(Value::Integer(i))
                } else if let Ok(f) = s.parse::<f64>() {
                    Ok(Value::Float(f))
                } else {
                    bail!("Cannot convert '{}' to number", s);
                }
            }
            Value::Boolean(b) => Ok(Value::Integer(if *b { 1 } else { 0 })),
            v => Ok(v.clone()),
        }
    }

    fn to_boolean(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("TO_BOOLEAN requires exactly 1 argument");
        }
        let bool_val = match &args[0] {
            Value::Boolean(b) => *b,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty() && s.to_lowercase() != "false",
            Value::Null => false,
            Value::List(l) => !l.is_empty(),
            Value::Regex(_) => true,
        };
        Ok(Value::Boolean(bool_val))
    }

    // List access functions
    fn first(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("FIRST requires exactly 1 argument");
        }
        match &args[0] {
            Value::List(list) => {
                if list.is_empty() {
                    Ok(Value::Null)
                } else {
                    Ok(list[0].clone())
                }
            }
            _ => bail!("FIRST requires a list argument"),
        }
    }

    fn last(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            bail!("LAST requires exactly 1 argument");
        }
        match &args[0] {
            Value::List(list) => {
                if list.is_empty() {
                    Ok(Value::Null)
                } else {
                    Ok(list[list.len() - 1].clone())
                }
            }
            _ => bail!("LAST requires a list argument"),
        }
    }

    fn get(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            bail!("GET requires exactly 2 arguments");
        }
        match (&args[0], &args[1]) {
            (Value::List(list), Value::Integer(index)) => {
                let idx = *index as usize;
                if idx < list.len() {
                    Ok(list[idx].clone())
                } else {
                    Ok(Value::Null)
                }
            }
            _ => bail!("GET requires a list and an integer index"),
        }
    }
}

/// Evaluates a parsed AST `Expression` against a set of facts.
pub fn evaluate(expr: &Expression, facts: &Facts) -> Result<Value> {
    evaluate_with_functions(expr, facts, &FunctionLibrary::new())
}

/// Evaluates a parsed AST `Expression` with a function library.
pub fn evaluate_with_functions(expr: &Expression, facts: &Facts, functions: &FunctionLibrary) -> Result<Value> {
    match expr {
        Expression::Literal(val) => Ok(val.clone()),

        Expression::Identifier(name) => {
            Ok(facts.get(name)
                 .cloned()
                 .unwrap_or(Value::Null))  // Return null instead of error for missing facts
        }

        Expression::Assignment { target: _, value } => {
            let result = evaluate_with_functions(value, facts, functions)?;
            // Note: In a real system, you'd update the facts here
            // For now, just return the computed value
            Ok(result)
        }

        Expression::BinaryOp { op, left, right } => {
            let left_val = evaluate_with_functions(left, facts, functions)?;
            let right_val = evaluate_with_functions(right, facts, functions)?;
            evaluate_binary_op(*op, &left_val, &right_val)
        }

        Expression::UnaryOp { op, operand } => {
            let operand_val = evaluate_with_functions(operand, facts, functions)?;
            evaluate_unary_op(*op, &operand_val)
        }

        Expression::FunctionCall { name, args } => {
            let mut arg_values = Vec::new();
            for arg_expr in args {
                arg_values.push(evaluate_with_functions(arg_expr, facts, functions)?);
            }
            functions.call_function(name, &arg_values)
        }

        Expression::Cast { expr, data_type } => {
            let value = evaluate_with_functions(expr, facts, functions)?;
            cast_value(value, data_type)
        }

        Expression::List(exprs) => {
            let mut values = Vec::new();
            for expr in exprs {
                values.push(evaluate_with_functions(expr, facts, functions)?);
            }
            Ok(Value::List(values))
        }

        Expression::Conditional { condition, then_expr, else_expr } => {
            let condition_val = evaluate_with_functions(condition, facts, functions)?;
            let condition_bool = match condition_val {
                Value::Boolean(b) => b,
                Value::Null => false,
                Value::Integer(i) => i != 0,
                Value::Float(f) => f != 0.0,
                Value::String(s) => !s.is_empty(),
                Value::List(l) => !l.is_empty(),
                Value::Regex(_) => true,
            };

            if condition_bool {
                evaluate_with_functions(then_expr, facts, functions)
            } else if let Some(else_expr) = else_expr {
                evaluate_with_functions(else_expr, facts, functions)
            } else {
                Ok(Value::Null)
            }
        }
    }
}

fn evaluate_binary_op(op: BinaryOperator, left: &Value, right: &Value) -> Result<Value> {
    match op {
        // Arithmetic operators
        BinaryOperator::Add => arithmetic_add(left, right),
        BinaryOperator::Subtract => arithmetic_subtract(left, right),
        BinaryOperator::Multiply => arithmetic_multiply(left, right),
        BinaryOperator::Divide => arithmetic_divide(left, right),
        BinaryOperator::Modulo => arithmetic_modulo(left, right),
        BinaryOperator::Power => arithmetic_power(left, right),

        // String operations
        BinaryOperator::Concat => Ok(Value::String(format!("{}{}", value_to_string(left), value_to_string(right)))),

        // Comparison operators
        BinaryOperator::Equals => Ok(Value::Boolean(values_equal(left, right))),
        BinaryOperator::NotEquals => Ok(Value::Boolean(!values_equal(left, right))),
        BinaryOperator::LessThan => Ok(Value::Boolean(compare_values(left, right)? < 0)),
        BinaryOperator::LessThanOrEqual => Ok(Value::Boolean(compare_values(left, right)? <= 0)),
        BinaryOperator::GreaterThan => Ok(Value::Boolean(compare_values(left, right)? > 0)),
        BinaryOperator::GreaterThanOrEqual => Ok(Value::Boolean(compare_values(left, right)? >= 0)),

        // Pattern matching
        BinaryOperator::Matches => pattern_matches(left, right),
        BinaryOperator::NotMatches => Ok(Value::Boolean(!to_bool(&pattern_matches(left, right)?))),
        BinaryOperator::Contains => Ok(Value::Boolean(value_to_string(left).contains(&value_to_string(right)))),
        BinaryOperator::StartsWith => Ok(Value::Boolean(value_to_string(left).starts_with(&value_to_string(right)))),
        BinaryOperator::EndsWith => Ok(Value::Boolean(value_to_string(left).ends_with(&value_to_string(right)))),

        // Logical operators
        BinaryOperator::And => Ok(Value::Boolean(to_bool(left) && to_bool(right))),
        BinaryOperator::Or => Ok(Value::Boolean(to_bool(left) || to_bool(right))),

        // Set operations
        BinaryOperator::In => value_in_list(left, right),
        BinaryOperator::NotIn => Ok(Value::Boolean(!to_bool(&value_in_list(left, right)?))),
    }
}

fn evaluate_unary_op(op: UnaryOperator, operand: &Value) -> Result<Value> {
    match op {
        UnaryOperator::Not => Ok(Value::Boolean(!to_bool(operand))),
        UnaryOperator::Minus => match operand {
            Value::Integer(i) => Ok(Value::Integer(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => bail!("Cannot apply unary minus to {:?}", operand),
        },
        UnaryOperator::Plus => match operand {
            Value::Integer(_) | Value::Float(_) => Ok(operand.clone()),
            _ => bail!("Cannot apply unary plus to {:?}", operand),
        },
    }
}

// Helper functions for arithmetic operations
fn arithmetic_add(left: &Value, right: &Value) -> Result<Value> {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
        (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(*l as f64 + r)),
        (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l + *r as f64)),
        _ => bail!("Cannot add {:?} and {:?}", left, right),
    }
}

fn arithmetic_subtract(left: &Value, right: &Value) -> Result<Value> {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
        (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(*l as f64 - r)),
        (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l - *r as f64)),
        _ => bail!("Cannot subtract {:?} and {:?}", left, right),
    }
}

fn arithmetic_multiply(left: &Value, right: &Value) -> Result<Value> {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
        (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(*l as f64 * r)),
        (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l * *r as f64)),
        _ => bail!("Cannot multiply {:?} and {:?}", left, right),
    }
}

fn arithmetic_divide(left: &Value, right: &Value) -> Result<Value> {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => {
            if *r == 0 { bail!("Division by zero"); }
            Ok(Value::Float(*l as f64 / *r as f64))
        },
        (Value::Float(l), Value::Float(r)) => {
            if *r == 0.0 { bail!("Division by zero"); }
            Ok(Value::Float(l / r))
        },
        (Value::Integer(l), Value::Float(r)) => {
            if *r == 0.0 { bail!("Division by zero"); }
            Ok(Value::Float(*l as f64 / r))
        },
        (Value::Float(l), Value::Integer(r)) => {
            if *r == 0 { bail!("Division by zero"); }
            Ok(Value::Float(l / *r as f64))
        },
        _ => bail!("Cannot divide {:?} and {:?}", left, right),
    }
}

fn arithmetic_modulo(left: &Value, right: &Value) -> Result<Value> {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => {
            if *r == 0 { bail!("Modulo by zero"); }
            Ok(Value::Integer(l % r))
        },
        _ => bail!("Modulo operation requires integers"),
    }
}

fn arithmetic_power(left: &Value, right: &Value) -> Result<Value> {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => {
            if *r < 0 {
                Ok(Value::Float((*l as f64).powf(*r as f64)))
            } else {
                Ok(Value::Integer(l.pow(*r as u32)))
            }
        },
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l.powf(*r))),
        (Value::Integer(l), Value::Float(r)) => Ok(Value::Float((*l as f64).powf(*r))),
        (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l.powf(*r as f64))),
        _ => bail!("Cannot raise {:?} to power {:?}", left, right),
    }
}

// Helper functions
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::List(list) => {
            let items: Vec<String> = list.iter().map(value_to_string).collect();
            format!("[{}]", items.join(", "))
        },
        Value::Regex(pattern) => format!("/{}/", pattern),
    }
}

fn to_bool(value: &Value) -> bool {
    match value {
        Value::Boolean(b) => *b,
        Value::Integer(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Null => false,
        Value::List(l) => !l.is_empty(),
        Value::Regex(_) => true,
    }
}

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => l == r,
        (Value::Float(l), Value::Float(r)) => (l - r).abs() < f64::EPSILON,
        (Value::Integer(l), Value::Float(r)) => (*l as f64 - r).abs() < f64::EPSILON,
        (Value::Float(l), Value::Integer(r)) => (l - *r as f64).abs() < f64::EPSILON,
        (Value::String(l), Value::String(r)) => l == r,
        (Value::Boolean(l), Value::Boolean(r)) => l == r,
        (Value::Null, Value::Null) => true,
        (Value::List(l), Value::List(r)) => l == r,
        (Value::Regex(l), Value::Regex(r)) => l == r,
        _ => false,
    }
}

fn compare_values(left: &Value, right: &Value) -> Result<i32> {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => Ok(l.cmp(r) as i32),
        (Value::Float(l), Value::Float(r)) => Ok(l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal) as i32),
        (Value::Integer(l), Value::Float(r)) => Ok((*l as f64).partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal) as i32),
        (Value::Float(l), Value::Integer(r)) => Ok(l.partial_cmp(&(*r as f64)).unwrap_or(std::cmp::Ordering::Equal) as i32),
        (Value::String(l), Value::String(r)) => Ok(l.cmp(r) as i32),
        _ => bail!("Cannot compare {:?} and {:?}", left, right),
    }
}

fn pattern_matches(text: &Value, pattern: &Value) -> Result<Value> {
    let text_str = value_to_string(text);
    let pattern_str = match pattern {
        Value::Regex(pattern) => pattern.clone(),
        _ => value_to_string(pattern),
    };

    match Regex::new(&pattern_str) {
        Ok(regex) => Ok(Value::Boolean(regex.is_match(&text_str))),
        Err(_) => bail!("Invalid regex pattern: {}", pattern_str),
    }
}

fn value_in_list(value: &Value, list: &Value) -> Result<Value> {
    match list {
        Value::List(items) => {
            for item in items {
                if values_equal(value, item) {
                    return Ok(Value::Boolean(true));
                }
            }
            Ok(Value::Boolean(false))
        },
        _ => bail!("IN operator requires a list on the right side"),
    }
}

fn cast_value(value: Value, data_type: &str) -> Result<Value> {
    match data_type.to_uppercase().as_str() {
        "STRING" => Ok(Value::String(value_to_string(&value))),
        "INTEGER" => match value {
            Value::Integer(i) => Ok(Value::Integer(i)),
            Value::Float(f) => Ok(Value::Integer(f as i64)),
            Value::String(s) => s.parse::<i64>().map(Value::Integer).map_err(|_| anyhow::anyhow!("Cannot cast '{}' to integer", s)),
            Value::Boolean(b) => Ok(Value::Integer(if b { 1 } else { 0 })),
            _ => bail!("Cannot cast {:?} to integer", value),
        },
        "FLOAT" => match value {
            Value::Float(f) => Ok(Value::Float(f)),
            Value::Integer(i) => Ok(Value::Float(i as f64)),
            Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| anyhow::anyhow!("Cannot cast '{}' to float", s)),
            _ => bail!("Cannot cast {:?} to float", value),
        },
        "BOOLEAN" => Ok(Value::Boolean(to_bool(&value))),
        _ => bail!("Unknown data type: {}", data_type),
    }
}