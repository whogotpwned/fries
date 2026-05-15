use crate::eval::EvalError;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn get_bib(name: &str) -> Option<Vec<(&str, Value)>> {
    match name {
        "json" => Some(json_bib()),
        _ => None,
    }
}

fn json_bib() -> Vec<(&'static str, Value)> {
    vec![
        ("json_parse", Value::BuiltinFn("json_parse".into())),
        ("json_stringify", Value::BuiltinFn("json_stringify".into())),
        ("json_keys", Value::BuiltinFn("json_keys".into())),
        ("json_values", Value::BuiltinFn("json_values".into())),
        ("json_has", Value::BuiltinFn("json_has".into())),
        ("json_get", Value::BuiltinFn("json_get".into())),
        ("json_set", Value::BuiltinFn("json_set".into())),
    ]
}

pub fn call_json_builtin(name: &str, args: &[Value]) -> Result<Value, EvalError> {
    match name {
        "json_parse" => {
            let s = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err(EvalError::Runtime("json_parse() requires a string".into())),
            };
            parse_json_value(&s)
        }
        "json_stringify" => {
            Ok(Value::String(stringify_json(&args[0])))
        }
        "json_keys" => {
            match &args[0] {
                Value::Dict(map) => {
                    let map = map.borrow();
                    Ok(Value::List(Rc::new(RefCell::new(
                        map.keys().map(|k| Value::String(k.clone())).collect()
                    ))))
                }
                _ => Err(EvalError::Runtime("json_keys() requires a dict".into())),
            }
        }
        "json_values" => {
            match &args[0] {
                Value::Dict(map) => {
                    let map = map.borrow();
                    Ok(Value::List(Rc::new(RefCell::new(map.values().cloned().collect()))))
                }
                _ => Err(EvalError::Runtime("json_values() requires a dict".into())),
            }
        }
        "json_has" => {
            match (&args[0], &args[1]) {
                (Value::Dict(map), Value::String(key)) => {
                    let map = map.borrow();
                    Ok(Value::Bool(map.contains_key(key)))
                }
                _ => Err(EvalError::Runtime("json_has() requires a dict and a string key".into())),
            }
        }
        "json_get" => {
            match (&args[0], &args[1]) {
                (Value::Dict(map), Value::String(key)) => {
                    let map = map.borrow();
                    Ok(map.get(key).cloned().unwrap_or(Value::Null))
                }
                _ => Err(EvalError::Runtime("json_get() requires a dict and a string key".into())),
            }
        }
        "json_set" => {
            match (&args[0], &args[1], &args[2]) {
                (Value::Dict(map), Value::String(key), val) => {
                    let mut new_map = map.borrow().clone();
                    new_map.insert(key.clone(), val.clone());
                    Ok(Value::Dict(Rc::new(RefCell::new(new_map))))
                }
                _ => Err(EvalError::Runtime("json_set() requires a dict, key, and value".into())),
            }
        }
        _ => Err(EvalError::Runtime(format!("Unknown json function: {name}"))),
    }
}

fn parse_json_value(input: &str) -> Result<Value, EvalError> {
    let trimmed = input.trim();
    let (val, remaining) = parse_json(trimmed)?;
    if remaining.trim().is_empty() {
        Ok(val)
    } else {
        Err(EvalError::Runtime(format!("Unexpected trailing content: {remaining}")))
    }
}

fn parse_json(input: &str) -> Result<(Value, &str), EvalError> {
    let input = input.trim();
    if input.starts_with('{') {
        parse_json_object(input)
    } else if input.starts_with('[') {
        parse_json_array(input)
    } else if input.starts_with('"') {
        parse_json_string(input)
    } else if input.starts_with("true") {
        Ok((Value::Bool(true), &input[4..]))
    } else if input.starts_with("false") {
        Ok((Value::Bool(false), &input[5..]))
    } else if input.starts_with("null") {
        Ok((Value::Null, &input[4..]))
    } else if input.starts_with('-') || input.starts_with(|c: char| c.is_ascii_digit()) {
        parse_json_number(input)
    } else {
        Err(EvalError::Runtime(format!("Invalid JSON: {input}")))
    }
}

