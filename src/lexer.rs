use crate::token::{Token, RESERVER_KEYWORDS};
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

    fn number(&mut self) -> Result<Token> {
        let mut number_str = String::new();

        while let Some(&ch) = self.chars.peek() {
            if ch.is_ascii_digit() {
                number_str.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }

        if number_str.is_empty() {
            return Err(anyhow!("Expected integer but found none"));
        }

        if let Some('.') = self.chars.peek() {
            number_str.push('.');
            self.chars.next();

            while let Some(&ch) = self.chars.peek() {
                if ch.is_ascii_digit() {
                    number_str.push(ch);
                    self.chars.next();
                } else {
                    break;
                }
            }

            let float_val = number_str.parse::<f32>()?;
            return Ok(Token::RealConst(float_val));
        }

        let int_val = number_str.parse::<i32>()?;
        Ok(Token::IntegerConst(int_val))
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

    fn skip_comment(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            self.chars.next();
            if ch == '}' {
                break;
            }
        }
    }

    fn _id(&mut self) -> Result<Token> {
        let mut result = String::new();
        while self.chars.peek().map_or(false, |c| c.is_alphanumeric()) {
            result.push(self.chars.next().unwrap().to_ascii_lowercase());
        }

        let v = RESERVER_KEYWORDS
            .get(&result)
            .map_or(Token::Id(result), |v| v.clone());
        Ok(v)
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        let Some(ch) = self.chars.peek().copied() else {
            return Ok(Token::Eof);
        };

        if ch.is_ascii_digit() {
            return Ok(self.number()?);
        } else if ch.is_alphanumeric() {
            return Ok(self._id()?);
        } else if ch == '{' {
            self.chars.next();
            self.skip_comment();
            return self.next_token();
        }

        match self.chars.next().unwrap() {
            ':' if self.chars.peek() == Some(&'=') => {
                self.chars.next();
                Ok(Token::Assign)
            }
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Asterisk),
            '/' => Ok(Token::FloatDiv),
            '(' => Ok(Token::LParenthesis),
            ')' => Ok(Token::RParenthesis),
            '.' => Ok(Token::Dot),
            ';' => Ok(Token::Semi),
            ':' => Ok(Token::Colon),
            ',' => Ok(Token::Comma),
            c => Err(anyhow!("Unexpected character: {}", c)),
        }
    }
}
