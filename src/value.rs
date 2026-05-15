use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

use crate::ast::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Rc<RefCell<Vec<Value>>>),
    BuiltinFn(String),
    Fn {
        name: Option<String>,
        params: Vec<String>,
        body: Vec<Expr>,
        closure: Rc<RefCell<Environment>>,
    },
    Variant {
        type_name: String,
        name: String,
        fields: Vec<Value>,
    },
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.borrow().is_empty(),
            Value::Variant { .. } => true,
            Value::BuiltinFn(_) => true,
            Value::Fn { .. } => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::List(_) => "list",
            Value::BuiltinFn(_) => "function",
            Value::Fn { .. } => "function",
            Value::Variant { .. } => "variant",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::List(items) => {
                let items = items.borrow();
                let formatted: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", formatted.join(", "))
            }
            Value::BuiltinFn(name) => write!(f, "<builtin:{name}>"),
            Value::Fn { name, .. } => {
                match name {
                    Some(n) => write!(f, "<fn:{n}>"),
                    None => write!(f, "<fn>"),
                }
            }
            Value::Variant { name, fields, .. } => {
                if fields.is_empty() {
                    write!(f, "{name}")
                } else {
                    let formatted: Vec<String> = fields.iter().map(|v| v.to_string()).collect();
                    write!(f, "{}({})", name, formatted.join(", "))
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    vars: Vec<(String, bool, Value)>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            vars: Vec::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: String, mutable: bool, value: Value) {
        self.vars.push((name, mutable, value));
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        for (n, _, v) in self.vars.iter().rev() {
            if n == name {
                return Some(v.clone());
            }
        }
        if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: &str, value: Value) -> Result<(), String> {
        for (n, mutable, v) in self.vars.iter_mut() {
            if n == name {
                if *mutable {
                    *v = value;
                    return Ok(());
                } else {
                    return Err(format!("Cannot assign to immutable variable '{name}'"));
                }
            }
        }
        if let Some(parent) = &self.parent {
            parent.borrow_mut().set(name, value)
        } else {
            Err(format!("Undefined variable '{name}'"))
        }
    }
}