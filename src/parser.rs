use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1, take_while, escaped},
    character::complete::{char, digit1, multispace0, one_of},
    combinator::{map, opt, recognize, value},
    multi::{separated_list0, many0},
    sequence::{preceded, tuple, delimited, pair},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ASTNode {
    Assignment {
        target: String,
        value: Box<ASTNode>,
    },
    BinaryOp {
        left: Box<ASTNode>,
        op: BinaryOperator,
        right: Box<ASTNode>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<ASTNode>,
    },
    FunctionCall {
        name: String,
        args: Vec<ASTNode>,
    },
    Identifier(String),
    Number(f64),
    String(String),
    Boolean(bool),
    List(Vec<ASTNode>),
    Regex(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
    Concat,
    Matches,  // For regex matching: field MATCHES /pattern/
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Negate,
}

// Helper function to consume whitespace
fn ws<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(multispace0, inner, multispace0)
}

// Parse identifiers (variable names, function names)
fn identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((tag("_"), take_while1(|c: char| c.is_alphabetic()))),
            take_while(|c: char| c.is_alphanumeric() || c == '_')
        )),
        |s: &str| s.to_string()
    )(input)
}

// Parse numbers (integers and floats)
fn number(input: &str) -> IResult<&str, ASTNode> {
    alt((
        // Try parsing negative number directly first
        map(
            recognize(tuple((
                tag("-"),
                digit1,
                opt(tuple((tag("."), digit1)))
            ))),
            |s: &str| ASTNode::Number(s.parse::<f64>().unwrap())
        ),
        // Then try positive number
        map(
            recognize(tuple((
                digit1,
                opt(tuple((tag("."), digit1)))
            ))),
            |s: &str| ASTNode::Number(s.parse::<f64>().unwrap())
        )
    ))(input)
}

// Parse strings with escape sequences
fn string_literal(input: &str) -> IResult<&str, ASTNode> {
    map(
        alt((
            delimited(
                char('"'),
                escaped(
                    take_while(|c: char| c != '"' && c != '\\'),
                    '\\',
                    one_of("\"\\nrt")
                ),
                char('"')
            ),
            delimited(
                char('\''),
                escaped(
                    take_while(|c: char| c != '\'' && c != '\\'),
                    '\\',
                    one_of("'\\nrt")
                ),
                char('\'')
            )
        )),
        |s: &str| ASTNode::String(s.to_string())
    )(input)
}

// Parse regex literals: /pattern/flags or r"pattern"
fn regex_literal(input: &str) -> IResult<&str, ASTNode> {
    alt((
        // Slash syntax: /pattern/
        map(
            delimited(
                char('/'),
                take_while(|c: char| c != '/'),
                char('/')
            ),
            |pattern: &str| ASTNode::Regex(pattern.to_string())
        ),
        // Raw string syntax: r"pattern"
        map(
            preceded(
                char('r'),
                delimited(
                    char('"'),
                    take_while(|c: char| c != '"'),
                    char('"')
                )
            ),
            |pattern: &str| ASTNode::Regex(pattern.to_string())
        )
    ))(input)
}

// Parse boolean literals
fn boolean(input: &str) -> IResult<&str, ASTNode> {
    alt((
        value(ASTNode::Boolean(true), tag("true")),
        value(ASTNode::Boolean(false), tag("false"))
    ))(input)
}

// Parse function calls
fn function_call(input: &str) -> IResult<&str, ASTNode> {
    map(
        tuple((
            identifier,
            ws(char('(')),
            separated_list0(ws(char(',')), expression),
            ws(char(')'))
        )),
        |(name, _, args, _)| ASTNode::FunctionCall { name, args }
    )(input)
}

// Parse lists
fn list(input: &str) -> IResult<&str, ASTNode> {
    map(
        delimited(
            ws(char('[')),
            separated_list0(ws(char(',')), expression),
            ws(char(']'))
        ),
        ASTNode::List
    )(input)
}

// Parse primary expressions (atoms)
fn primary(input: &str) -> IResult<&str, ASTNode> {
    ws(alt((
        number,  // This now handles negative numbers directly
        string_literal,
        regex_literal,  // Add regex literal support
        boolean,
        function_call,
        list,
        map(identifier, ASTNode::Identifier),
        delimited(ws(char('(')), expression, ws(char(')')))
    )))(input)
}

