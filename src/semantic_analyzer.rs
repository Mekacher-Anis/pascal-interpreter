use std::cell::RefCell;
use std::rc::Rc;

use anyhow::anyhow;

use crate::ast::ASTNode;
use crate::interpreter::{InterpretError, InterpretResult};
use crate::symbols::{ScopedSymbolTable, Symbol, SymbolKind};

pub struct SemanticAnalyzer {
    pub current_scope: Rc<RefCell<ScopedSymbolTable>>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            current_scope: Rc::new(RefCell::new(ScopedSymbolTable::new(
                "0".to_string(),
                0,
                None,
            ))),
        }
    }

    pub fn analyze(&mut self, node: &ASTNode) -> InterpretResult<()> {
        self.visit(node)
    }

    pub fn into_symbol_table(self) -> Result<ScopedSymbolTable, anyhow::Error> {
        match Rc::try_unwrap(self.current_scope) {
            Ok(v) => Ok(v.into_inner()),
            Err(_) => Err(anyhow!("No current scope available")),
        }
    }

    fn visit(&mut self, node: &ASTNode) -> InterpretResult<()> {
        match node {
            ASTNode::Program { block, .. } => self.visit_program_node(block),
            ASTNode::Block {
                declarations,
                compound_statement,
            } => self.visit_block_node(declarations, compound_statement),
            ASTNode::ProcedureDecl {
                proc_name,
                params,
                block_node,
            } => self.visit_procedure_decl_node(proc_name, params, block_node),
            ASTNode::VarDecl {
                var_node,
                type_node,
            } => self.visit_var_decl_node(var_node, type_node),
            ASTNode::Type { .. } => Ok(()),
            ASTNode::Compound { children } => self.visit_compound_node(children),
            ASTNode::Assign { left, right, .. } => self.visit_assign_node(left, right),
            ASTNode::Var { name } => self.visit_var_node(name),
            ASTNode::NoOp => Ok(()),
            ASTNode::UnaryOpNode { expr, .. } => self.visit(expr),
            ASTNode::BinOpNode { left, right, .. } => {
                self.visit(left)?;
                self.visit(right)
            }
            ASTNode::NumNode { .. } => Ok(()),
            ASTNode::Param {
                var_node,
                type_node,
            } => todo!(),
        }
    }

    fn visit_program_node(&mut self, block: &Box<ASTNode>) -> InterpretResult<()> {
        self.enter_scope("global");
        let res = self.visit(block);
        self.exit_scope();
        res
    }

    fn visit_block_node(
        &mut self,
        declarations: &Vec<Box<ASTNode>>,
        compound_statement: &Box<ASTNode>,
    ) -> InterpretResult<()> {
        for declaration in declarations {
            self.visit(declaration)?;
        }
        self.visit(compound_statement)
    }

    fn visit_compound_node(&mut self, children: &Vec<Box<ASTNode>>) -> InterpretResult<()> {
        for child in children {
            self.visit(child)?;
        }
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

        self.lookup_symbol(type_name, false)
            .ok_or_else(|| InterpretError::UndefinedType {
                type_name: type_name.clone(),
                var_name: var_name.clone(),
            })?;

        if let Some(_) = self.lookup_symbol(var_name, true) {
            return Err(InterpretError::SymbolAlreadyDefined {
                name: var_name.to_string(),
            });
        }

        let symbol = Symbol {
            name: var_name.clone(),
            kind: SymbolKind::Variable {
                type_name: type_name.to_owned(),
            },
        };

        self.define_symbol(symbol);

        Ok(())
    }

    fn visit_procedure_decl_node(
        &mut self,
        procedure_name: &str,
        params: &[Box<ASTNode>],
        block: &ASTNode,
    ) -> InterpretResult<()> {
        let param_names = params
            .iter()
            .map(|node| {
                let ASTNode::Param { var_node, .. } = &**node else {
                    return Err(InterpretError::InvalidVarDeclVarNode);
                };
                let ASTNode::Var { name } = &**var_node else {
                    return Err(InterpretError::AssignTargetMustBeVar);
                };
                Ok(name.clone())
            })
            .collect::<Result<Vec<_>, _>>()?;

        let proc_symbol = Symbol {
            name: procedure_name.to_string(),
            kind: SymbolKind::Procedure { param_names },
        };

        self.define_symbol(proc_symbol);

        self.enter_scope(procedure_name);

        params
            .iter()
            .map(|node| {
                let ASTNode::Param {
                    var_node,
                    type_node,
                } = &**node
                else {
                    return Err(InterpretError::InvalidVarDeclVarNode);
                };
                let ASTNode::Var { name } = &**var_node else {
                    return Err(InterpretError::InvalidVarDeclVarNode);
                };
                let ASTNode::Type {
                    value: type_name, ..
                } = &**type_node
                else {
                    return Err(InterpretError::InvalidVarDeclTypeNode);
                };

                let param_symbol = Symbol {
                    name: name.to_string(),
                    kind: SymbolKind::Variable {
                        type_name: type_name.to_string(),
                    },
                };

                self.define_symbol(param_symbol);

                Ok(())
            })
            .collect::<Result<Vec<_>, _>>()?;

        let res = self.visit(block);

        self.exit_scope();

        res
    }

    fn visit_assign_node(&mut self, left: &ASTNode, right: &ASTNode) -> InterpretResult<()> {
        let ASTNode::Var { .. } = left else {
            return Err(InterpretError::AssignTargetMustBeVar);
        };

        self.visit(left)?;

        self.visit(right)
    }

    fn visit_var_node(&self, name: &String) -> InterpretResult<()> {
        if self.lookup_symbol(name, false).is_none() {
            return Err(InterpretError::UndefinedVariable { name: name.clone() });
        }
        Ok(())
    }

    pub fn enter_scope(&mut self, scope_name: &str) {
        let scope_level = self.current_scope.borrow().scope_level + 1;

        let new_scope = Rc::new(RefCell::new(ScopedSymbolTable::new(
            scope_name.to_string(),
            scope_level,
            Some(Rc::clone(&self.current_scope)),
        )));

        self.current_scope = new_scope;
    }

    pub fn exit_scope(&mut self) {
        // println!("Exiting Scope:\n{}", self.current_scope.borrow());

        let parent = self
            .current_scope
            .borrow()
            .enclosing_scope
            .as_ref()
            .map(|p| Rc::clone(p));

        if let Some(parent) = parent {
            self.current_scope = parent;
        }
    }

    pub fn define_symbol(&mut self, symbol: Symbol) {
        self.current_scope.borrow_mut().define(symbol);
    }

    pub fn lookup_symbol(&self, name: &str, current_scope_only: bool) -> Option<Symbol> {
        // Look in current scope
        if let Some(sym) = self.current_scope.borrow().lookup(name, current_scope_only) {
            return Some(sym);
        }
        None
    }
}
