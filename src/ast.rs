use crate::token::Token;

#[derive(Debug)]
pub enum ASTNode {
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