// Parse power operations (highest precedence)
fn power(input: &str) -> IResult<&str, ASTNode> {
    let (input, left) = primary(input)?;

    let (input, rights) = many0(preceded(ws(tag("**")), primary))(input)?;

    Ok((input, rights.into_iter().fold(left, |acc, right| {
        ASTNode::BinaryOp {
            left: Box::new(acc),
            op: BinaryOperator::Power,
            right: Box::new(right),
        }
    })))
}

// Parse unary operations
fn unary(input: &str) -> IResult<&str, ASTNode> {
    alt((
        // Parse NOT operations
        map(
            preceded(ws(alt((tag("not"), tag("!")))), unary),
            |operand| ASTNode::UnaryOp {
                op: UnaryOperator::Not,
                operand: Box::new(operand),
            }
        ),
        // For unary minus, we need to be careful not to consume negative number literals
        // Let power handle primary expressions including negative numbers
        power
    ))(input)
}

// Parse multiplication and division
fn term(input: &str) -> IResult<&str, ASTNode> {
    let (input, left) = unary(input)?;

    let (input, operations) = many0(tuple((
        ws(alt((
            value(BinaryOperator::Multiply, char('*')),
            value(BinaryOperator::Divide, char('/')),
            value(BinaryOperator::Modulo, char('%'))
        ))),
        unary
    )))(input)?;

    Ok((input, operations.into_iter().fold(left, |acc, (op, right)| {
        ASTNode::BinaryOp {
            left: Box::new(acc),
            op,
            right: Box::new(right),
        }
    })))
}

// Parse addition and subtraction
fn arithmetic(input: &str) -> IResult<&str, ASTNode> {
    let (input, left) = term(input)?;

    let (input, operations) = many0(tuple((
        ws(alt((
            value(BinaryOperator::Add, char('+')),
            value(BinaryOperator::Subtract, char('-'))
        ))),
        term
    )))(input)?;

    Ok((input, operations.into_iter().fold(left, |acc, (op, right)| {
        ASTNode::BinaryOp {
            left: Box::new(acc),
            op,
            right: Box::new(right),
        }
    })))
}

// Parse string concatenation
fn concatenation(input: &str) -> IResult<&str, ASTNode> {
    let (input, left) = arithmetic(input)?;

    let (input, operations) = many0(tuple((
        ws(value(BinaryOperator::Concat, char('&'))),
        arithmetic
    )))(input)?;

    Ok((input, operations.into_iter().fold(left, |acc, (op, right)| {
        ASTNode::BinaryOp {
            left: Box::new(acc),
            op,
            right: Box::new(right),
        }
    })))
}

// Parse comparison operations
fn comparison(input: &str) -> IResult<&str, ASTNode> {
    let (input, left) = concatenation(input)?;

    let (input, operation) = opt(tuple((
        ws(alt((
            value(BinaryOperator::Matches, tag("MATCHES")),
            value(BinaryOperator::Matches, tag("~")),  // Shorthand for regex match
            value(BinaryOperator::LessThanOrEqual, tag("<=")),
            value(BinaryOperator::GreaterThanOrEqual, tag(">=")),
            value(BinaryOperator::NotEqual, tag("!=")),
            value(BinaryOperator::NotEqual, tag("<>")),
            value(BinaryOperator::Equal, tag("==")),
            value(BinaryOperator::Equal, tag("=")),
            value(BinaryOperator::LessThan, char('<')),
            value(BinaryOperator::GreaterThan, char('>'))
        ))),
        concatenation
    )))(input)?;

    Ok((input, match operation {
        Some((op, right)) => ASTNode::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        },
        None => left
    }))
}

// Parse logical AND
fn logical_and(input: &str) -> IResult<&str, ASTNode> {
    let (input, left) = comparison(input)?;

    let (input, operations) = many0(preceded(
        ws(alt((tag("and"), tag("&&")))),
        comparison
    ))(input)?;

    Ok((input, operations.into_iter().fold(left, |acc, right| {
        ASTNode::BinaryOp {
            left: Box::new(acc),
            op: BinaryOperator::And,
            right: Box::new(right),
        }
    })))
}

