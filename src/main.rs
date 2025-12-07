use std::env;
use std::fs;
use std::io;

mod ast;
mod interpreter;
mod lexer;
mod parser;
mod token;
mod visualizer;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use visualizer::Visualizer;


fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let content = fs::read_to_string(filename)?;

    let lexer = Lexer::new(&content);
    let mut parser = match Parser::new(lexer) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let mut visualizer = Visualizer::new();
    let svg_content = visualizer.generate_svg(&ast);
    if let Err(e) = std::fs::write("ast.svg", svg_content) {
        eprintln!("Error writing SVG: {}", e);
    } else {
        println!("AST visualization saved to ast.svg");
    }

    let mut interpreter = Interpreter::new();
    match interpreter.interpret(&ast) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
    // Pretty print interpreter variables after execution completes
    interpreter.pretty_print_variables();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(text: &str) -> i32 {
        let lexer = Lexer::new(text);
        let mut parser = Parser::new(lexer).unwrap();
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        interpreter.interpret(&ast).unwrap()
    }

    #[test]
    fn test_single_integer() {
        assert_eq!(evaluate("123"), 123);
    }

    #[test]
    fn test_addition() {
        assert_eq!(evaluate("1 + 2"), 3);
    }

    #[test]
    fn test_subtraction() {
        assert_eq!(evaluate("10 - 3"), 7);
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(evaluate("4 * 5"), 20);
    }

    #[test]
    fn test_division() {
        assert_eq!(evaluate("20 / 4"), 5);
    }

    #[test]
    fn test_precedence() {
        assert_eq!(evaluate("2 + 3 * 4"), 14);
        assert_eq!(evaluate("10 - 4 / 2"), 8);
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(evaluate("  12   +   34  "), 46);
    }

    #[test]
    fn test_complex_expression() {
        assert_eq!(evaluate("3 + 5 * 2 - 8 / 4"), 11);
    }

    #[test]
    fn test_parenthesis() {
        assert_eq!(evaluate("(1 + 2) * 3"), 9);
        assert_eq!(evaluate("10 / (2 + 3)"), 2);
    }

    #[test]
    fn test_nested_parenthesis() {
        assert_eq!(evaluate("((1 + 2) * (3 + 4))"), 21);
        assert_eq!(evaluate("2 * (3 + (4 * 5))"), 46);
    }

    #[test]
    fn test_unary_minus() {
        assert_eq!(evaluate("-3"), -3);
    }

    #[test]
    fn test_unary_plus() {
        assert_eq!(evaluate("+3"), 3);
    }

    #[test]
    fn test_multiple_unary_operators() {
        assert_eq!(evaluate("--3"), 3);
        assert_eq!(evaluate("++3"), 3);
        assert_eq!(evaluate("-+3"), -3);
        assert_eq!(evaluate("---3"), -3);
    }

    #[test]
    fn test_unary_operators_in_expressions() {
        assert_eq!(evaluate("5 - -2"), 7);
        assert_eq!(evaluate("5 + +2"), 7);
        assert_eq!(evaluate("5 * -2"), -10);
        assert_eq!(evaluate("-5 * -2"), 10);
    }
}
