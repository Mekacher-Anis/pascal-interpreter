use std::cell::RefCell;
use std::fmt;
use std::iter::zip;
use std::rc::Rc;

use crate::ast::{ASTNode, BuiltinNumTypes};
use crate::call_stack::{ARType, ActivationRecord, CallStack};
use crate::symbols::{Symbol, SymbolKind};
use crate::token::Token;

pub type InterpretResult<T> = std::result::Result<T, InterpretError>;

#[derive(Debug, Clone)]
pub enum InterpretError {
    SymbolAlreadyDefined {
        name: String,
    },
    InvalidVarDeclVarNode,
    InvalidVarDeclTypeNode,
    UndefinedType {
        type_name: String,
        var_name: String,
    },
    AssignTargetMustBeVar,
    UndefinedVariable {
        name: String,
    },
    UndefinedFunction {
        name: String,
    },
    ProcCallMissingArgs {
        proc_name: String,
        expected: usize,
        got: usize,
    },
    UninitializedVariable {
        name: String,
    },
    MissingUnaryOperand,
    InvalidUnaryOperator {
        token: Token,
    },
    MissingBinaryOperand {
        side: BinaryOperandSide,
    },
    InvalidBinaryOperator {
        token: Token,
    },
    MissingAssignmentValue {
        name: String,
    },
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
            InterpretError::SymbolAlreadyDefined { name } => {
                write!(f, "Symbol '{name}' is already defined")
            }
            InterpretError::UndefinedFunction { name } => {
                write!(f, "Trying to call an undefined function '{name}'")
            }
            InterpretError::ProcCallMissingArgs {
                proc_name,
                expected,
                got,
            } => {
                write!(
                    f,
                    "Function {} expects {} arguments but got {}",
                    proc_name, expected, got
                )
            }
        }
    }
}

impl std::error::Error for InterpretError {}

pub struct Interpreter {
    log_call_stack: bool,
    call_stack: CallStack,
}

impl Interpreter {
    pub fn new(log_call_stack: bool) -> Self {
        Interpreter {
            log_call_stack: log_call_stack,
            call_stack: CallStack::new(),
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
                proc_name,
                params,
                block_node,
            } => {
                self.visit_procedure_decl_node(proc_name, params, block_node)?;
                Ok(None)
            }
            ASTNode::Param { .. } => Ok(None),
            ASTNode::ProcedureCall {
                proc_name,
                arguments,
                proc_symbol,
            } => self.visit_procedure_call_node(proc_name, arguments, proc_symbol),
        }
    }

    fn log(&self) {
        if self.log_call_stack {
            println!("{}", self.call_stack);
        }
    }

    fn visit_program_node(
        &mut self,
        name: &String,
        block: &Box<ASTNode>,
    ) -> InterpretResult<Option<BuiltinNumTypes>> {
        let ar = Rc::new(RefCell::new(ActivationRecord::new(
            &name,
            ARType::Program,
            1,
        )));
        self.call_stack.push(ar);
        self.log();
        let res = self.visit(block);

        self.call_stack.pop();
        res
    }

    fn visit_block_node(
        &mut self,
        declarations: &Vec<Box<ASTNode>>,
        compound_statement: &Box<ASTNode>,
    ) -> InterpretResult<Option<BuiltinNumTypes>> {
        for d in declarations {
            self.visit(d)?;
        }

        self.visit(compound_statement)
    }

    fn visit_var_decl_node(
        &mut self,
        _var_node: &Box<ASTNode>,
        _type_node: &Box<ASTNode>,
    ) -> InterpretResult<()> {
        Ok(())
    }

    fn visit_procedure_decl_node(
        &mut self,
        _procedure_name: &String,
        _params: &Vec<Box<ASTNode>>,
        _block: &Box<ASTNode>,
    ) -> InterpretResult<()> {
        Ok(())
    }

    fn visit_procedure_call_node(
        &mut self,
        proc_name: &str,
        arguments: &Vec<Box<ASTNode>>,
        proc_symbol: &RefCell<Option<Box<Symbol>>>,
    ) -> InterpretResult<Option<BuiltinNumTypes>> {
        let current_nesting_level = self.call_stack.peek().unwrap().borrow().nesting_level();

        let ar = Rc::new(RefCell::new(ActivationRecord::new(
            &proc_name,
            ARType::Procedure,
            current_nesting_level + 1,
        )));
        self.call_stack.push(ar);

        let Some(symbol_ptr) = proc_symbol.borrow().clone() else {
            return Err(InterpretError::UndefinedFunction {
                name: proc_name.to_string(),
            });
        };

        let Symbol {
            kind:
                SymbolKind::Procedure {
                    param_names,
                    block: block_node,
                },
            ..
        } = symbol_ptr.as_ref()
        else {
            return Err(InterpretError::UndefinedFunction {
                name: proc_name.to_string(),
            });
        };

        for (param, arg) in zip(param_names, arguments) {
            let value = self
                .visit(arg)?
                .ok_or(InterpretError::AssignTargetMustBeVar)?;
            self.call_stack
                .peek()
                .unwrap()
                .borrow_mut()
                .set(param, value);
        }

        let res = self.visit(&block_node);

        self.log();

        self.call_stack.pop();

        res
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

        let res = self.visit(right)?;

        let Some(right_hand_value) = res else {
            return Err(InterpretError::MissingAssignmentValue { name: name.clone() });
        };

        self.call_stack
            .peek()
            .unwrap()
            .borrow_mut()
            .set(name, right_hand_value);

        Ok(())
    }

    fn visit_var_node(&mut self, name: &String) -> InterpretResult<BuiltinNumTypes> {
        self.call_stack
            .peek()
            .unwrap()
            .borrow()
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
