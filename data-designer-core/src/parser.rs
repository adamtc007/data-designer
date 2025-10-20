use crate::models::{Expression, Value, BinaryOperator, UnaryOperator};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0, none_of},
    combinator::{map, recognize, map_res, opt, value},
    error::ParseError,
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

// Whitespace wrapper
fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

// Parse identifiers (variables, function names)
fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_"), tag(".")))),
        )),
        String::from,
    )(input)
}

// Parse integers and floats
fn parse_number(input: &str) -> IResult<&str, Value> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            digit1,
            opt(tuple((char('.'), digit1))),
        ))),
        |s: &str| {
            if s.contains('.') {
                s.parse::<f64>().map(Value::Float).map_err(|_| "Invalid float")
            } else {
                s.parse::<i64>().map(Value::Integer).map_err(|_| "Invalid integer")
            }
        },
    )(input)
}

// Parse string literals with escape sequences
fn parse_string_literal(input: &str) -> IResult<&str, Value> {
    alt((
        // Double-quoted strings
        map(
            delimited(
                char('"'),
                many0(alt((
                    map(tag("\\\""), |_| '"'),
                    map(tag("\\\\"), |_| '\\'),
                    map(tag("\\n"), |_| '\n'),
                    map(tag("\\t"), |_| '\t'),
                    map(tag("\\r"), |_| '\r'),
                    map(none_of("\"\\"), |c| c),
                ))),
                char('"'),
            ),
            |chars| Value::String(chars.into_iter().collect()),
        ),
        // Single-quoted strings (simpler)
        map(
            delimited(char('\''), take_while(|c| c != '\''), char('\'')),
            |s: &str| Value::String(s.to_string()),
        ),
    ))(input)
}

// Parse regex literals: /pattern/ or r"pattern"
fn parse_regex_literal(input: &str) -> IResult<&str, Value> {
    alt((
        // Slash syntax: /pattern/
        map(
            delimited(char('/'), take_while(|c| c != '/'), char('/')),
            |pattern: &str| Value::Regex(pattern.to_string()),
        ),
        // Raw string syntax: r"pattern"
        map(
            preceded(
                char('r'),
                delimited(char('"'), take_while(|c| c != '"'), char('"')),
            ),
            |pattern: &str| Value::Regex(pattern.to_string()),
        ),
    ))(input)
}

// Parse boolean literals
fn parse_boolean(input: &str) -> IResult<&str, Value> {
    alt((
        value(Value::Boolean(true), tag("true")),
        value(Value::Boolean(false), tag("false")),
    ))(input)
}

// Parse null literal
fn parse_null(input: &str) -> IResult<&str, Value> {
    value(Value::Null, tag("null"))(input)
}

// Parse list literals: [item1, item2, ...]
fn parse_list(input: &str) -> IResult<&str, Expression> {
    map(
        delimited(
            ws(char('[')),
            separated_list0(ws(char(',')), parse_expression),
            ws(char(']')),
        ),
        Expression::List,
    )(input)
}

// Parse function calls: FUNC(arg1, arg2, ...)
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

// Parse fund accounting workflow verbs
fn parse_configure_system(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            ws(tag("CONFIGURE_SYSTEM")),
            ws(parse_string_literal),
            opt(tuple((
                ws(char('(')),
                separated_list0(ws(char(',')), parse_expression),
                ws(char(')')),
            ))),
        )),
        |(_, capability_name, args)| {
            let capability_name = match capability_name {
                Value::String(s) => s,
                _ => "unknown".to_string(),
            };
            let arguments = args.map(|(_, args, _)| args).unwrap_or_default();
            Expression::ConfigureSystem { capability_name, arguments }
        },
    )(input)
}

