use std::collections::HashMap;

use crate::ast::ASTNode;
use crate::token::Token;
use anyhow::{anyhow, Ok, Result};

pub struct Interpreter {
    pub variables: HashMap<String, i32>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            variables: HashMap::new(),
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

    pub fn interpret(&mut self, node: &ASTNode) -> Result<i32> {
        self.visit(node)
    }

    pub fn visit(&mut self, node: &ASTNode) -> Result<i32> {
        match node {
            ASTNode::NumNode { value, .. } => self.visit_num_node(*value),
            ASTNode::UnaryOpNode { expr, token } => self.visit_unary_op_node(token, expr),
            ASTNode::BinOpNode { left, right, op } => self.visit_bin_op_node(op, left, right),
            ASTNode::Assign { left, right, .. } => self.visit_assign_node(left, right),
            ASTNode::Var { name: value, .. } => self.visit_var_node(value),
            ASTNode::Compound { children } => self.visit_compound_node(children),
            ASTNode::NoOp => Ok(0),
        }
    }

    fn visit_num_node(&self, value: i32) -> Result<i32> {
        Ok(value)
    }

    fn visit_unary_op_node(&mut self, token: &Token, expr: &ASTNode) -> Result<i32> {
        let value = self.visit(expr)?;
        match token {
            Token::Plus => Ok(value),
            Token::Minus => Ok(-value),
            _ => Err(anyhow!("Invalid operator")),
        }
    }

    fn visit_bin_op_node(&mut self, op: &Token, left: &ASTNode, right: &ASTNode) -> Result<i32> {
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

    fn visit_assign_node(&mut self, left: &ASTNode, right: &ASTNode) -> Result<i32> {
        let ASTNode::Var { name, .. } = left else {
            return Err(anyhow!(
                "Left hand-side of assignment needs to be a variable"
            ));
        };

        self.visit(left)?;

        if self.variables.contains_key(name) == false {
            return Err(anyhow!("Assignment to undefined variable '{}'", name));
        }

        let right_hand_value = self.visit(right)?;

        self.variables.insert(name.to_owned(), right_hand_value);

        Ok(right_hand_value)
    }

    fn visit_var_node(&mut self, name: &String) -> Result<i32> {
        if self.variables.contains_key(name) {
            Ok(*self.variables.get(name).unwrap())
        } else {
            self.variables.insert(name.to_owned(), 0);
            Ok(0)
        }
    }

    fn visit_compound_node(&mut self, children: &Vec<Box<ASTNode>>) -> Result<i32> {
        for child in children {
            self.visit(child)?;
        }
        Ok(0)
    }
}
