use std::collections::HashMap;
use std::fmt;

use crate::ast::{ASTNode, BuiltinNumTypes};
use crate::symbols::{Symbol, SymbolKind, SymbolTable};
use crate::token::Token;

pub type InterpretResult<T> = std::result::Result<T, InterpretError>;

#[derive(Debug, Clone)]
pub enum InterpretError {
    InvalidVarDeclVarNode,
    InvalidVarDeclTypeNode,
    UndefinedType { type_name: String, var_name: String },
    AssignTargetMustBeVar,
    UndefinedVariable { name: String },
    UninitializedVariable { name: String },
    MissingUnaryOperand,
    InvalidUnaryOperator { token: Token },
    MissingBinaryOperand { side: BinaryOperandSide },
    InvalidBinaryOperator { token: Token },
    MissingAssignmentValue { name: String },
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperandSide {
    Left,
    Right,
}

impl fmt::Display for BinaryOperandSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperandSide::Left => write!(f, "left"),
            BinaryOperandSide::Right => write!(f, "right"),
        }
    }
}

impl fmt::Display for InterpretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpretError::InvalidVarDeclVarNode => {
                write!(f, "Variable declarations must start with a variable name")
            }
            InterpretError::InvalidVarDeclTypeNode => {
                write!(f, "Variable declarations must specify a valid type")
            }
            InterpretError::UndefinedType {
                type_name,
                var_name,
            } => write!(
                f,
                "Undefined type '{type_name}' used for variable '{var_name}'"
            ),
            InterpretError::AssignTargetMustBeVar => {
                write!(f, "The left-hand side of an assignment must be a variable")
            }
            InterpretError::UndefinedVariable { name } => {
                write!(f, "Undefined variable '{name}'")
            }
            InterpretError::UninitializedVariable { name } => {
                write!(f, "Variable '{name}' is declared but has no value yet")
            }
            InterpretError::MissingUnaryOperand => {
                write!(f, "Unary operation is missing its operand")
            }
            InterpretError::InvalidUnaryOperator { token } => {
                write!(f, "Invalid unary operator '{token}'")
            }
            InterpretError::MissingBinaryOperand { side } => {
                write!(f, "Binary operation missing its {side} operand")
            }
            InterpretError::InvalidBinaryOperator { token } => {
                write!(f, "Invalid binary operator '{token}'")
            }
            InterpretError::MissingAssignmentValue { name } => {
                write!(f, "Assignment to '{name}' is missing a value")
            }
        }
    }
}

impl std::error::Error for InterpretError {}