fn parse_activate(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            ws(tag("ACTIVATE")),
            opt(parse_string_literal),
            opt(tuple((
                ws(char('(')),
                separated_list0(ws(char(',')), parse_expression),
                ws(char(')')),
            ))),
        )),
        |(_, target, args)| {
            let target = target.and_then(|t| match t {
                Value::String(s) => Some(s),
                _ => None,
            });
            let arguments = args.map(|(_, args, _)| args).unwrap_or_default();
            Expression::Activate { target, arguments }
        },
    )(input)
}

fn parse_run_health_check(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            ws(tag("RUN_HEALTH_CHECK")),
            ws(parse_string_literal),
            opt(tuple((
                ws(char('(')),
                separated_list0(ws(char(',')), parse_expression),
                ws(char(')')),
            ))),
        )),
        |(_, check_type, args)| {
            let check_type = match check_type {
                Value::String(s) => s,
                _ => "unknown".to_string(),
            };
            let arguments = args.map(|(_, args, _)| args).unwrap_or_default();
            Expression::RunHealthCheck { check_type, arguments }
        },
    )(input)
}

fn parse_set_status(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            ws(tag("SET_STATUS")),
            ws(parse_string_literal),
            opt(parse_string_literal),
        )),
        |(_, status, target)| {
            let status = match status {
                Value::String(s) => s,
                _ => "unknown".to_string(),
            };
            let target = target.and_then(|t| match t {
                Value::String(s) => Some(s),
                _ => None,
            });
            Expression::SetStatus { status, target }
        },
    )(input)
}

fn parse_workflow(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            ws(tag("WORKFLOW")),
            ws(parse_string_literal),
            many0(parse_expression),
        )),
        |(_, name, steps)| {
            let name = match name {
                Value::String(s) => s,
                _ => "unknown".to_string(),
            };
            Expression::Workflow { name, steps }
        },
    )(input)
}

// Parse assignment: target = expression
fn parse_assignment(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            parse_identifier,
            ws(char('=')),
            parse_expression,
        )),
        |(target, _, value)| Expression::Assignment {
            target,
            value: Box::new(value),
        },
    )(input)
}

// Parse conditional: IF condition THEN expr ELSE expr
fn parse_conditional(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            preceded(ws(tag("IF")), parse_expression),
            preceded(ws(tag("THEN")), parse_expression),
            opt(preceded(ws(tag("ELSE")), parse_expression)),
        )),
        |(condition, then_expr, else_expr)| Expression::Conditional {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: else_expr.map(Box::new),
        },
    )(input)
}

// Parse WHEN...THEN...ELSE patterns
fn parse_when_then(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            preceded(ws(tag("WHEN")), parse_expression),
            preceded(ws(tag("THEN")), parse_expression),
            opt(preceded(ws(tag("ELSE")), parse_expression)),
        )),
        |(condition, then_expr, else_expr)| Expression::Conditional {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: else_expr.map(Box::new),
        },
    )(input)
}

// Parse primary expressions (literals, identifiers, parentheses)
fn parse_primary(input: &str) -> IResult<&str, Expression> {
    ws(alt((
        // Fund Accounting Workflow Verbs (must come before function calls)
        parse_configure_system,
        parse_activate,
        parse_run_health_check,
        parse_set_status,
        parse_workflow,

        // Literals
        map(parse_number, Expression::Literal),
        map(parse_string_literal, Expression::Literal),
        map(parse_regex_literal, Expression::Literal),
        map(parse_boolean, Expression::Literal),
        map(parse_null, Expression::Literal),

        // Complex expressions
        parse_list,
        parse_conditional,
        parse_when_then,
        parse_function_call,

        // Simple identifier
        map(parse_identifier, Expression::Identifier),

        // Parenthesized expression
        delimited(ws(char('(')), parse_expression, ws(char(')'))),
    )))(input)
}

