use crate::token::{LocatedToken, Token, RESERVER_KEYWORDS};
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} at line {}, column {}",
            self.message, self.line, self.column
        )?;
        writeln!(f, "{}", self.snippet)?;
        writeln!(f, "{:>width$}^", "", width = self.column.saturating_sub(1))
    }
}

impl std::error::Error for LexerError {}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    input: &'a str,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Lexer {
            chars: text.chars().peekable(),
            input: text,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn consume(&mut self) -> Option<char> {
        let ch = self.chars.next();
        if let Some(ch) = ch {
            self.pos += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    fn snippet_at(&self, pos: usize) -> String {
        let start = self.input[..pos]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let end = self.input[pos..]
            .find('\n')
            .map(|i| pos + i)
            .unwrap_or(self.input.len());
        self.input[start..end].to_string()
    }

    fn get_snippet(&self) -> String {
        self.snippet_at(self.pos)
    }

    fn number(&mut self) -> Result<Token, LexerError> {
        let mut number_str = String::new();

        while let Some(&ch) = self.chars.peek() {
            if ch.is_ascii_digit() {
                number_str.push(ch);
                self.consume();
            } else {
                break;
            }
        }

        if number_str.is_empty() {
            return Err(LexerError {
                message: "Expected integer but found none".to_string(),
                line: self.line,
                column: self.column,
                snippet: self.get_snippet(),
            });
        }

        if let Some('.') = self.chars.peek() {
            number_str.push('.');
            self.consume();

            while let Some(&ch) = self.chars.peek() {
                if ch.is_ascii_digit() {
                    number_str.push(ch);
                    self.consume();
                } else {
                    break;
                }
            }

            let float_val = number_str.parse::<f32>().map_err(|e| LexerError {
                message: format!("Parse error: {}", e),
                line: self.line,
                column: self.column,
                snippet: self.get_snippet(),
            })?;
            return Ok(Token::RealConst(float_val));
        }

        let int_val = number_str.parse::<i32>().map_err(|e| LexerError {
            message: format!("Parse error: {}", e),
            line: self.line,
            column: self.column,
            snippet: self.get_snippet(),
        })?;
        Ok(Token::IntegerConst(int_val))
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.consume() {
            if ch == '}' {
                break;
            }
        }
    }

    fn _id(&mut self) -> Result<Token, LexerError> {
        let mut result = String::new();
        while self.chars.peek().map_or(false, |c| c.is_alphanumeric()) {
            result.push(self.consume().unwrap().to_ascii_lowercase());
        }

        let v = RESERVER_KEYWORDS
            .get(&result)
            .map_or(Token::Id(result), |v| v.clone());
        Ok(v)
    }

    pub fn next_token(&mut self) -> Result<LocatedToken, LexerError> {
        self.skip_whitespace();

        let start_line = self.line;
        let start_column = self.column;
        let start_pos = self.pos;

        let token = match self.chars.peek().copied() {
            None => Token::Eof,
            Some(ch) if ch.is_ascii_digit() => self.number()?,
            Some(ch) if ch.is_alphanumeric() => self._id()?,
            Some('{') => {
                self.consume();
                self.skip_comment();
                return self.next_token();
            }
            _ => {
                let c = self.consume().unwrap();
                match c {
                    ':' if self.chars.peek() == Some(&'=') => {
                        self.consume();
                        Token::Assign
                    }
                    '+' => Token::Plus,
                    '-' => Token::Minus,
                    '*' => Token::Asterisk,
                    '/' => Token::FloatDiv,
                    '(' => Token::LParenthesis,
                    ')' => Token::RParenthesis,
                    '.' => Token::Dot,
                    ';' => Token::Semi,
                    ':' => Token::Colon,
                    ',' => Token::Comma,
                    _ => {
                        return Err(LexerError {
                            message: format!("Unexpected character '{}'", c),
                            line: self.line,
                            column: self.column,
                            snippet: self.get_snippet(),
                        })
                    }
                }
            }
        };

        Ok(LocatedToken::new(
            token,
            start_line,
            start_column,
            self.snippet_at(start_pos),
        ))
    }
}