pub struct Interpreter {
    pub global_memory: HashMap<String, BuiltinNumTypes>,
    pub symtab: SymbolTable,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            global_memory: HashMap::new(),
            symtab: SymbolTable::new(),
        }
    }

    /// Pretty print the variable hashmap in sorted order by variable name.
    ///
    /// This prints one variable per line with two-space indentation, for example:
    ///
    /// Variables:
    ///   a: 1
    ///   b: 2
    pub fn pretty_print_variables(&self) {
        if self.global_memory.is_empty() {
            println!("Variables: {{}} (no variables)");
            return;
        }

        println!("Variables:");
        let mut entries: Vec<_> = self.global_memory.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        for (k, v) in entries {
            println!("  {}: {}", k, v);
        }
    }

    pub fn interpret(&mut self, node: &ASTNode) -> InterpretResult<Option<BuiltinNumTypes>> {
        self.visit(node)
    }

    pub fn visit(&mut self, node: &ASTNode) -> InterpretResult<Option<BuiltinNumTypes>> {
        match node {
            ASTNode::NumNode { value, .. } => {
                let res = self.visit_num_node(*value)?;
                Ok(Some(res))
            }
            ASTNode::UnaryOpNode { expr, token } => {
                let res = self.visit_unary_op_node(token, expr)?;
                Ok(Some(res))
            }
            ASTNode::BinOpNode { left, right, op } => {
                let res = self.visit_bin_op_node(op, left, right)?;
                Ok(Some(res))
            }
            ASTNode::Assign { left, right, .. } => {
                self.visit_assign_node(left, right)?;
                Ok(None)
            }
            ASTNode::Var { name: value, .. } => {
                let value = self.visit_var_node(value)?;
                Ok(Some(value))
            }
            ASTNode::Compound { children } => {
                self.visit_compound_node(children)?;
                Ok(None)
            }
            ASTNode::NoOp => Ok(None),
            ASTNode::Program { name, block } => {
                self.visit_program_node(name, block)?;
                Ok(None)
            }
            ASTNode::Block {
                declarations,
                compound_statement,
            } => {
                self.visit_block_node(declarations, compound_statement)?;
                Ok(None)
            }
            ASTNode::VarDecl {
                var_node,
                type_node,
            } => {
                self.visit_var_decl_node(var_node, type_node)?;
                Ok(None)
            }
            ASTNode::Type { value, .. } => {
                self.visit_type_node(value)?;
                Ok(None)
            }
            ASTNode::ProcedureDecl {
                proc_name: name,
                block_node: block,
            } => {
                self.visit_procedure_decl_node(name, block)?;
                Ok(None)
            }
        }
    }

    fn visit_program_node(&mut self, _name: &String, block: &Box<ASTNode>) -> InterpretResult<()> {
        self.visit(block)?;
        Ok(())
    }

    fn visit_block_node(
        &mut self,
        declarations: &Vec<Box<ASTNode>>,
        compound_statement: &Box<ASTNode>,
    ) -> InterpretResult<()> {
        for d in declarations {
            self.visit(d)?;
        }

        self.visit(compound_statement)?;

        Ok(())
    }

    fn visit_var_decl_node(
        &mut self,
        var_node: &Box<ASTNode>,
        type_node: &Box<ASTNode>,
    ) -> InterpretResult<()> {
        let ASTNode::Var { name: var_name } = &**var_node else {
            return Err(InterpretError::InvalidVarDeclVarNode);
        };
        let ASTNode::Type {
            value: type_name, ..
        } = &**type_node
        else {
            return Err(InterpretError::InvalidVarDeclTypeNode);
        };

        // make sure it's defined first
        self.symtab
            .lookup(type_name)
            .ok_or_else(|| InterpretError::UndefinedType {
                type_name: type_name.clone(),
                var_name: var_name.clone(),
            })?;

        let symbol = Symbol {
            name: var_name.clone(),
            kind: SymbolKind::Variable {
                type_name: type_name.to_owned(),
            },
        };

        self.symtab.define(symbol);

        Ok(())
    }

    fn visit_procedure_decl_node(
        &mut self,
        procedure_name: &String,
        block: &Box<ASTNode>,
    ) -> InterpretResult<()> {
        Ok(())
    }

    fn visit_type_node(&self, _value: &String) -> InterpretResult<()> {
        Ok(())
    }

    fn visit_num_node(&self, value: BuiltinNumTypes) -> InterpretResult<BuiltinNumTypes> {
        Ok(value)
    }

    fn visit_unary_op_node(
        &mut self,
        token: &Token,
        expr: &ASTNode,
    ) -> InterpretResult<BuiltinNumTypes> {
        let value = match self.visit(expr)? {
            Some(BuiltinNumTypes::F32(v)) => v,
            Some(BuiltinNumTypes::I32(v)) => v as f32,
            None => return Err(InterpretError::MissingUnaryOperand),
        };

        match token {
            Token::Plus => Ok(BuiltinNumTypes::F32(value)),
            Token::Minus => Ok(BuiltinNumTypes::F32(-value)),
            _ => Err(InterpretError::InvalidUnaryOperator {
                token: token.clone(),
            }),
        }
    }

    fn visit_bin_op_node(
        &mut self,
        op: &Token,
        left: &ASTNode,
        right: &ASTNode,
    ) -> InterpretResult<BuiltinNumTypes> {
        let left_value = match self.visit(left)? {
            Some(BuiltinNumTypes::F32(v)) => v,
            Some(BuiltinNumTypes::I32(v)) => v as f32,
            None => {
                return Err(InterpretError::MissingBinaryOperand {
                    side: BinaryOperandSide::Left,
                })
            }
        };

        let right_value = match self.visit(right)? {
            Some(BuiltinNumTypes::F32(v)) => v,
            Some(BuiltinNumTypes::I32(v)) => v as f32,
            None => {
                return Err(InterpretError::MissingBinaryOperand {
                    side: BinaryOperandSide::Right,
                })
            }
        };

        match op {
            Token::Plus => Ok(BuiltinNumTypes::F32(left_value + right_value)),
            Token::Minus => Ok(BuiltinNumTypes::F32(left_value - right_value)),
            Token::Asterisk => Ok(BuiltinNumTypes::F32(left_value * right_value)),
            Token::FloatDiv => Ok(BuiltinNumTypes::F32(left_value / right_value)),
            Token::IntegerDiv => Ok(BuiltinNumTypes::F32(
                ((left_value as i32) / (right_value as i32)) as f32,
            )),
            _ => Err(InterpretError::InvalidBinaryOperator { token: op.clone() }),
        }
    }

    fn visit_assign_node(&mut self, left: &ASTNode, right: &ASTNode) -> InterpretResult<()> {
        let ASTNode::Var { name, .. } = left else {
            return Err(InterpretError::AssignTargetMustBeVar);
        };

        self.symtab
            .lookup(name)
            .ok_or_else(|| InterpretError::UndefinedVariable { name: name.clone() })?;

        let res = self.visit(right)?;

        let Some(right_hand_value) = res else {
            return Err(InterpretError::MissingAssignmentValue { name: name.clone() });
        };

        self.global_memory.insert(name.to_owned(), right_hand_value);

        Ok(())
    }

    fn visit_var_node(&mut self, name: &String) -> InterpretResult<BuiltinNumTypes> {
        self.symtab
            .lookup(name)
            .ok_or_else(|| InterpretError::UndefinedVariable { name: name.clone() })?;

        self.global_memory
            .get(name)
            .cloned()
            .ok_or_else(|| InterpretError::UninitializedVariable { name: name.clone() })
    }

    fn visit_compound_node(&mut self, children: &Vec<Box<ASTNode>>) -> InterpretResult<()> {
        for child in children {
            self.visit(child)?;
        }
        Ok(())
    }
}