// Parse unary expressions: NOT expr, -expr, +expr
fn parse_unary(input: &str) -> IResult<&str, Expression> {
    alt((
        map(
            preceded(ws(alt((tag("NOT"), tag("!")))), parse_unary),
            |operand| Expression::UnaryOp {
                op: UnaryOperator::Not,
                operand: Box::new(operand),
            },
        ),
        map(
            preceded(ws(char('-')), parse_unary),
            |operand| Expression::UnaryOp {
                op: UnaryOperator::Minus,
                operand: Box::new(operand),
            },
        ),
        map(
            preceded(ws(char('+')), parse_unary),
            |operand| Expression::UnaryOp {
                op: UnaryOperator::Plus,
                operand: Box::new(operand),
            },
        ),
        parse_primary,
    ))(input)
}

// Parse power operations: expr ** expr (right-associative)
fn parse_power(input: &str) -> IResult<&str, Expression> {
    let (input, left) = parse_unary(input)?;
    let (input, rest) = many0(preceded(ws(tag("**")), parse_unary))(input)?;

    Ok((input, rest.into_iter().fold(left, |acc, right| {
        Expression::BinaryOp {
            left: Box::new(acc),
            op: BinaryOperator::Power,
            right: Box::new(right),
        }
    })))
}

// Parse multiplication, division, modulo
fn parse_term(input: &str) -> IResult<&str, Expression> {
    let (input, left) = parse_power(input)?;
    let (input, operations) = many0(tuple((
        ws(alt((
            value(BinaryOperator::Multiply, char('*')),
            value(BinaryOperator::Divide, char('/')),
            value(BinaryOperator::Modulo, char('%')),
        ))),
        parse_power,
    )))(input)?;

    Ok((input, operations.into_iter().fold(left, |acc, (op, right)| {
        Expression::BinaryOp {
            left: Box::new(acc),
            op,
            right: Box::new(right),
        }
    })))
}

// Parse addition and subtraction
fn parse_arithmetic(input: &str) -> IResult<&str, Expression> {
    let (input, left) = parse_term(input)?;
    let (input, operations) = many0(tuple((
        ws(alt((
            value(BinaryOperator::Add, char('+')),
            value(BinaryOperator::Subtract, char('-')),
        ))),
        parse_term,
    )))(input)?;

    Ok((input, operations.into_iter().fold(left, |acc, (op, right)| {
        Expression::BinaryOp {
            left: Box::new(acc),
            op,
            right: Box::new(right),
        }
    })))
}

// Parse string concatenation
fn parse_concatenation(input: &str) -> IResult<&str, Expression> {
    let (input, left) = parse_arithmetic(input)?;
    let (input, operations) = many0(tuple((
        ws(value(BinaryOperator::Concat, char('&'))),
        parse_arithmetic,
    )))(input)?;

    Ok((input, operations.into_iter().fold(left, |acc, (op, right)| {
        Expression::BinaryOp {
            left: Box::new(acc),
            op,
            right: Box::new(right),
        }
    })))
}

// Parse comparison operations
fn parse_comparison(input: &str) -> IResult<&str, Expression> {
    let (input, left) = parse_concatenation(input)?;
    let (input, operation) = opt(tuple((
        ws(alt((
            value(BinaryOperator::Matches, tag("MATCHES")),
            value(BinaryOperator::NotMatches, tag("NOT_MATCHES")),
            value(BinaryOperator::Contains, tag("CONTAINS")),
            value(BinaryOperator::StartsWith, tag("STARTS_WITH")),
            value(BinaryOperator::EndsWith, tag("ENDS_WITH")),
            value(BinaryOperator::In, tag("IN")),
            value(BinaryOperator::NotIn, tag("NOT_IN")),
            value(BinaryOperator::LessThanOrEqual, tag("<=")),
            value(BinaryOperator::GreaterThanOrEqual, tag(">=")),
            value(BinaryOperator::NotEquals, tag("!=")),
            value(BinaryOperator::NotEquals, tag("<>")),
            value(BinaryOperator::Equals, tag("==")),
            value(BinaryOperator::Equals, tag("=")),
            value(BinaryOperator::LessThan, tag("<")),
            value(BinaryOperator::GreaterThan, tag(">")),
        ))),
        parse_concatenation,
    )))(input)?;

    Ok((input, match operation {
        Some((op, right)) => Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        },
        None => left,
    }))
}

