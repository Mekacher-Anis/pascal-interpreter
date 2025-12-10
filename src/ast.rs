use crate::token::Token;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ASTNode {
    Program {
        name: String,
        block: Box<ASTNode>,
    },
    Block {
        declarations: Vec<Box<ASTNode>>,
        compound_statement: Box<ASTNode>,
    },
    ProcedureDecl {
        proc_name: String,
        params: Vec<Box<ASTNode>>,
        block_node: Box<ASTNode>,
    },
    Param {
        var_node: Box<ASTNode>,
        type_node: Box<ASTNode>,
    },
    VarDecl {
        var_node: Box<ASTNode>,
        type_node: Box<ASTNode>,
    },
    Type {
        token: Token,
        value: String,
    },
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
        value: BuiltinNumTypes,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum BuiltinNumTypes {
    I32(i32),
    F32(f32),
}

impl fmt::Display for BuiltinNumTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuiltinNumTypes::I32(val) => write!(f, "{}", val),
            BuiltinNumTypes::F32(val) => write!(f, "{}", val),
        }
    }
}

impl fmt::Display for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ASTNode::Program { name, block } => write!(f, "PROGRAM {};\n{}", name, block),
            ASTNode::Block {
                declarations,
                compound_statement,
            } => {
                for decl in declarations {
                    write!(f, "{}\n", decl)?;
                }
                write!(f, "{}", compound_statement)
            }
            ASTNode::VarDecl {
                var_node,
                type_node,
            } => write!(f, "VAR {} : {};", var_node, type_node),
            ASTNode::Type { value, .. } => write!(f, "{}", value),
            ASTNode::Compound { children } => {
                write!(f, "BEGIN\n")?;
                for child in children {
                    write!(f, "{};\n", child)?;
                }
                write!(f, "END")
            }
            ASTNode::Assign { left, right, .. } => write!(f, "{} := {}", left, right),
            ASTNode::Var { name } => write!(f, "{}", name),
            ASTNode::NoOp => Ok(()),
            ASTNode::UnaryOpNode { expr, token } => write!(f, "{}{}", token, expr),
            ASTNode::BinOpNode { left, right, op } => write!(f, "{} {} {}", left, op, right),
            ASTNode::NumNode { value, .. } => write!(f, "{}", value),
            ASTNode::ProcedureDecl {
                proc_name: name, ..
            } => write!(f, "fn {name}"),
            ASTNode::Param {
                var_node,
                type_node,
            } => write!(f, "param({}: {})", var_node.as_ref(), type_node.as_ref()),
        }
    }
}
