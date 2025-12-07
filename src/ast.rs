use crate::token::Token;

#[derive(Debug)]
pub enum ASTNode {
    Compound {
        children: Vec<Box<ASTNode>>,
    },
    Assign {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
        token: Token,
    },
    Var {
        name: String,
        token: Token,
    },
    NoOp,
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
