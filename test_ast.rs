extern crate data_designer;
use data_designer::parser;

fn main() {
    let test_expressions = vec![
        "result = 42",
        "result = price * quantity",
        "result = price * quantity + tax",
        "name = \"Hello World\"",
    ];

    for expr in test_expressions {
        println!("\nTesting: {}", expr);
        match parser::parse_rule(expr) {
            Ok((remaining, ast)) => {
                println!("Success! Remaining: '{}', AST: {:#?}", remaining, ast);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}