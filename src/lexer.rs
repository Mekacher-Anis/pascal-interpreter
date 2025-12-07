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

    pub fn next_token(&mut self) -> Result<Token> {
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
}
