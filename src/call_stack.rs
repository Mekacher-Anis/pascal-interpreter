use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::ast::BuiltinNumTypes;

pub enum ARType {
    Program,
    Procedure,
}

impl fmt::Display for ARType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ARType::Program => write!(f, "PRORGRAM"),
            ARType::Procedure => write!(f, "PROCEDURE"),
        }
    }
}

pub struct ActivationRecord {
    name: String,
    ar_type: ARType,
    nesting_level: usize,
    members: HashMap<String, BuiltinNumTypes>,
}

impl ActivationRecord {
    pub fn new(name: &str, ar_type: ARType, nesting_level: usize) -> Self {
        ActivationRecord {
            name: name.to_string(),
            ar_type: ar_type,
            nesting_level: nesting_level,
            members: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: &str, value: BuiltinNumTypes) {
        self.members.insert(name.to_owned(), value);
    }

    pub fn get(&self, name: &str) -> Option<&BuiltinNumTypes> {
        self.members.get(name)
    }

    pub fn nesting_level(&self) -> usize {
        self.nesting_level
    }
}

impl fmt::Display for ActivationRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} (level {})", self.name, self.nesting_level)?;
        writeln!(f, "Type: {}", self.ar_type)?;
        writeln!(f, "Members:")?;

        // deterministic ordering for printing
        let mut keys: Vec<&String> = self.members.keys().collect();
        keys.sort();
        for k in keys {
            let v = &self.members[k];
            writeln!(f, "  {} = {:?}", k, v)?;
        }
        Ok(())
    }
}

pub struct CallStack {
    stack: Vec<Rc<RefCell<ActivationRecord>>>,
}

impl CallStack {
    pub fn new() -> Self {
        CallStack { stack: vec![] }
    }

    pub fn push(&mut self, ar: Rc<RefCell<ActivationRecord>>) {
        self.stack.push(ar);
    }

    pub fn pop(&mut self) -> Option<Rc<RefCell<ActivationRecord>>> {
        self.stack.pop()
    }

    pub fn peek(&self) -> Option<&Rc<RefCell<ActivationRecord>>> {
        self.stack.last()
    }
}

impl fmt::Display for CallStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CALL STACK (size: {})", self.stack.len())?;
        // print top-most frame last in a visually conventional way (top of stack at end)
        for ar_rc in self.stack.iter().rev() {
            let ar = ar_rc.borrow();
            writeln!(f, "{}", &*ar)?;
        }
        Ok(())
    }
}
