use phf::phf_map;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Program,
    Var,
    Colon,
    Comma,
    IntegerConst(i32),
    Integer,
    IntegerDiv,
    RealConst(f32),
    Real,
    FloatDiv,
    Plus,
    Minus,
    Asterisk,
    LParenthesis,
    RParenthesis,
    Begin,
    End,
    Dot,
    Id(String),
    Assign,
    Semi,
    Eof,
    Procedure,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocatedToken {
    pub token: Token,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
}

impl LocatedToken {
    pub fn new(token: Token, line: usize, column: usize, snippet: String) -> Self {
        Self {
            token,
            line,
            column,
            snippet,
        }
    }
}

pub static RESERVER_KEYWORDS: phf::Map<&'static str, Token> = phf_map! {
    "program" => Token::Program,
    "begin" => Token::Begin,
    "end" => Token::End,
    "var" => Token::Var,
    "div" => Token::IntegerDiv,
    "integer" => Token::Integer,
    "real" => Token::Real,
    "procedure" => Token::Procedure,
};

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::IntegerConst(n) => write!(f, "IntegerConst({n})"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Asterisk => write!(f, "*"),
            Token::Eof => write!(f, "EOF"),
            Token::LParenthesis => write!(f, "("),
            Token::RParenthesis => write!(f, ")"),
            Token::Begin => write!(f, "BEGIN"),
            Token::End => write!(f, "END"),
            Token::Dot => write!(f, "."),
            Token::Id(name) => write!(f, "{name}"),
            Token::Assign => write!(f, ":="),
            Token::Semi => write!(f, "SEMI"),
            Token::Program => write!(f, "PROGRAM"),
            Token::Var => write!(f, "var"),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::Integer => write!(f, "INTEGER"),
            Token::IntegerDiv => write!(f, "DIV"),
            Token::RealConst(v) => write!(f, "RealConst({v})"),
            Token::Real => write!(f, "REAL"),
            Token::FloatDiv => write!(f, "/"),
            Token::Procedure => write!(f, "PROCEDURE"),
        }
    }
}
