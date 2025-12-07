use crate::token::Token;

#[derive(Debug)]
pub enum ASTNode {
    UnaryOpNode {
        expr: Box<ASTNode>,
        token: Token,
    },
    BinOpNode {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
        op: Token,
    },
    NumNode {
        token: Token,
        value: i32,
    },
}
