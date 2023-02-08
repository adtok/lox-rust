mod expression;
mod scanner;
use crate::scanner::*;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process::exit;

fn run_file(path: &str) -> Result<(), String> {
    match fs::read_to_string(path) {
        Err(msg) => return Err(msg.to_string()),
        Ok(contents) => return run(&contents),
    }
}

fn run(contents: &str) -> Result<(), String> {
    let mut scanner = Scanner::new(contents);
    let tokens = scanner.scan_tokens()?;

    for token in tokens {
        println!("{:?}", token)
    }
    Ok(())
}

fn run_prompt() -> Result<(), String> {
    println!("Entering Lox repl... Ctrl + D or `.exit` to exit.");
    loop {
        print!("> ");
        io::stdout().flush().expect("Should not fail.");
        let mut buffer = String::new();
        let stdin = io::stdin();
        match stdin.read_line(&mut buffer) {
            Err(msg) => return Err(msg.to_string()),
            Ok(value) => {
                if value == 0 {
                    println!("\nClosing...");
                    exit(0)
                }
            }
        }
        let value = buffer.trim();
        if value == ".exit".to_string() {
            break;
        }
        run(&value).unwrap();
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: lox [script]");
        exit(64)
    } else if args.len() == 2 {
        match run_file(&args[1]) {
            Ok(_) => exit(0),
            Err(msg) => panic!("Error\n{}", msg),
        }
    } else {
        match run_prompt() {
            Ok(_) => exit(0),
            Err(msg) => panic!("Error:\n{}", msg),
        }
    }
}
