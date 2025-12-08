use std::collections::HashMap;

use crate::ast::{ASTNode, BuiltinNumTypes};
use crate::symbols::{Symbol, SymbolKind, SymbolTable};
use crate::token::Token;
use anyhow::{anyhow, Ok, Result};

pub struct Interpreter {
    pub variables: HashMap<String, BuiltinNumTypes>,
    pub symtab: SymbolTable,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            variables: HashMap::new(),
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
        if self.variables.is_empty() {
            println!("Variables: {{}} (no variables)");
            return;
        }

        println!("Variables:");
        let mut entries: Vec<_> = self.variables.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        for (k, v) in entries {
            println!("  {}: {}", k, v);
        }
    }

    pub fn interpret(&mut self, node: &ASTNode) -> Result<Option<BuiltinNumTypes>> {
        self.visit(node)
    }

    pub fn visit(&mut self, node: &ASTNode) -> Result<Option<BuiltinNumTypes>> {
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
        }
    }

    fn visit_program_node(&mut self, name: &String, block: &Box<ASTNode>) -> Result<()> {
        self.visit(&block)?;
        Ok(())
    }

    fn visit_block_node(
        &mut self,
        declarations: &Vec<Box<ASTNode>>,
        compound_statement: &Box<ASTNode>,
    ) -> Result<()> {
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
    ) -> Result<()> {
        let ASTNode::Var { name: var_name } = &**var_node else {
            return Err(anyhow!(
                "visit_var_decl_node expected var_node to be of type ASTNode::Var"
            ));
        };
        let ASTNode::Type {
            value: type_name, ..
        } = &**type_node
        else {
            return Err(anyhow!(
                "visit_var_decl_node expected type_node to be of type ASTNode::Var"
            ));
        };

        // make sure it's defined first
        self.symtab.lookup(type_name).ok_or(anyhow!(
            "Undefined type '{type_name}' used for variable '{var_name}'"
        ))?;

        let symbol = Symbol {
            name: var_name.clone(),
            kind: SymbolKind::Variable {
                type_name: type_name.to_owned(),
            },
        };

        self.symtab.define(symbol);

        Ok(())
    }

    fn visit_type_node(&self, value: &String) -> Result<()> {
        Ok(())
    }

    fn visit_num_node(&self, value: BuiltinNumTypes) -> Result<BuiltinNumTypes> {
        Ok(value)
    }

    fn visit_unary_op_node(&mut self, token: &Token, expr: &ASTNode) -> Result<BuiltinNumTypes> {
        let res: Option<BuiltinNumTypes> = self.visit(expr)?;
        let value;
        match res {
            Some(val) => match val {
                BuiltinNumTypes::F32(v) => value = v,
                BuiltinNumTypes::I32(v) => value = v as f32,
            },
            None => return Err(anyhow!("Expected a value for a unary op")),
        }

        match token {
            Token::Plus => Ok(BuiltinNumTypes::F32(value)),
            Token::Minus => Ok(BuiltinNumTypes::F32(-value)),
            _ => Err(anyhow!("Invalid operator")),
        }
    }

    fn visit_bin_op_node(
        &mut self,
        op: &Token,
        left: &ASTNode,
        right: &ASTNode,
    ) -> Result<BuiltinNumTypes> {
        let mut res = self.visit(left)?;
        let left_value;
        match res {
            Some(val) => match val {
                BuiltinNumTypes::F32(v) => left_value = v,
                BuiltinNumTypes::I32(v) => left_value = v as f32,
            },
            None => return Err(anyhow!("Expected a value of the left hand of a bin op")),
        }

        res = self.visit(right)?;
        let right_value;
        match res {
            Some(val) => match val {
                BuiltinNumTypes::F32(v) => right_value = v,
                BuiltinNumTypes::I32(v) => right_value = v as f32,
            },
            None => return Err(anyhow!("Expected a value of the left hand of a bin op")),
        }

        match op {
            Token::Plus => Ok(BuiltinNumTypes::F32(left_value + right_value)),
            Token::Minus => Ok(BuiltinNumTypes::F32(left_value - right_value)),
            Token::Asterisk => Ok(BuiltinNumTypes::F32(left_value * right_value)),
            Token::FloatDiv => Ok(BuiltinNumTypes::F32(left_value / right_value)),
            Token::IntegerDiv => Ok(BuiltinNumTypes::F32(
                ((left_value as i32) / (right_value as i32)) as f32,
            )),
            _ => Err(anyhow!("Invalid operator")),
        }
    }

    fn visit_assign_node(&mut self, left: &ASTNode, right: &ASTNode) -> Result<()> {
        let ASTNode::Var { name, .. } = left else {
            return Err(anyhow!(
                "Left hand-side of assignment needs to be a variable"
            ));
        };

        self.symtab
            .lookup(name)
            .ok_or(anyhow!("Assigning undefined variable {name}"))?;

        let res = self.visit(right)?;

        let Some(right_hand_value) = res else {
            return Err(anyhow!(
                "Expected right hand value for an assignment to be a valid value, got None"
            ));
        };

        self.variables.insert(name.to_owned(), right_hand_value);

        Ok(())
    }

    fn visit_var_node(&mut self, name: &String) -> Result<BuiltinNumTypes> {
        self.symtab
            .lookup(name)
            .ok_or(anyhow!("Accessing undefined variable {name}"))?;

        self.variables
            .get(name)
            .map(|v| v.clone())
            .ok_or(anyhow!("Accessing undefined variable \"{name}\""))
    }

    fn visit_compound_node(&mut self, children: &Vec<Box<ASTNode>>) -> Result<()> {
        for child in children {
            self.visit(child)?;
        }
        Ok(())
    }
}