// Parse logical OR
fn logical_or(input: &str) -> IResult<&str, ASTNode> {
    let (input, left) = logical_and(input)?;

    let (input, operations) = many0(preceded(
        ws(alt((tag("or"), tag("||")))),
        logical_and
    ))(input)?;

    Ok((input, operations.into_iter().fold(left, |acc, right| {
        ASTNode::BinaryOp {
            left: Box::new(acc),
            op: BinaryOperator::Or,
            right: Box::new(right),
        }
    })))
}

// Parse expressions (without assignment)
pub fn expression(input: &str) -> IResult<&str, ASTNode> {
    logical_or(input)
}

// Parse assignment statements
fn assignment(input: &str) -> IResult<&str, ASTNode> {
    alt((
        map(
            tuple((
                identifier,
                ws(char('=')),
                expression
            )),
            |(target, _, value)| ASTNode::Assignment {
                target,
                value: Box::new(value),
            }
        ),
        expression
    ))(input)
}

// Main parser entry point
pub fn parse_rule(input: &str) -> IResult<&str, ASTNode> {
    ws(assignment)(input)
}

// Evaluate AST node with context
impl ASTNode {
    pub fn evaluate(&self, context: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
        match self {
            ASTNode::Number(n) => Ok(serde_json::json!(n)),
            ASTNode::String(s) => Ok(serde_json::json!(s)),
            ASTNode::Boolean(b) => Ok(serde_json::json!(b)),
            ASTNode::Regex(pattern) => Ok(serde_json::json!(format!("/{}/", pattern))),
            ASTNode::List(items) => {
                let evaluated: Result<Vec<_>, _> = items.iter()
                    .map(|item| item.evaluate(context))
                    .collect();
                Ok(serde_json::json!(evaluated?))
            },
            ASTNode::Identifier(name) => {
                context.get(name)
                    .cloned()
                    .ok_or_else(|| format!("Variable '{}' not found", name))
            },
            ASTNode::Assignment { target: _, value } => {
                // In a real implementation, this would modify the context
                value.evaluate(context)
            },
            ASTNode::BinaryOp { left, op, right } => {
                let left_val = left.evaluate(context)?;
                let right_val = right.evaluate(context)?;

                match op {
                    BinaryOperator::Add => {
                        if let (Some(l), Some(r)) = (left_val.as_f64(), right_val.as_f64()) {
                            Ok(serde_json::json!(l + r))
                        } else {
                            Err("Type error: addition requires numbers".to_string())
                        }
                    },
                    BinaryOperator::Subtract => {
                        if let (Some(l), Some(r)) = (left_val.as_f64(), right_val.as_f64()) {
                            Ok(serde_json::json!(l - r))
                        } else {
                            Err("Type error: subtraction requires numbers".to_string())
                        }
                    },
                    BinaryOperator::Multiply => {
                        if let (Some(l), Some(r)) = (left_val.as_f64(), right_val.as_f64()) {
                            Ok(serde_json::json!(l * r))
                        } else {
                            Err("Type error: multiplication requires numbers".to_string())
                        }
                    },
                    BinaryOperator::Divide => {
                        if let (Some(l), Some(r)) = (left_val.as_f64(), right_val.as_f64()) {
                            if r == 0.0 {
                                Err("Division by zero".to_string())
                            } else {
                                Ok(serde_json::json!(l / r))
                            }
                        } else {
                            Err("Type error: division requires numbers".to_string())
                        }
                    },
                    BinaryOperator::Modulo => {
                        if let (Some(l), Some(r)) = (left_val.as_f64(), right_val.as_f64()) {
                            Ok(serde_json::json!(l % r))
                        } else {
                            Err("Type error: modulo requires numbers".to_string())
                        }
                    },
                    BinaryOperator::Power => {
                        if let (Some(l), Some(r)) = (left_val.as_f64(), right_val.as_f64()) {
                            Ok(serde_json::json!(l.powf(r)))
                        } else {
                            Err("Type error: power requires numbers".to_string())
                        }
                    },
                    BinaryOperator::Concat => {
                        let l_str = if let Some(s) = left_val.as_str() {
                            s.to_string()
                        } else {
                            left_val.to_string()
                        };
                        let r_str = if let Some(s) = right_val.as_str() {
                            s.to_string()
                        } else {
                            right_val.to_string()
                        };
                        Ok(serde_json::json!(format!("{}{}", l_str, r_str)))
                    },
                    BinaryOperator::Equal => {
                        Ok(serde_json::json!(left_val == right_val))
                    },
                    BinaryOperator::NotEqual => {
                        Ok(serde_json::json!(left_val != right_val))
                    },
                    BinaryOperator::LessThan => {
                        if let (Some(l), Some(r)) = (left_val.as_f64(), right_val.as_f64()) {
                            Ok(serde_json::json!(l < r))
                        } else {
                            Err("Type error: comparison requires numbers".to_string())
                        }
                    },
                    BinaryOperator::LessThanOrEqual => {
                        if let (Some(l), Some(r)) = (left_val.as_f64(), right_val.as_f64()) {
                            Ok(serde_json::json!(l <= r))
                        } else {
                            Err("Type error: comparison requires numbers".to_string())
                        }
                    },
                    BinaryOperator::GreaterThan => {
                        if let (Some(l), Some(r)) = (left_val.as_f64(), right_val.as_f64()) {
                            Ok(serde_json::json!(l > r))
                        } else {
                            Err("Type error: comparison requires numbers".to_string())
                        }
                    },
                    BinaryOperator::GreaterThanOrEqual => {
                        if let (Some(l), Some(r)) = (left_val.as_f64(), right_val.as_f64()) {
                            Ok(serde_json::json!(l >= r))
                        } else {
                            Err("Type error: comparison requires numbers".to_string())
                        }
                    },
                    BinaryOperator::And => {
                        let l_bool = left_val.as_bool().unwrap_or(false);
                        let r_bool = right_val.as_bool().unwrap_or(false);
                        Ok(serde_json::json!(l_bool && r_bool))
                    },
                    BinaryOperator::Or => {
                        let l_bool = left_val.as_bool().unwrap_or(false);
                        let r_bool = right_val.as_bool().unwrap_or(false);
                        Ok(serde_json::json!(l_bool || r_bool))
                    },
                    BinaryOperator::Matches => {
                        // Handle regex matching: value MATCHES /pattern/ or value ~ r"pattern"
                        let text = match left_val.as_str() {
                            Some(s) => s,
                            None => return Err("Left operand of MATCHES must be a string".to_string())
                        };

                        // Extract pattern from right side
                        let pattern = if let Some(s) = right_val.as_str() {
                            // Remove surrounding slashes if present (from evaluation)
                            if s.starts_with('/') && s.ends_with('/') && s.len() > 2 {
                                &s[1..s.len()-1]
                            } else {
                                s
                            }
                        } else {
                            return Err("Right operand of MATCHES must be a regex pattern".to_string());
                        };

                        // Compile and test regex
                        match Regex::new(pattern) {
                            Ok(re) => Ok(serde_json::json!(re.is_match(text))),
                            Err(e) => Err(format!("Invalid regex pattern: {}", e))
                        }
                    },
                }
            },
            ASTNode::UnaryOp { op, operand } => {
                let val = operand.evaluate(context)?;
                match op {
                    UnaryOperator::Negate => {
                        if let Some(n) = val.as_f64() {
                            Ok(serde_json::json!(-n))
                        } else {
                            Err("Type error: negation requires a number".to_string())
                        }
                    },
                    UnaryOperator::Not => {
                        let bool_val = val.as_bool().unwrap_or(false);
                        Ok(serde_json::json!(!bool_val))
                    },
                }
            },
            ASTNode::FunctionCall { name, args } => {
                match name.to_uppercase().as_str() {
                    "CONCAT" => {
                        let mut result = String::new();
                        for arg in args {
                            let val = arg.evaluate(context)?;
                            if let Some(s) = val.as_str() {
                                result.push_str(s);
                            } else {
                                result.push_str(&val.to_string());
                            }
                        }
                        Ok(serde_json::json!(result))
                    },
                    "SUBSTRING" => {
                        if args.len() != 3 {
                            return Err("SUBSTRING requires 3 arguments".to_string());
                        }
                        let str_val = args[0].evaluate(context)?;
                        let start = args[1].evaluate(context)?;
                        let end = args[2].evaluate(context)?;

                        if let Some(s) = str_val.as_str() {
                            let start_idx = start.as_f64().unwrap_or(0.0) as usize;
                            let end_idx = end.as_f64().unwrap_or(s.len() as f64) as usize;

                            let substring = s.chars()
                                .skip(start_idx)
                                .take(end_idx.saturating_sub(start_idx))
                                .collect::<String>();
                            Ok(serde_json::json!(substring))
                        } else {
                            Err("SUBSTRING requires string argument".to_string())
                        }
                    },
                    "LOOKUP" => {
                        if args.len() != 2 {
                            return Err("LOOKUP requires 2 arguments".to_string());
                        }
                        let key = args[0].evaluate(context)?;
                        let table_name = args[1].evaluate(context)?;

                        // In a real implementation, this would look up in external tables
                        // For now, we'll simulate with some sample data
                        if let (Some(k), Some(t)) = (key.as_str(), table_name.as_str()) {
                            match t {
                                "countries" => {
                                    match k {
                                        "US" => Ok(serde_json::json!("United States")),
                                        "UK" => Ok(serde_json::json!("United Kingdom")),
                                        _ => Ok(serde_json::json!("Unknown"))
                                    }
                                },
                                "rates" => {
                                    match k {
                                        "premium" => Ok(serde_json::json!(0.15)),
                                        "standard" => Ok(serde_json::json!(0.10)),
                                        _ => Ok(serde_json::json!(0.05))
                                    }
                                },
                                _ => Err(format!("Unknown table: {}", t))
                            }
                        } else {
                            Err("LOOKUP requires string arguments".to_string())
                        }
                    },
                    "LEN" | "LENGTH" => {
                        if args.len() != 1 {
                            return Err(format!("{} requires 1 argument", name.to_uppercase()));
                        }
                        let val = args[0].evaluate(context)?;
                        if let Some(s) = val.as_str() {
                            Ok(serde_json::json!(s.len() as f64))
                        } else {
                            Err(format!("{} requires a string argument", name.to_uppercase()))
                        }
                    },
                    "UPPER" | "UPPERCASE" => {
                        if args.len() != 1 {
                            return Err(format!("{} requires 1 argument", name.to_uppercase()));
                        }
                        let val = args[0].evaluate(context)?;
                        if let Some(s) = val.as_str() {
                            Ok(serde_json::json!(s.to_uppercase()))
                        } else {
                            Err(format!("{} requires a string argument", name.to_uppercase()))
                        }
                    },
                    "LOWER" | "LOWERCASE" => {
                        if args.len() != 1 {
                            return Err(format!("{} requires 1 argument", name.to_uppercase()));
                        }
                        let val = args[0].evaluate(context)?;
                        if let Some(s) = val.as_str() {
                            Ok(serde_json::json!(s.to_lowercase()))
                        } else {
                            Err(format!("{} requires a string argument", name.to_uppercase()))
                        }
                    },
                    "IS_EMAIL" => {
                        // Validate email format for KYC
                        if args.len() != 1 {
                            return Err("IS_EMAIL requires 1 argument".to_string());
                        }
                        let val = args[0].evaluate(context)?;
                        if let Some(s) = val.as_str() {
                            let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
                            Ok(serde_json::json!(email_regex.is_match(s)))
                        } else {
                            Ok(serde_json::json!(false))
                        }
                    },
                    "IS_LEI" => {
                        // Validate Legal Entity Identifier (20 characters: 4 alphanumeric + 2 reserved + 14 alphanumeric)
                        if args.len() != 1 {
                            return Err("IS_LEI requires 1 argument".to_string());
                        }
                        let val = args[0].evaluate(context)?;
                        if let Some(s) = val.as_str() {
                            let lei_regex = Regex::new(r"^[A-Z0-9]{4}00[A-Z0-9]{14}$").unwrap();
                            Ok(serde_json::json!(lei_regex.is_match(s)))
                        } else {
                            Ok(serde_json::json!(false))
                        }
                    },
                    "IS_SWIFT" => {
                        // Validate SWIFT/BIC code (8 or 11 characters)
                        if args.len() != 1 {
                            return Err("IS_SWIFT requires 1 argument".to_string());
                        }
                        let val = args[0].evaluate(context)?;
                        if let Some(s) = val.as_str() {
                            let swift_regex = Regex::new(r"^[A-Z]{6}[A-Z0-9]{2}([A-Z0-9]{3})?$").unwrap();
                            Ok(serde_json::json!(swift_regex.is_match(s)))
                        } else {
                            Ok(serde_json::json!(false))
                        }
                    },
                    "IS_PHONE" => {
                        // Validate international phone number
                        if args.len() != 1 {
                            return Err("IS_PHONE requires 1 argument".to_string());
                        }
                        let val = args[0].evaluate(context)?;
                        if let Some(s) = val.as_str() {
                            let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
                            Ok(serde_json::json!(phone_regex.is_match(s)))
                        } else {
                            Ok(serde_json::json!(false))
                        }
                    },
                    "VALIDATE" => {
                        // Generic pattern validation: VALIDATE(value, pattern)
                        if args.len() != 2 {
                            return Err("VALIDATE requires 2 arguments: value and pattern".to_string());
                        }
                        let val = args[0].evaluate(context)?;
                        let pattern = args[1].evaluate(context)?;

                        if let (Some(s), Some(p)) = (val.as_str(), pattern.as_str()) {
                            match Regex::new(p) {
                                Ok(re) => Ok(serde_json::json!(re.is_match(s))),
                                Err(e) => Err(format!("Invalid regex pattern: {}", e))
                            }
                        } else {
                            Err("VALIDATE requires string arguments".to_string())
                        }
                    },
                    "EXTRACT" => {
                        // Extract pattern matches: EXTRACT(value, pattern)
                        if args.len() != 2 {
                            return Err("EXTRACT requires 2 arguments: value and pattern".to_string());
                        }
                        let val = args[0].evaluate(context)?;
                        let pattern = args[1].evaluate(context)?;

                        if let (Some(s), Some(p)) = (val.as_str(), pattern.as_str()) {
                            match Regex::new(p) {
                                Ok(re) => {
                                    if let Some(capture) = re.find(s) {
                                        Ok(serde_json::json!(capture.as_str()))
                                    } else {
                                        Ok(serde_json::json!(null))
                                    }
                                },
                                Err(e) => Err(format!("Invalid regex pattern: {}", e))
                            }
                        } else {
                            Err("EXTRACT requires string arguments".to_string())
                        }
                    },
                    _ => Err(format!("Unknown function: {}", name))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_numbers() {
        let (_, ast) = parse_rule("42").unwrap();
        assert_eq!(ast, ASTNode::Number(42.0));

        let (_, ast) = parse_rule("-3.14").unwrap();
        assert_eq!(ast, ASTNode::Number(-3.14));
    }

    #[test]
    fn test_parse_strings() {
        let (_, ast) = parse_rule("\"hello world\"").unwrap();
        assert_eq!(ast, ASTNode::String("hello world".to_string()));

        let (_, ast) = parse_rule("'single quotes'").unwrap();
        assert_eq!(ast, ASTNode::String("single quotes".to_string()));
    }

    #[test]
    fn test_parse_arithmetic() {
        let (_, ast) = parse_rule("2 + 3 * 4").unwrap();
        // Should parse as 2 + (3 * 4) due to precedence
        match ast {
            ASTNode::BinaryOp { op: BinaryOperator::Add, .. } => {},
            _ => panic!("Expected addition at top level")
        }
    }

    #[test]
    fn test_parse_assignment() {
        let (_, ast) = parse_rule("result = 10 + 20").unwrap();
        match ast {
            ASTNode::Assignment { target, .. } => {
                assert_eq!(target, "result");
            },
            _ => panic!("Expected assignment")
        }
    }

    #[test]
    fn test_parse_function_call() {
        let (_, ast) = parse_rule("CONCAT(\"hello\", \" \", \"world\")").unwrap();
        match ast {
            ASTNode::FunctionCall { name, args } => {
                assert_eq!(name, "CONCAT");
                assert_eq!(args.len(), 3);
            },
            _ => panic!("Expected function call")
        }
    }

    #[test]
    fn test_evaluation() {
        let context = HashMap::from([
            ("x".to_string(), serde_json::json!(10)),
            ("y".to_string(), serde_json::json!(20)),
        ]);

        let (_, ast) = parse_rule("x + y * 2").unwrap();
        let result = ast.evaluate(&context).unwrap();
        assert_eq!(result, serde_json::json!(50.0));
    }
}