use crate::eval::EvalError;
use crate::value::{Environment, Value};
use std::cell::RefCell;
use std::rc::Rc;

pub fn register_builtins(env: &mut Environment) {
    env.define("print".into(), true, Value::BuiltinFn("print".into()));
    env.define("println".into(), true, Value::BuiltinFn("println".into()));
    env.define("len".into(), true, Value::BuiltinFn("len".into()));
    env.define("push".into(), true, Value::BuiltinFn("push".into()));
    env.define("pop".into(), true, Value::BuiltinFn("pop".into()));
    env.define("typeof".into(), true, Value::BuiltinFn("typeof".into()));
    env.define("str".into(), true, Value::BuiltinFn("str".into()));
    env.define("int".into(), true, Value::BuiltinFn("int".into()));
    env.define("float".into(), true, Value::BuiltinFn("float".into()));
    env.define("abs".into(), true, Value::BuiltinFn("abs".into()));
    env.define("min".into(), true, Value::BuiltinFn("min".into()));
    env.define("max".into(), true, Value::BuiltinFn("max".into()));
    env.define("map".into(), true, Value::BuiltinFn("map".into()));
    env.define("filter".into(), true, Value::BuiltinFn("filter".into()));
    env.define("reduce".into(), true, Value::BuiltinFn("reduce".into()));
    env.define("range".into(), true, Value::BuiltinFn("range".into()));
    env.define("head".into(), true, Value::BuiltinFn("head".into()));
    env.define("tail".into(), true, Value::BuiltinFn("tail".into()));
}

impl crate::eval::Evaluator {
    pub fn call_builtin(&mut self, name: &str, args: &[Value]) -> Result<Value, EvalError> {
        match name {
            "print" => {
                let output = args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
                print!("{output}");
                Ok(Value::Null)
            }
            "println" => {
                let output = args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
                println!("{output}");
                Ok(Value::Null)
            }
            "len" => {
                match &args[0] {
                    Value::String(s) => Ok(Value::Int(s.len() as i64)),
                    Value::List(l) => Ok(Value::Int(l.borrow().len() as i64)),
                    _ => Err(EvalError::Runtime(format!("len() requires string or list, got {}", args[0].type_name()))),
                }
            }
            "push" => {
                match &args[0] {
                    Value::List(l) => {
                        l.borrow_mut().push(args[1].clone());
                        Ok(args[0].clone())
                    }
                    _ => Err(EvalError::Runtime(format!("push() requires list, got {}", args[0].type_name()))),
                }
            }
            "pop" => {
                match &args[0] {
                    Value::List(l) => {
                        l.borrow_mut().pop().ok_or_else(|| EvalError::Runtime("Cannot pop from empty list".into()))
                    }
                    _ => Err(EvalError::Runtime(format!("pop() requires list, got {}", args[0].type_name()))),
                }
            }
            "typeof" => Ok(Value::String(args[0].type_name().into())),
            "str" => Ok(Value::String(args[0].to_string())),
            "int" => match &args[0] {
                Value::Int(n) => Ok(Value::Int(*n)),
                Value::Float(n) => Ok(Value::Int(*n as i64)),
                Value::String(s) => s.parse::<i64>()
                    .map(Value::Int)
                    .map_err(|_| EvalError::Runtime(format!("Cannot convert '{s}' to int"))),
                _ => Err(EvalError::Runtime(format!("Cannot convert {} to int", args[0].type_name()))),
            },
            "float" => match &args[0] {
                Value::Int(n) => Ok(Value::Float(*n as f64)),
                Value::Float(n) => Ok(Value::Float(*n)),
                Value::String(s) => s.parse::<f64>()
                    .map(Value::Float)
                    .map_err(|_| EvalError::Runtime(format!("Cannot convert '{s}' to float"))),
                _ => Err(EvalError::Runtime(format!("Cannot convert {} to float", args[0].type_name()))),
            },
            "abs" => match &args[0] {
                Value::Int(n) => Ok(Value::Int(n.abs())),
                Value::Float(n) => Ok(Value::Float(n.abs())),
                _ => Err(EvalError::Runtime(format!("abs() requires number, got {}", args[0].type_name()))),
            },
            "min" => match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.min(b))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
                _ => Err(EvalError::Runtime("min() requires two numbers".into())),
            },
            "max" => match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.max(b))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
                _ => Err(EvalError::Runtime("max() requires two numbers".into())),
            },
            "head" => match &args[0] {
                Value::List(l) => {
                    let l = l.borrow();
                    if l.is_empty() {
                        Err(EvalError::Runtime("head of empty list".into()))
                    } else {
                        Ok(l[0].clone())
                    }
                }
                _ => Err(EvalError::Runtime(format!("head() requires list, got {}", args[0].type_name()))),
            },
            "tail" => match &args[0] {
                Value::List(l) => {
                    let l = l.borrow();
                    if l.is_empty() {
                        Ok(Value::List(Rc::new(RefCell::new(Vec::new()))))
                    } else {
                        Ok(Value::List(Rc::new(RefCell::new(l[1..].to_vec()))))
                    }
                }
                _ => Err(EvalError::Runtime(format!("tail() requires list, got {}", args[0].type_name()))),
            },
            "range" => {
                let start = match &args[0] {
                    Value::Int(n) => *n,
                    _ => return Err(EvalError::Runtime("range() requires integers".into())),
                };
                let end = match &args[1] {
                    Value::Int(n) => *n,
                    _ => return Err(EvalError::Runtime("range() requires integers".into())),
                };
                let mut v = Vec::new();
                for i in start..end {
                    v.push(Value::Int(i));
                }
                Ok(Value::List(Rc::new(RefCell::new(v))))
            }
            "map" => {
                Err(EvalError::Runtime("Use list comprehension syntax instead of map()".into()))
            }
            "filter" => {
                Err(EvalError::Runtime("Use list comprehension syntax instead of filter()".into()))
            }
            "reduce" => {
                Err(EvalError::Runtime("Use for loops instead of reduce()".into()))
            }
            _ if name.starts_with("__ctor_") => {
                let variant_name = name.strip_prefix("__ctor_").unwrap();
                let registry = self.type_registry.borrow();
                let info = registry.iter()
                    .find(|v| v.variant_name == variant_name)
                    .ok_or_else(|| EvalError::Runtime(format!("Unknown variant {variant_name}")))?;
                let type_name = info.type_name.clone();
                let arity = info.arity;
                if args.len() != arity {
                    return Err(EvalError::Runtime(format!("{}() expects {} arguments, got {}", variant_name, arity, args.len())));
                }
                Ok(Value::Variant {
                    type_name,
                    name: variant_name.to_string(),
                    fields: args.to_vec(),
                })
            }
            _ => Err(EvalError::Runtime(format!("Unknown builtin function: {name}"))),
        }
    }
}