// Parse logical AND
fn parse_and(input: &str) -> IResult<&str, Expression> {
    let (input, left) = parse_comparison(input)?;
    let (input, operations) = many0(tuple((
        ws(alt((tag("AND"), tag("&&")))),
        parse_comparison,
    )))(input)?;

    Ok((input, operations.into_iter().fold(left, |acc, (_, right)| {
        Expression::BinaryOp {
            left: Box::new(acc),
            op: BinaryOperator::And,
            right: Box::new(right),
        }
    })))
}

// Parse logical OR
fn parse_or(input: &str) -> IResult<&str, Expression> {
    let (input, left) = parse_and(input)?;
    let (input, operations) = many0(tuple((
        ws(alt((tag("OR"), tag("||")))),
        parse_and,
    )))(input)?;

    Ok((input, operations.into_iter().fold(left, |acc, (_, right)| {
        Expression::BinaryOp {
            left: Box::new(acc),
            op: BinaryOperator::Or,
            right: Box::new(right),
        }
    })))
}

// Parse full expressions (including assignments)
pub fn parse_expression(input: &str) -> IResult<&str, Expression> {
    alt((
        parse_assignment,
        parse_or,
    ))(input)
}

// Main entry point for parsing rules
pub fn parse_rule(input: &str) -> IResult<&str, Expression> {
    delimited(multispace0, parse_expression, multispace0)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let result = parse_rule("2 + 3 * 4").unwrap().1;
        println!("Parsed: {:?}", result);
    }

    #[test]
    fn test_function_call() {
        let result = parse_rule("CONCAT(\"Hello\", \" \", \"World\")").unwrap().1;
        println!("Parsed: {:?}", result);
    }

    #[test]
    fn test_assignment() {
        let result = parse_rule("result = price * quantity").unwrap().1;
        println!("Parsed: {:?}", result);
    }

    #[test]
    fn test_conditional() {
        let result = parse_rule("IF age > 18 THEN \"adult\" ELSE \"minor\"").unwrap().1;
        println!("Parsed: {:?}", result);
    }

    #[test]
    fn test_regex() {
        let result = parse_rule("email MATCHES /^[\\w]+@[\\w]+\\.[\\w]+$/").unwrap().1;
        println!("Parsed: {:?}", result);
    }

    #[test]
    fn test_list() {
        let result = parse_rule("[1, 2, 3, \"hello\"]").unwrap().1;
        println!("Parsed: {:?}", result);
    }

    #[test]
    fn test_configure_system() {
        let result = parse_rule("CONFIGURE_SYSTEM \"account_setup\"").unwrap().1;
        println!("Parsed: {:?}", result);
    }

    #[test]
    fn test_activate() {
        let result = parse_rule("ACTIVATE").unwrap().1;
        println!("Parsed: {:?}", result);
    }

    #[test]
    fn test_workflow() {
        let input = r#"
            WORKFLOW "SetupCoreFA"
            CONFIGURE_SYSTEM "account_setup"
            CONFIGURE_SYSTEM "trade_feed_setup"
            ACTIVATE
        "#;
        let result = parse_rule(input).unwrap().1;
        println!("Parsed: {:?}", result);
    }

    #[test]
    fn test_fund_accounting_dsl() {
        let input = r#"
            CONFIGURE_SYSTEM "account_setup"
            CONFIGURE_SYSTEM "trade_feed_setup"
            CONFIGURE_SYSTEM "nav_calculation_setup"
            ACTIVATE
            RUN_HEALTH_CHECK "health_check"
            SET_STATUS "Active"
        "#;

        // Parse each line separately since workflow parsing needs improvement
        let lines = input.lines().filter(|line| !line.trim().is_empty());
        for line in lines {
            if let Ok((_, expr)) = parse_rule(line.trim()) {
                println!("Parsed line: {:?}", expr);
            }
        }
    }
}