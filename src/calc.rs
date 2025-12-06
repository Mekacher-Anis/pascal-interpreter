use anyhow::{anyhow, Result};
use std::iter::Peekable;
use std::str::Chars;
use std::{fmt, result};

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Integer(u32),
    Plus,
    Minus,
    Asterisk,
    Slash,
    LParenthesis,
    RParenthesis,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Integer(n) => write!(f, "Integer({})", n),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Asterisk => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Eof => write!(f, "EOF"),
            Token::LParenthesis => write!(f, "("),
            Token::RParenthesis => write!(f, ")"),
        }
    }
}

enum ASTNode {
    BinOpNode {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
        op: Token,
    },
    NumNode {
        token: Token,
        value: u32,
    },
}

pub struct Interpreter<'a> {
    chars: Peekable<Chars<'a>>,
    current_token: Token,
}

impl<'a> Interpreter<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut interpreter = Interpreter {
            chars: text.chars().peekable(),
            current_token: Token::Eof,
        };
        interpreter.current_token = interpreter.generate_next_token().unwrap_or(Token::Eof);
        interpreter
    }

    fn integer(&mut self) -> Result<u32> {
        let mut result: u32 = 0;
        let mut found = false;

        while let Some(&ch) = self.chars.peek() {
            if let Some(digit) = ch.to_digit(10) {
                result = result * 10 + digit;
                self.chars.next();
                found = true;
            } else {
                break;
            }
        }

        if !found {
            return Err(anyhow!("Expected integer but found none"));
        }
        Ok(result)
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn generate_next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        if let Some(ch) = (&mut self.chars).peek().copied() {
            if ch.is_ascii_digit() {
                return Ok(Token::Integer(self.integer()?));
            }

            self.chars.next();

            match ch {
                '+' => Ok(Token::Plus),
                '-' => Ok(Token::Minus),
                '*' => Ok(Token::Asterisk),
                '/' => Ok(Token::Slash),
                '(' => Ok(Token::LParenthesis),
                ')' => Ok(Token::RParenthesis),
                _ => Err(anyhow!("Unexpected character: {}", ch)),
            }
        } else {
            Ok(Token::Eof)
        }
    }

    fn eat(&mut self, expected_type: Option<&Token>) -> Result<()> {
        if let Some(expected) = expected_type {
            if std::mem::discriminant(&self.current_token) != std::mem::discriminant(expected) {
                return Err(anyhow!(
                    "Expected {:?}, found {:?}",
                    expected,
                    self.current_token
                ));
            }
        }
        self.current_token = self.generate_next_token()?;
        Ok(())
    }

    fn factor(&mut self) -> Result<ASTNode> {
        match self.current_token {
            Token::Integer(val) => {
                self.eat(Some(&Token::Integer(0)))?;
                Ok(ASTNode::NumNode {
                    token: self.current_token.clone(),
                    value: val,
                })
            }
            Token::LParenthesis => {
                self.eat(Some(&Token::LParenthesis))?;
                let result = self.expr()?;
                self.eat(Some(&Token::RParenthesis))?;
                Ok(result)
            }
            _ => return Err(anyhow!("Syntax error, expected an integer")),
        }
    }

    fn term(&mut self) -> Result<ASTNode> {
        let mut result = self.factor()?;

        loop {
            let op = self.current_token.clone();

            match op {
                Token::Eof => break,
                Token::Asterisk | Token::Slash => {
                    self.eat(Some(&op))?;

                    let right_node = self.factor()?;

                    match op {
                        Token::Asterisk | Token::Slash => {
                            result = ASTNode::BinOpNode {
                                left: Box::new(result),
                                right: Box::new(right_node),
                                op,
                            }
                        }
                        _ => break,
                    }
                }
                // _ => return Err(anyhow!("Unexpected token in expression: {:?}", op)),
                _ => break,
            }
        }

        Ok(result)
    }

    pub fn expr(&mut self) -> Result<ASTNode> {
        // consume op
        let mut result = self.term()?;

        loop {
            let op = self.current_token.clone();

            match op {
                Token::Eof => break,
                Token::Plus | Token::Minus => {
                    self.eat(Some(&op))?;

                    let right = self.term()?;

                    match op {
                        Token::Plus | Token::Minus => {
                            result = ASTNode::BinOpNode {
                                left: Box::new(result),
                                right: Box::new(right),
                                op,
                            }
                        }
                        _ => break,
                    }
                }
                // _ => return Err(anyhow!("Unexpected token in expression: {:?}", op)),
                _ => break,
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_integer() {
        let mut interpreter = Interpreter::new("123");
        assert_eq!(interpreter.expr().unwrap(), 123);
    }

    #[test]
    fn test_addition() {
        let mut interpreter = Interpreter::new("1 + 2");
        assert_eq!(interpreter.expr().unwrap(), 3);
    }

    #[test]
    fn test_subtraction() {
        let mut interpreter = Interpreter::new("10 - 3");
        assert_eq!(interpreter.expr().unwrap(), 7);
    }

    #[test]
    fn test_multiplication() {
        let mut interpreter = Interpreter::new("4 * 5");
        assert_eq!(interpreter.expr().unwrap(), 20);
    }

    #[test]
    fn test_division() {
        let mut interpreter = Interpreter::new("20 / 4");
        assert_eq!(interpreter.expr().unwrap(), 5);
    }

    #[test]
    fn test_precedence() {
        let mut interpreter = Interpreter::new("2 + 3 * 4");
        assert_eq!(interpreter.expr().unwrap(), 14);

        let mut interpreter = Interpreter::new("10 - 4 / 2");
        assert_eq!(interpreter.expr().unwrap(), 8);
    }

    #[test]
    fn test_whitespace() {
        let mut interpreter = Interpreter::new("  12   +   34  ");
        assert_eq!(interpreter.expr().unwrap(), 46);
    }

    #[test]
    fn test_complex_expression() {
        let mut interpreter = Interpreter::new("3 + 5 * 2 - 8 / 4");
        // 3 + 10 - 2 = 11
        assert_eq!(interpreter.expr().unwrap(), 11);
    }

    #[test]
    fn test_invalid_syntax() {
        let mut interpreter = Interpreter::new("1 + + 2");
        assert!(interpreter.expr().is_err());
    }

    #[test]
    fn test_unexpected_char() {
        let mut interpreter = Interpreter::new("1 & 2");
        assert!(interpreter.expr().is_err());
    }

    #[test]
    fn test_parenthesis() {
        let mut interpreter = Interpreter::new("(1 + 2) * 3");
        assert_eq!(interpreter.expr().unwrap(), 9);

        let mut interpreter = Interpreter::new("10 / (2 + 3)");
        assert_eq!(interpreter.expr().unwrap(), 2);
    }

    #[test]
    fn test_nested_parenthesis() {
        let mut interpreter = Interpreter::new("((1 + 2) * (3 + 4))");
        assert_eq!(interpreter.expr().unwrap(), 21);

        let mut interpreter = Interpreter::new("2 * (3 + (4 * 5))");
        assert_eq!(interpreter.expr().unwrap(), 46);
    }

    #[test]
    fn test_ast_creation() {
        let leaf_1 = ASTNode::NumNode {
            token: Token::Integer(2),
            value: 2,
        };
        let leaf_2 = ASTNode::NumNode {
            token: Token::Integer(7),
            value: 7,
        };
        let leaf_3 = ASTNode::NumNode {
            token: Token::Integer(3),
            value: 3,
        };
        let mul_node = ASTNode::BinOpNode {
            left: Box::new(leaf_1),
            right: Box::new(leaf_2),
            op: Token::Asterisk,
        };
        let add_node = ASTNode::BinOpNode {
            left: Box::new(mul_node),
            right: Box::new(leaf_3),
            op: Token::Plus,
        };
    }
}
