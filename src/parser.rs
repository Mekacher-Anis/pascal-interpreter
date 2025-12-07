use crate::ast::ASTNode;
use crate::lexer::Lexer;
use crate::token::Token;
use anyhow::{anyhow, Result};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Result<Self> {
        let current_token = lexer.next_token().unwrap_or(Token::Eof);
        Ok(Parser {
            lexer,
            current_token,
        })
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
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    fn factor(&mut self) -> Result<ASTNode> {
        match self.current_token {
            Token::Plus => {
                self.eat(Some(&Token::Plus))?;
                Ok(ASTNode::UnaryOpNode {
                    token: Token::Plus,
                    expr: Box::new(self.factor()?),
                })
            }
            Token::Minus => {
                self.eat(Some(&Token::Minus))?;
                Ok(ASTNode::UnaryOpNode {
                    token: Token::Minus,
                    expr: Box::new(self.factor()?),
                })
            }
            Token::Integer(val) => {
                self.eat(Some(&Token::Integer(0)))?;
                Ok(ASTNode::NumNode {
                    token: Token::Integer(val),
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
                _ => break,
            }
        }

        Ok(result)
    }

    pub fn parse(&mut self) -> Result<ASTNode> {
        self.expr()
    }

    fn expr(&mut self) -> Result<ASTNode> {
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
                _ => break,
            }
        }

        Ok(result)
    }
}
