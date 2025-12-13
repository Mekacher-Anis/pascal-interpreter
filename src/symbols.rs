use core::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ast::ASTNode;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    BuiltinType(BuiltinTypes),
    Variable {
        type_name: String,
    },
    Procedure {
        param_names: Vec<String>,
        block: Box<ASTNode>,
    },
}

#[derive(Debug, Clone)]
pub enum BuiltinTypes {
    Integer,
    Real,
}

impl fmt::Display for BuiltinTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltinTypes::Integer => write!(f, "INTEGER"),
            BuiltinTypes::Real => write!(f, "REAL"),
        }
    }
}

pub struct ScopedSymbolTable {
    table: HashMap<String, Symbol>,
    scope_name: String,
    pub enclosing_scope: Option<Rc<RefCell<ScopedSymbolTable>>>,
    pub scope_level: u32,
}

impl ScopedSymbolTable {
    pub fn new(
        scope_name: String,
        scope_level: u32,
        enclosing_scope: Option<Rc<RefCell<ScopedSymbolTable>>>,
    ) -> Self {
        let mut table = ScopedSymbolTable {
            table: HashMap::new(),
            scope_name,
            enclosing_scope: enclosing_scope,
            scope_level,
        };
        table.init_builtins();
        table
    }

    fn init_builtins(&mut self) {
        self.define(Symbol {
            name: BuiltinTypes::Integer.to_string(),
            kind: SymbolKind::BuiltinType(BuiltinTypes::Integer),
        });
        self.define(Symbol {
            name: BuiltinTypes::Real.to_string(),
            kind: SymbolKind::BuiltinType(BuiltinTypes::Real),
        });
    }

    pub fn define(&mut self, symbol: Symbol) {
        self.table.insert(symbol.name.to_string(), symbol);
    }

    pub fn lookup(&self, name: &str, current_scope_only: bool) -> Option<Symbol> {
        // Look in current scope
        if let Some(sym) = self.table.get(name) {
            return Some(sym.clone());
        }

        if current_scope_only {
            return None;
        }

        // Look in parent scopes
        if let Some(scope) = self.enclosing_scope.as_ref().map(|s| Rc::clone(s)) {
            return scope.borrow().lookup(name, false);
        }

        None
    }
}

impl fmt::Display for ScopedSymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parent_name = self
            .enclosing_scope
            .as_ref()
            .map(|p| p.borrow().scope_name.clone())
            .unwrap_or_else(|| "None".to_string());
        let mut rows: Vec<(String, String)> = vec![];
        for (name, symbol) in &self.table {
            let desc = match &symbol.kind {
                SymbolKind::BuiltinType(builtin_type) => format!("BuiltinType({builtin_type})"),
                SymbolKind::Variable { type_name } => {
                    format!("Variable of type {}", type_name)
                }
                SymbolKind::Procedure { param_names, .. } => {
                    let params = param_names.join(", ");
                    format!("Procedure([{}])", params)
                }
            };
            rows.push((name.clone(), desc));
        }
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        let name_max = rows.iter().map(|(n, _)| n.len()).max().unwrap_or(4);
        let desc_max = rows.iter().map(|(_, d)| d.len()).max().unwrap_or(4);
        let name_width = name_max.max(4);
        let desc_width = desc_max.max(4);
        let name_col_width = name_width + 2;
        let desc_col_width = desc_width + 2;
        writeln!(
            f,
            "{} - {} Scope (Parent: {})",
            self.scope_level, self.scope_name, parent_name
        )?;
        writeln!(f, "+{:-<name_col_width$}+{:-<desc_col_width$}+", "", "")?;
        writeln!(f, "| {:<name_width$} | {:<desc_width$} |", "Name", "Type")?;
        writeln!(f, "+{:-<name_col_width$}+{:-<desc_col_width$}+", "", "")?;
        for (name, desc) in rows {
            writeln!(f, "| {:<name_width$} | {:<desc_width$} |", name, desc)?;
        }
        writeln!(f, "+{:-<name_col_width$}+{:-<desc_col_width$}+", "", "")?;
        Ok(())
    }
}
