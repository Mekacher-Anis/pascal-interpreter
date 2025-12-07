use crate::token::Token;
use anyhow::{anyhow, Result};
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Lexer {
            chars: text.chars().peekable(),
        }
    }

    fn integer(&mut self) -> Result<i32> {
        let mut result: i32 = 0;
        let mut found = false;

        while let Some(&ch) = self.chars.peek() {
            if let Some(digit) = ch.to_digit(10) {
                result = result * 10 + digit as i32;
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

    fn _id(&mut self) -> Result<Token> {
        let mut result = String::new();
        while self.chars.peek().map_or(false, |c| c.is_alphanumeric()) {
            result.push(self.chars.next().unwrap());
        }

        match result.as_str() {
            "BEGIN" => Ok(Token::Begin),
            "END" => Ok(Token::End),
            _ => Ok(Token::Id(result)),
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        let Some(ch) = self.chars.peek().copied() else {
            return Ok(Token::Eof);
        };

        if ch.is_ascii_digit() {
            return Ok(Token::Integer(self.integer()?));
        }
        if ch.is_alphanumeric() {
            return Ok(self._id()?);
        }

        match self.chars.next().unwrap() {
            ':' if self.chars.peek() == Some(&'=') => {
                self.chars.next();
                Ok(Token::Assign)
            }
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Asterisk),
            '/' => Ok(Token::Slash),
            '(' => Ok(Token::LParenthesis),
            ')' => Ok(Token::RParenthesis),
            '.' => Ok(Token::Dot),
            ';' => Ok(Token::Semi),
            c => Err(anyhow!("Unexpected character: {}", c)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    #[test]
    fn test_next_token_basic() {
        let input = "+ - * / ( ) . ;";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::Plus);
        assert_eq!(lexer.next_token().unwrap(), Token::Minus);
        assert_eq!(lexer.next_token().unwrap(), Token::Asterisk);
        assert_eq!(lexer.next_token().unwrap(), Token::Slash);
        assert_eq!(lexer.next_token().unwrap(), Token::LParenthesis);
        assert_eq!(lexer.next_token().unwrap(), Token::RParenthesis);
        assert_eq!(lexer.next_token().unwrap(), Token::Dot);
        assert_eq!(lexer.next_token().unwrap(), Token::Semi);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_next_token_integer() {
        let input = "123 456";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::Integer(123));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(456));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_next_token_identifiers_and_keywords() {
        let input = "BEGIN END x y123";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::Begin);
        assert_eq!(lexer.next_token().unwrap(), Token::End);
        assert_eq!(lexer.next_token().unwrap(), Token::Id("x".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Id("y123".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_next_token_assign() {
        let input = ":=";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::Assign);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_next_token_expression() {
        let input = "BEGIN x := 10; END.";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::Begin);
        assert_eq!(lexer.next_token().unwrap(), Token::Id("x".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Assign);
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(10));
        assert_eq!(lexer.next_token().unwrap(), Token::Semi);
        assert_eq!(lexer.next_token().unwrap(), Token::End);
        assert_eq!(lexer.next_token().unwrap(), Token::Dot);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_unexpected_character() {
        let input = "?";
        let mut lexer = Lexer::new(input);

        assert!(lexer.next_token().is_err());
    }
}
