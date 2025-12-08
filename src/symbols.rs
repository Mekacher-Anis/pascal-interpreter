use core::fmt;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
}

#[derive(Debug)]
pub enum SymbolKind {
    BuiltinType(BuiltinTypes),
    Variable { type_name: String },
}

#[derive(Debug)]
pub enum BuiltinTypes {
    Integer,
    Real,
}

impl BuiltinTypes {
    const fn as_str(self) -> &'static str {
        match self {
            BuiltinTypes::Integer => "INTEGER",
            BuiltinTypes::Real => "REAL",
        }
    }
}

impl fmt::Display for BuiltinTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltinTypes::Integer => write!(f, "INTEGER"),
            BuiltinTypes::Real => write!(f, "REAL"),
        }
    }
}

pub struct SymbolTable {
    table: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = SymbolTable {
            table: HashMap::new(),
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

    pub fn lookup(&self, name: &String) -> Option<&Symbol> {
        self.table.get(name)
    }
}

impl fmt::Display for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rows: Vec<(String, String)> = vec![];
        for (name, symbol) in &self.table {
            let desc = match &symbol.kind {
                SymbolKind::BuiltinType(builtin_type) => format!("BuiltinType({builtin_type})"),
                SymbolKind::Variable { type_name } => {
                    format!("Variable of type {}", type_name)
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
