use crate::ast::ASTNode;
use crate::token::Token;
use anyhow::{anyhow, Result};

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Interpreter
    }

    pub fn visit(&self, node: &ASTNode) -> Result<i32> {
        match node {
            ASTNode::NumNode { value, .. } => Ok(*value),
            ASTNode::UnaryOpNode { expr, token } => {
                let value = self.visit(&expr)?;
                match token {
                    Token::Plus => Ok(value),
                    Token::Minus => Ok(-value),
                    _ => Err(anyhow!("Invalid operator")),
                }
            }
            ASTNode::BinOpNode { left, right, op } => {
                let left_val = self.visit(left)?;
                let right_val = self.visit(right)?;
                match op {
                    Token::Plus => Ok(left_val + right_val),
                    Token::Minus => Ok(left_val - right_val),
                    Token::Asterisk => Ok(left_val * right_val),
                    Token::Slash => Ok(left_val / right_val),
                    _ => Err(anyhow!("Invalid operator")),
                }
            }
        }
    }

    pub fn interpret(&self, node: &ASTNode) -> Result<i32> {
        self.visit(node)
    }
}