fn parse_json_object(input: &str) -> Result<(Value, &str), EvalError> {
    let mut input = &input[1..];
    let mut map = HashMap::new();
    input = input.trim();
    if input.starts_with('}') {
        return Ok((Value::Dict(Rc::new(RefCell::new(map))), &input[1..]));
    }
    loop {
        input = input.trim();
        let (key, rest) = parse_json_string(input)?;
        let key_str = match key {
            Value::String(s) => s,
            _ => return Err(EvalError::Runtime("JSON object key must be a string".into())),
        };
        input = rest.trim();
        if !input.starts_with(':') {
            return Err(EvalError::Runtime("Expected ':' in JSON object".into()));
        }
        input = &input[1..];
        let (val, rest) = parse_json(input)?;
        map.insert(key_str, val);
        input = rest.trim();
        if input.starts_with(',') {
            input = &input[1..];
            continue;
        }
        if input.starts_with('}') {
            return Ok((Value::Dict(Rc::new(RefCell::new(map))), &input[1..]));
        }
        return Err(EvalError::Runtime("Expected ',' or '}' in JSON object".into()));
    }
}

fn parse_json_array(input: &str) -> Result<(Value, &str), EvalError> {
    let mut input = &input[1..];
    let mut items = Vec::new();
    input = input.trim();
    if input.starts_with(']') {
        return Ok((Value::List(Rc::new(RefCell::new(items))), &input[1..]));
    }
    loop {
        let (val, rest) = parse_json(input)?;
        items.push(val);
        input = rest.trim();
        if input.starts_with(',') {
            input = &input[1..];
            continue;
        }
        if input.starts_with(']') {
            return Ok((Value::List(Rc::new(RefCell::new(items))), &input[1..]));
        }
        return Err(EvalError::Runtime("Expected ',' or ']' in JSON array".into()));
    }
}

fn parse_json_string(input: &str) -> Result<(Value, &str), EvalError> {
    if !input.starts_with('"') {
        return Err(EvalError::Runtime("Expected '\"'".into()));
    }
    let mut chars = input[1..].chars();
    let mut s = String::new();
    loop {
        match chars.next() {
            Some('"') => return Ok((Value::String(s), chars.as_str())),
            Some('\\') => match chars.next() {
                Some('n') => s.push('\n'),
                Some('t') => s.push('\t'),
                Some('r') => s.push('\r'),
                Some('\\') => s.push('\\'),
                Some('"') => s.push('"'),
                Some('/') => s.push('/'),
                Some('u') => {
                    let hex: String = chars.by_ref().take(4).collect();
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| EvalError::Runtime("Invalid unicode escape".into()))?;
                    s.push(char::from_u32(code).unwrap_or('\0'));
                }
                Some(c) => s.push(c),
                None => return Err(EvalError::Runtime("Unterminated string escape".into())),
            },
            Some(c) => s.push(c),
            None => return Err(EvalError::Runtime("Unterminated string".into())),
        }
    }
}

fn parse_json_number(input: &str) -> Result<(Value, &str), EvalError> {
    let mut end = 0;
    let bytes = input.as_bytes();
    if end < bytes.len() && bytes[end] == b'-' { end += 1; }
    while end < bytes.len() && bytes[end].is_ascii_digit() { end += 1; }
    let is_float = end < bytes.len() && bytes[end] == b'.';
    if is_float {
        end += 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() { end += 1; }
    }
    if end < bytes.len() && (bytes[end] == b'e' || bytes[end] == b'E') {
        end += 1;
        if end < bytes.len() && (bytes[end] == b'+' || bytes[end] == b'-') { end += 1; }
        while end < bytes.len() && bytes[end].is_ascii_digit() { end += 1; }
    }
    let num_str = &input[..end];
    if is_float {
        let n: f64 = num_str.parse().map_err(|_| EvalError::Runtime(format!("Invalid number: {num_str}")))?;
        Ok((Value::Float(n), &input[end..]))
    } else {
        let n: i64 = num_str.parse().map_err(|_| EvalError::Runtime(format!("Invalid number: {num_str}")))?;
        Ok((Value::Int(n), &input[end..]))
    }
}

fn stringify_json(val: &Value) -> String {
    match val {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\t', "\\t")),
        Value::List(items) => {
            let items = items.borrow();
            let inner: Vec<String> = items.iter().map(stringify_json).collect();
            format!("[{}]", inner.join(", "))
        }
        Value::Dict(map) => {
            let map = map.borrow();
            let entries: Vec<String> = map.iter()
                .map(|(k, v)| format!("\"{k}\": {}", stringify_json(v)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
        Value::Variant { name, fields, .. } => {
            if fields.is_empty() {
                format!("\"{name}\"")
            } else {
                let inner: Vec<String> = fields.iter().map(stringify_json).collect();
                format!("{{\"name\": \"{}\", \"fields\": [{}]}}", name, inner.join(", "))
            }
        }
        Value::BuiltinFn(_) | Value::Fn { .. } => "null".into(),
    }
}