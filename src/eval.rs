use std::rc::Rc;
use std::cell::RefCell;

use crate::ast::{Block, Expr, Pattern};
use crate::bibs;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::TokenKind;
use crate::value::{Environment, Value};

pub struct Evaluator {
    pub globals: Rc<RefCell<Environment>>,
    pub type_registry: Rc<RefCell<Vec<TypeVariantInfo>>>,
}

pub struct TypeVariantInfo {
    pub type_name: String,
    pub variant_name: String,
    pub arity: usize,
}

pub enum EvalError {
    Runtime(String),
    Return(Value),
}

impl From<String> for EvalError {
    fn from(s: String) -> Self {
        EvalError::Runtime(s)
    }
}

impl Evaluator {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        let ev = Self {
            globals: globals.clone(),
            type_registry: Rc::new(RefCell::new(Vec::new())),
        };
        crate::builtins::register_builtins(&mut *ev.globals.borrow_mut());
        ev
    }

    pub fn run(&mut self, source: &str) -> Result<Option<Value>, String> {
        let tokens = Lexer::new(source).tokenize()
            .map_err(|e| format!("Lexer error: {e}"))?;
        let ast = Parser::new(tokens).parse()
            .map_err(|e| format!("Parser error: {e}"))?;
        self.eval_block(&ast, &self.globals.clone())
            .map_err(|e| match e {
                EvalError::Runtime(msg) => msg,
                EvalError::Return(_) => "Unexpected return".into(),
            })
    }

    pub fn eval_block(&mut self, block: &Block, env: &Rc<RefCell<Environment>>) -> Result<Option<Value>, EvalError> {
        let mut result = None;
        for expr in block {
            result = Some(self.eval(expr, env)?);
        }
        Ok(result)
    }

    fn eval(&mut self, expr: &Expr, env: &Rc<RefCell<Environment>>) -> Result<Value, EvalError> {
        match expr {
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Float(n) => Ok(Value::Float(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Null => Ok(Value::Null),

            Expr::Ident(name) => {
                env.borrow().get(name).ok_or_else(|| EvalError::Runtime(format!("Undefined variable '{name}'")))
            }

            Expr::UnaryOp { op, right } => {
                let val = self.eval(right, env)?;
                match op {
                    TokenKind::Minus => match val {
                        Value::Int(n) => Ok(Value::Int(-n)),
                        Value::Float(n) => Ok(Value::Float(-n)),
                        _ => Err(EvalError::Runtime(format!("Cannot negate {}", val.type_name()))),
                    },
                    TokenKind::Not => match val {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        _ => Err(EvalError::Runtime(format!("Cannot negate {}", val.type_name()))),
                    },
                    _ => Err(EvalError::Runtime(format!("Unknown unary operator {op}"))),
                }
            }

            Expr::BinaryOp { left, op, right } => {
                let lv = self.eval(left, env)?;
                let rv = self.eval(right, env)?;
                self.eval_binary_op(&lv, op, &rv)
            }

            Expr::List(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.eval(e, env)?);
                }
                Ok(Value::List(Rc::new(RefCell::new(vals))))
            }

            Expr::Range { start, end } => {
                let sv = self.eval(start, env)?;
                let ev = self.eval(end, env)?;
                let s = match sv {
                    Value::Int(n) => n,
                    _ => return Err(EvalError::Runtime(format!("Range start must be int, got {}", sv.type_name()))),
                };
                let e = match ev {
                    Value::Int(n) => n,
                    _ => return Err(EvalError::Runtime(format!("Range end must be int, got {}", ev.type_name()))),
                };
                let mut vals = Vec::new();
                let mut i = s;
                if s <= e {
                    while i <= e {
                        vals.push(Value::Int(i));
                        i += 1;
                    }
                } else {
                    while i >= e {
                        vals.push(Value::Int(i));
                        i -= 1;
                    }
                }
                Ok(Value::List(Rc::new(RefCell::new(vals))))
            }

            Expr::RangeStep { start, step, end } => {
                let sv = self.eval(start, env)?;
                let stepv = self.eval(step, env)?;
                let ev = self.eval(end, env)?;
                let s = match sv {
                    Value::Int(n) => n,
                    _ => return Err(EvalError::Runtime(format!("Range start must be int, got {}", sv.type_name()))),
                };
                let st = match stepv {
                    Value::Int(n) => n,
                    _ => return Err(EvalError::Runtime(format!("Range step must be int, got {}", stepv.type_name()))),
                };
                let e = match ev {
                    Value::Int(n) => n,
                    _ => return Err(EvalError::Runtime(format!("Range end must be int, got {}", ev.type_name()))),
                };
                if st == 0 {
                    return Err(EvalError::Runtime("Range step cannot be zero".into()));
                }
                let mut vals = Vec::new();
                let mut i = s;
                if st > 0 {
                    while i <= e {
                        vals.push(Value::Int(i));
                        i += st;
                    }
                } else {
                    while i >= e {
                        vals.push(Value::Int(i));
                        i += st;
                    }
                }
                Ok(Value::List(Rc::new(RefCell::new(vals))))
            }

            Expr::Comprehension { expr, name, iterable, filter } => {
                let iter_val = self.eval(iterable, env)?;
                let items = match &iter_val {
                    Value::List(l) => l.borrow().clone(),
                    _ => return Err(EvalError::Runtime(format!("Cannot iterate over {}", iter_val.type_name()))),
                };
                let mut result = Vec::new();
                let scope = Rc::new(RefCell::new(Environment::with_parent(env.clone())));
                for item in items {
                    scope.borrow_mut().define(name.clone(), true, item.clone());
                    if let Some(f) = filter {
                        let cond = self.eval(f, &scope)?;
                        if !cond.is_truthy() { continue; }
                    }
                    result.push(self.eval(expr, &scope)?);
                }
                Ok(Value::List(Rc::new(RefCell::new(result))))
            }

            Expr::Index { object, index } => {
                let obj = self.eval(object, env)?;
                let idx = self.eval(index, env)?;
                match (&obj, &idx) {
                    (Value::List(items), Value::Int(n)) => {
                        let i = *n as usize;
                        let items = items.borrow();
                        if i < items.len() {
                            Ok(items[i].clone())
                        } else {
                            Err(EvalError::Runtime(format!("Index {n} out of bounds (len {})", items.len())))
                        }
                    }
                    (Value::String(s), Value::Int(n)) => {
                        let i = *n as usize;
                        if i < s.len() {
                            Ok(Value::String(s[i..i+1].to_string()))
                        } else {
                            Err(EvalError::Runtime(format!("Index {n} out of bounds (len {})", s.len())))
                        }
                    }
                    _ => Err(EvalError::Runtime(format!("Cannot index {} with {}", obj.type_name(), idx.type_name()))),
                }
            }

            Expr::Dot { object, field } => {
                let obj = self.eval(object, env)?;
                match &obj {
                    Value::Variant { fields, name, .. } => {
                        if let Ok(idx) = field.parse::<usize>() {
                            if idx < fields.len() {
                                return Ok(fields[idx].clone());
                            }
                        }
                        Err(EvalError::Runtime(format!("Variant {name} has no field '{field}'")))
                    }
                    Value::List(items) => {
                        match field.as_str() {
                            "len" => Ok(Value::Int(items.borrow().len() as i64)),
                            _ => Err(EvalError::Runtime(format!("List has no method '{field}'"))),
                        }
                    }
                    Value::String(s) => {
                        match field.as_str() {
                            "len" => Ok(Value::Int(s.len() as i64)),
                            _ => Err(EvalError::Runtime(format!("String has no method '{field}'"))),
                        }
                    }
                    _ => Err(EvalError::Runtime(format!("Cannot access field '{field}' on {}", obj.type_name()))),
                }
            }

            Expr::Let { name, value } => {
                let val = self.eval(value, env)?;
                env.borrow_mut().define(name.clone(), false, val.clone());
                Ok(val)
            }

            Expr::Var { name, value } => {
                let val = self.eval(value, env)?;
                env.borrow_mut().define(name.clone(), true, val.clone());
                Ok(val)
            }

            Expr::Assign { target, value } => {
                let val = self.eval(value, env)?;
                match target.as_ref() {
                    Expr::Ident(name) => {
                        env.borrow_mut().set(name, val.clone())?;
                        Ok(val)
                    }
                    Expr::Index { object, index } => {
                        let idx = self.eval(index, env)?;
                        let obj = self.eval(object, env)?;
                        match (&obj, &idx) {
                            (Value::List(items), Value::Int(n)) => {
                                let i = *n as usize;
                                let mut items = items.borrow_mut();
                                if i < items.len() {
                                    items[i] = val.clone();
                                    Ok(val)
                                } else {
                                    Err(EvalError::Runtime(format!("Index {n} out of bounds")))
                                }
                            }
                            _ => Err(EvalError::Runtime("Invalid assignment target".into())),
                        }
                    }
                    _ => Err(EvalError::Runtime("Invalid assignment target".into())),
                }
            }

            Expr::CompoundAssign { target, op, value } => {
                let current = match target.as_ref() {
                    Expr::Ident(name) => {
                        env.borrow().get(name).ok_or_else(|| EvalError::Runtime(format!("Undefined variable '{name}'")))?
                    }
                    _ => return Err(EvalError::Runtime("Invalid compound assignment target".into())),
                };
                let val = self.eval(value, env)?;
                let result = self.eval_binary_op(&current, op, &val)?;
                match target.as_ref() {
                    Expr::Ident(name) => {
                        env.borrow_mut().set(name, result.clone())?;
                        Ok(result)
                    }
                    _ => Err(EvalError::Runtime("Invalid compound assignment target".into())),
                }
            }

            Expr::If { condition, then_block, elif_clauses, else_block } => {
                if self.eval(condition, env)?.is_truthy() {
                    self.eval_block_expr(then_block, env)
                } else {
                    for (cond, block) in elif_clauses {
                        if self.eval(cond, env)?.is_truthy() {
                            return self.eval_block_expr(block, env);
                        }
                    }
                    if let Some(else_block) = else_block {
                        self.eval_block_expr(else_block, env)
                    } else {
                        Ok(Value::Null)
                    }
                }
            }

            Expr::Fn { name, params, body } => {
                let fn_val = Value::Fn {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: env.clone(),
                };
                if let Some(n) = name {
                    env.borrow_mut().define(n.clone(), true, fn_val.clone());
                }
                Ok(fn_val)
            }

            Expr::Call { callee, args } => {
                let func = self.eval(callee, env)?;
                self.call_function(func, args, env)
            }

            Expr::Return(value) => {
                let val = self.eval(value, env)?;
                Err(EvalError::Return(val))
            }

            Expr::Do(body) => {
                self.eval_block_expr(body, env)
            }

            Expr::While { condition, body } => {
                let mut result = Value::Null;
                while self.eval(condition, env)?.is_truthy() {
                    result = self.eval_block_expr(body, env)?;
                }
                Ok(result)
            }

            Expr::For { name, iterable, body } => {
                let iterable_val = self.eval(iterable, env)?;
                let items = match &iterable_val {
                    Value::List(l) => l.borrow().clone(),
                    _ => return Err(EvalError::Runtime(format!("Cannot iterate over {}", iterable_val.type_name()))),
                };
                let mut result = Value::Null;
                let scope = Rc::new(RefCell::new(Environment::with_parent(env.clone())));
                for item in items {
                    scope.borrow_mut().define(name.clone(), false, item);
                    result = self.eval_block_expr(body, &scope)?;
                }
                Ok(result)
            }

            Expr::Match { subject, arms } => {
                let val = self.eval(subject, env)?;
                for (pattern, body) in arms {
                    let match_env = Rc::new(RefCell::new(Environment::with_parent(env.clone())));
                    if self.match_pattern(pattern, &val, &match_env)? {
                        return self.eval_block_expr(body, &match_env);
                    }
                }
                Ok(Value::Null)
            }

            Expr::TypeDecl { name, variants } => {
                for variant in variants {
                    self.type_registry.borrow_mut().push(TypeVariantInfo {
                        type_name: name.clone(),
                        variant_name: variant.name.clone(),
                        arity: variant.fields.len(),
                    });

                    if variant.fields.is_empty() {
                        env.borrow_mut().define(
                            variant.name.clone(),
                            true,
                            Value::Variant {
                                type_name: name.clone(),
                                name: variant.name.clone(),
                                fields: Vec::new(),
                            },
                        );
                    } else {
                        env.borrow_mut().define(
                            variant.name.clone(),
                            true,
                            Value::BuiltinFn(format!("__ctor_{}", variant.name)),
                        );
                    }
                }
                Ok(Value::Null)
            }

            Expr::Load(name) => {
                let bindings = bibs::get_bib(name)
                    .ok_or_else(|| EvalError::Runtime(format!("Unknown bib: '{name}'")))?;
                for (key, val) in bindings {
                    env.borrow_mut().define(key.to_string(), true, val);
                }
                Ok(Value::Null)
            }
        }
    }

    fn eval_block_expr(&mut self, block: &Block, env: &Rc<RefCell<Environment>>) -> Result<Value, EvalError> {
        let scope = Rc::new(RefCell::new(Environment::with_parent(env.clone())));
        let mut result = Value::Null;
        for expr in block {
            result = self.eval(expr, &scope)?;
        }
        Ok(result)
    }

    fn match_pattern(&self, pattern: &Pattern, value: &Value, env: &Rc<RefCell<Environment>>) -> Result<bool, EvalError> {
        match pattern {
            Pattern::Wildcard => Ok(true),
            Pattern::Ident(name) => {
                env.borrow_mut().define(name.clone(), true, value.clone());
                Ok(true)
            }
            Pattern::Int(n) => Ok(matches!(value, Value::Int(v) if v == n)),
            Pattern::Float(n) => Ok(matches!(value, Value::Float(v) if v == n)),
            Pattern::String(s) => Ok(matches!(value, Value::String(v) if v == s)),
            Pattern::Bool(b) => Ok(matches!(value, Value::Bool(v) if v == b)),
            Pattern::Null => Ok(matches!(value, Value::Null)),
            Pattern::Constructor { name, bindings } => {
                match value {
                    Value::Variant { name: vname, fields, .. } => {
                        if name != vname || fields.len() != bindings.len() {
                            return Ok(false);
                        }
                        for (binding, field_val) in bindings.iter().zip(fields.iter()) {
                            env.borrow_mut().define(binding.clone(), true, field_val.clone());
                        }
                        Ok(true)
                    }
                    Value::Bool(b) if name == "true" => Ok(*b),
                    Value::Bool(b) if name == "false" => Ok(!b),
                    Value::Null if name == "null" => Ok(true),
                    _ => Ok(false),
                }
            }
            Pattern::List(pats) => {
                match value {
                    Value::List(items) => {
                        let items = items.borrow();
                        if items.len() != pats.len() {
                            return Ok(false);
                        }
                        for (pat, item) in pats.iter().zip(items.iter()) {
                            if !self.match_pattern(pat, item, env)? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            Pattern::Rest => Ok(true),
        }
    }

    pub fn call_function(&mut self, func: Value, args: &[Expr], env: &Rc<RefCell<Environment>>) -> Result<Value, EvalError> {
        match &func {
            Value::Fn { params, body, closure, .. } => {
                if args.len() != params.len() {
                    return Err(EvalError::Runtime(format!("Expected {} arguments, got {}", params.len(), args.len())));
                }
                let arg_vals: Vec<Value> = args.iter()
                    .map(|a| self.eval(a, env))
                    .collect::<Result<_, _>>()?;

                let call_env = Rc::new(RefCell::new(Environment::with_parent(closure.clone())));
                for (name, val) in params.iter().zip(arg_vals.iter()) {
                    call_env.borrow_mut().define(name.clone(), true, val.clone());
                }

                match self.eval_block_expr(body, &call_env) {
                    Ok(v) => Ok(v),
                    Err(EvalError::Return(v)) => Ok(v),
                    Err(e) => Err(e),
                }
            }
            Value::BuiltinFn(name) => {
                let arg_vals: Vec<Value> = args.iter()
                    .map(|a| self.eval(a, env))
                    .collect::<Result<_, _>>()?;
                self.call_builtin(name, &arg_vals)
            }
            _ => Err(EvalError::Runtime(format!("Cannot call {}", func.type_name()))),
        }
    }

    fn eval_binary_op(&self, left: &Value, op: &TokenKind, right: &Value) -> Result<Value, EvalError> {
        match op {
            TokenKind::Plus => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
                (Value::String(a), b) => Ok(Value::String(format!("{a}{b}"))),
                (a, Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
                _ => Err(EvalError::Runtime(format!("Cannot add {} and {}", left.type_name(), right.type_name()))),
            },
            TokenKind::Minus => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
                _ => Err(EvalError::Runtime(format!("Cannot subtract {} from {}", right.type_name(), left.type_name()))),
            },
            TokenKind::Star => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
                _ => Err(EvalError::Runtime(format!("Cannot multiply {} and {}", left.type_name(), right.type_name()))),
            },
            TokenKind::Slash => match (left, right) {
                (Value::Int(a), Value::Int(b)) => {
                    if *b == 0 { return Err(EvalError::Runtime("Division by zero".into())); }
                    Ok(Value::Int(a / b))
                }
                (Value::Float(a), Value::Float(b)) => {
                    if *b == 0.0 { return Err(EvalError::Runtime("Division by zero".into())); }
                    Ok(Value::Float(a / b))
                }
                (Value::Int(a), Value::Float(b)) => {
                    if *b == 0.0 { return Err(EvalError::Runtime("Division by zero".into())); }
                    Ok(Value::Float(*a as f64 / b))
                }
                (Value::Float(a), Value::Int(b)) => {
                    if *b == 0 { return Err(EvalError::Runtime("Division by zero".into())); }
                    Ok(Value::Float(a / *b as f64))
                }
                _ => Err(EvalError::Runtime(format!("Cannot divide {} by {}", left.type_name(), right.type_name()))),
            },
            TokenKind::Percent => match (left, right) {
                (Value::Int(a), Value::Int(b)) => {
                    if *b == 0 { return Err(EvalError::Runtime("Division by zero".into())); }
                    Ok(Value::Int(a % b))
                }
                _ => Err(EvalError::Runtime("Modulo requires integers".into())),
            },
            TokenKind::Eq => Ok(Value::Bool(left == right)),
            TokenKind::Neq => Ok(Value::Bool(left != right)),
            TokenKind::Lt => self.compare_values(left, right, |a, b| a < b),
            TokenKind::Gt => self.compare_values(left, right, |a, b| a > b),
            TokenKind::Lte => self.compare_values(left, right, |a, b| a <= b),
            TokenKind::Gte => self.compare_values(left, right, |a, b| a >= b),
            TokenKind::And => Ok(Value::Bool(left.is_truthy() && right.is_truthy())),
            TokenKind::Or => Ok(Value::Bool(left.is_truthy() || right.is_truthy())),
            TokenKind::PlusAssign | TokenKind::MinusAssign | TokenKind::StarAssign | TokenKind::SlashAssign => {
                let base_op = match op {
                    TokenKind::PlusAssign => TokenKind::Plus,
                    TokenKind::MinusAssign => TokenKind::Minus,
                    TokenKind::StarAssign => TokenKind::Star,
                    TokenKind::SlashAssign => TokenKind::Slash,
                    _ => unreachable!(),
                };
                self.eval_binary_op(left, &base_op, right)
            }
            _ => Err(EvalError::Runtime(format!("Unknown binary operator {op}"))),
        }
    }

    fn compare_values<F>(&self, left: &Value, right: &Value, cmp: F) -> Result<Value, EvalError>
    where F: Fn(f64, f64) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(cmp(*a as f64, *b as f64))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(cmp(*a, *b))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool(cmp(*a as f64, *b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(cmp(*a, *b as f64))),
            (Value::String(a), Value::String(b)) => {
                let ord = a.cmp(b);
                Ok(Value::Bool(cmp(0.0, if ord == std::cmp::Ordering::Less { -1.0 } else if ord == std::cmp::Ordering::Greater { 1.0 } else { 0.0 })))
            }
            _ => Err(EvalError::Runtime(format!("Cannot compare {} and {}", left.type_name(), right.type_name()))),
        }
    }
}