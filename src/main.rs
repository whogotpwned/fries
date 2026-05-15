use fries::eval::Evaluator;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let filename = &args[1];
        match fs::read_to_string(filename) {
            Ok(source) => {
                let mut evaluator = Evaluator::new();
                match evaluator.run(&source) {
                    Ok(Some(val)) => {
                        if !matches!(val, fries::value::Value::Null) {
                            println!("{val}");
                        }
                    }
                    Ok(None) => {}
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
            Err(e) => {
                eprintln!("Failed to read file '{filename}': {e}");
                std::process::exit(1);
            }
        }
    } else {
        repl();
    }
}

fn repl() {
    println!("Fries Language Interpreter v0.1.0");
    println!("Type 'exit' or Ctrl+D to quit.\n");

    let stdin = io::stdin();
    let mut evaluator = Evaluator::new();

    loop {
        print!("fries> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {}
            Err(_) => break,
        }

        let trimmed = line.trim();
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        let mut input = line.clone();
        let open_braces = input.chars().filter(|&c| c == '{').count();
        let close_braces = input.chars().filter(|&c| c == '}').count();

        if open_braces > close_braces {
            let needed = open_braces - close_braces;
            for _ in 0..needed {
                print!("  ...> ");
                io::stdout().flush().unwrap();
                let mut continuation = String::new();
                match stdin.lock().read_line(&mut continuation) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(_) => break,
                }
                input.push_str(&continuation);
            }
        }

        match evaluator.run(&input) {
            Ok(Some(val)) => println!("{val}"),
            Ok(None) => {}
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}