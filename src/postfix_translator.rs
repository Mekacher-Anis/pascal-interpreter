use crate::ast::ASTNode;
use crate::token::Token;
use anyhow::{anyhow, Result};

pub struct PostfixTranslator;

impl PostfixTranslator {
    pub fn new() -> Self {
        PostfixTranslator
    }

    pub fn visit(&self, node: &ASTNode) -> Result<String> {
        match node {
            ASTNode::NumNode { value, .. } => Ok(value.to_string()),
            ASTNode::BinOpNode { left, right, op } => {
                let left_val = self.visit(left)?;
                let right_val = self.visit(right)?;
                match op {
                    Token::Plus => Ok(format!("{left_val} {right_val} +")),
                    Token::Minus => Ok(format!("{left_val} {right_val} -")),
                    Token::Asterisk => Ok(format!("{left_val} {right_val} *")),
                    Token::Slash => Ok(format!("{left_val} {right_val} /")),
                    _ => Err(anyhow!("Invalid operator")),
                }
            }
        }
    }

    pub fn translate(&self, node: &ASTNode) -> Result<String> {
        self.visit(node)
    }
}
