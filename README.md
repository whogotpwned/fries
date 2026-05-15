<div align="center">
  <img src="Banner.png" alt="Fries Logo" width="600"/>

  # Fries

  A minimal, dynamically-typed programming language with functional and imperative features.

  ![Rust](https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square)
  ![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)
  ![Version](https://img.shields.io/badge/version-0.1.0-green?style=flat-square)
  ![Status](https://img.shields.io/badge/status-active-brightgreen?style=flat-square)

</div>

---

## Overview

Fries is a simple, untyped programming language that blends **functional** and **imperative** paradigms. It features first-class functions, closures, algebraic data types, pattern matching, and powerful list comprehensions — all with a clean, minimal syntax.

Built as a tree-walking interpreter in Rust.

## Features

- **Dynamic typing** — no type annotations needed
- **Immutable by default** — `let` binds immutably, `var` allows mutation
- **First-class functions** — functions are values
- **Closures** — functions capture their lexical scope
- **Algebraic Data Types** — `type Maybe = Just(value) | Nothing`
- **Pattern matching** — `match` expressions with destructuring
- **List ranges** — `[1..10]`, `[1, 3, ..., 10]`
- **List comprehensions** — `[x * 2 for x in xs]`, `[x for x in xs if x > 3]`
- **Expression-oriented** — `if`, `match`, `do` blocks return values
- **Simple syntax** — no semicolons required, minimal punctuation

## Quick Start

```bash
# Build
cargo build

# Run a file
./target/debug/fries examples/demo.fries

# Start REPL
./target/debug/fries
```

## Language Reference

### Variables

```fries
let x = 5          # immutable
var counter = 0    # mutable
counter += 1       # compound assignment (+=, -=, *=, /=)
```

### Data Types

```fries
42                  # int
3.14                # float
"hello"             # string
true / false        # bool
null                # null
[1, 2, 3]          # list
```

### Arithmetic & Comparison

```fries
1 + 2       # 3
10 / 3      # 3
1.0 / 2     # 0.5
"hello" + " " + "world"  # string concat

x == y      # equal
x != y      # not equal
x < y       # less than
x >= y      # greater or equal
!true       # false
```

### Functions

```fries
# Named function
fn greet(name) { "Hello, " + name }
println(greet("Fries"))

# Anonymous function / lambda
let double = fn(x) { x * 2 }
println(double(5))    # 10

# Closures
let make_adder = fn(x) {
  fn(y) { x + y }
}
let add5 = make_adder(5)
println(add5(3))      # 8
```

### Conditionals

```fries
# if is an expression — it returns a value
let label = if score >= 90 {
  "A"
} else if score >= 80 {
  "B"
} else if score >= 70 {
  "C"
} else {
  "F"
}
```

### Loops

```fries
# while
var i = 0
while i < 10 {
  i += 1
}

# for
for x in [1..10] {
  println(x)
}
```

### Lists

```fries
# Literal
let xs = [1, 2, 3, 4, 5]

# Range (inclusive)
let nums = [1..10]           # [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let down = [10..1]           # [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]

# Arithmetische Folge (step inferred)
let odds = [1, 2, ..., 10]  # [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let by3  = [0, 3, ..., 15]  # [0, 3, 6, 9, 12, 15]

# Short range with implicit step 1
let easy = [1, ..., 5]       # [1, 2, 3, 4, 5]

# List comprehension
let doubled = [x * 2 for x in [1..5]]        # [2, 4, 6, 8, 10]
let evens   = [x for x in [1..10] if x % 2 == 0]  # [2, 4, 6, 8, 10]
let squares = [x * x for x in [1..5]]        # [1, 4, 9, 16, 25]

# Indexing
println(xs[0])      # 1
println(len(xs))    # 5
```

### Algebraic Data Types

```fries
type Maybe = Just(value) | Nothing
type Result = Ok(value) | Err(msg)

let a = Just(42)
let b = Nothing
let c = Ok("success")
let d = Err("file not found")
```

### Pattern Matching

```fries
type Form = Kreis(radius) | Rechteck(breite, hoehe) | Quadrat(seite)

let form = Kreis(5)

let area = match form {
  Kreis(r) => pi * r * r,
  Rechteck(b, h) => b * h,
  Quadrat(s) => s * s
}

# Literal patterns
match x {
  0 => "zero",
  1 => "one",
  _ => "other"
}

# List patterns
match [1, 2, 3] {
  [a, b, c] => str(a + b + c)
}
```

### Do Blocks

```fries
# do blocks are expressions — last value is returned
let result = do {
  let a = 10
  let b = 20
  a + b
}
println(result)    # 30
```

### Built-in Functions

| Function | Description |
|----------|-------------|
| `print(v)` | Print without newline |
| `println(v)` | Print with newline |
| `len(x)` | Length of string or list |
| `push(list, val)` | Append to list, returns list |
| `pop(list)` | Remove and return last element |
| `head(list)` | First element |
| `tail(list)` | All elements except first |
| `range(start, end)` | Generate list from start to end-1 |
| `typeof(x)` | Type name as string |
| `str(x)` | Convert to string |
| `int(x)` | Convert to int |
| `float(x)` | Convert to float |
| `abs(x)` | Absolute value |
| `min(a, b)` | Minimum |
| `max(a, b)` | Maximum |

### Comments

```fries
# This is a comment
let x = 5  # inline comment
```

## Architecture

```
src/
├── main.rs       # REPL + file execution
├── lib.rs        # Module exports
├── token.rs      # Token definitions
├── lexer.rs      # Tokenizer
├── ast.rs        # AST node definitions
├── parser.rs     # Recursive-descent parser
├── value.rs      # Runtime values + environment
├── eval.rs       # Tree-walking evaluator
└── builtins.rs   # Built-in functions
```

**Pipeline:** Source → Lexer → Tokens → Parser → AST → Evaluator → Value

## Syntax Comparison

| Feature | Python | Fries |
|---------|--------|-------|
| Variable | `x = 5` | `let x = 5` |
| Mutable var | `x = 5` (always) | `var x = 5` |
| Function | `def add(a, b):` | `fn add(a, b) { ... }` |
| Lambda | `lambda x: x * 2` | `fn(x) { x * 2 }` |
| If | `if x > 0:` | `if x > 0 { ... }` |
| Else if | `elif x > 0:` | `else if x > 0 { ... }` |
| Range | `range(1, 11)` | `[1..10]` |
| Step range | `range(0, 11, 2)` | `[0, 2, ..., 10]` |
| List comp | `[x*2 for x in xs]` | `[x * 2 for x in xs]` |
| Filter comp | `[x for x in xs if x>0]` | `[x for x in xs if x > 0]` |
| Match | `match/case` (3.10+) | `match x { ... }` |
| ADTs | (not built-in) | `type T = A \| B` |
| Null | `None` | `null` |
| Bool | `True / False` | `true / false` |
| Comment | `# comment` | `# comment` |

## License

MIT