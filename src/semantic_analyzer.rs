use crate::ast::ASTNode;
use crate::interpreter::{InterpretError, InterpretResult};
use crate::symbols::{Symbol, SymbolKind, SymbolTable};

pub struct SemanticAnalyzer {
    pub symtab: SymbolTable,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symtab: SymbolTable::new(),
        }
    }

    pub fn analyze(&mut self, node: &ASTNode) -> InterpretResult<()> {
        self.visit(node)
    }

    pub fn into_symbol_table(self) -> SymbolTable {
        self.symtab
    }

    fn visit(&mut self, node: &ASTNode) -> InterpretResult<()> {
        match node {
            ASTNode::Program { block, .. } => self.visit(block),
            ASTNode::Block {
                declarations,
                compound_statement,
            } => self.visit_block_node(declarations, compound_statement),
            ASTNode::ProcedureDecl {
                proc_name,
                block_node,
            } => self.visit_procedure_decl_node(proc_name, block_node),
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
        }
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

        self.symtab
            .lookup(type_name)
            .ok_or_else(|| InterpretError::UndefinedType {
                type_name: type_name.clone(),
                var_name: var_name.clone(),
            })?;

        if let Some(_) = self.symtab.lookup(var_name) {
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

        self.symtab.define(symbol);

        Ok(())
    }

    fn visit_procedure_decl_node(
        &mut self,
        _procedure_name: &String,
        _block: &Box<ASTNode>,
    ) -> InterpretResult<()> {
        Ok(())
    }

    fn visit_assign_node(&mut self, left: &ASTNode, right: &ASTNode) -> InterpretResult<()> {
        let ASTNode::Var { .. } = left else {
            return Err(InterpretError::AssignTargetMustBeVar);
        };

        self.visit(left)?;

        self.visit(right)
    }

    fn visit_var_node(&self, name: &String) -> InterpretResult<()> {
        if self.symtab.lookup(name).is_none() {
            return Err(InterpretError::UndefinedVariable { name: name.clone() });
        }
        Ok(())
    }
}
