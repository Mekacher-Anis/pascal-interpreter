use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Integer(i32),
    Plus,
    Minus,
    Asterisk,
    Slash,
    LParenthesis,
    RParenthesis,
    Begin,
    End,
    Dot,
    Id(String),
    Assign,
    Semi,
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
            Token::Begin => write!(f, "BEGIN"),
            Token::End => write!(f, "END"),
            Token::Dot => write!(f, "."),
            Token::Id(name) => write!(f, "{name}"),
            Token::Assign => write!(f, ":="),
            Token::Semi => write!(f, "SEMI"),
        }
    }
}
