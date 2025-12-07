use std::io::{self, Write};

mod ast;
mod interpreter;
mod lexer;
mod parser;
mod token;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn main() -> io::Result<()> {
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut buffer = String::new();
        let bytes_read = io::stdin().read_line(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        if buffer.trim().is_empty() {
            continue;
        }
        if buffer.trim() == "/q" {
            break;
        }

        let lexer = Lexer::new(&buffer);
        let mut parser = match Parser::new(lexer) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error: {}", e);
                continue;
            }
        };

        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Error: {}", e);
                continue;
            }
        };

        let interpreter = Interpreter::new();
        match interpreter.interpret(&ast) {
            Ok(result) => println!("{}", result),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(text: &str) -> u32 {
        let lexer = Lexer::new(text);
        let mut parser = Parser::new(lexer).unwrap();
        let ast = parser.parse().unwrap();
        let interpreter = Interpreter::new();
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
}
