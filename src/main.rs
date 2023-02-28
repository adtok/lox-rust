mod callable;
mod environment;
mod expression;
mod interpreter;
mod parser;
mod scanner;
mod statement;
mod tests;
use interpreter::Interpreter;
use parser::Parser;

use crate::scanner::*;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process::exit;

fn run_file(path: &str) -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    match fs::read_to_string(path) {
        Err(msg) => Err(msg.to_string()),
        Ok(contents) => run(&mut interpreter, &contents),
    }
}

fn run(interpreter: &mut Interpreter, contents: &str) -> Result<(), String> {
    let mut scanner = Scanner::new(contents);
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;

    interpreter.interpret(statements.iter().collect())?;

    Ok(())
}

fn run_prompt() -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    println!("Entering Lox repl... Ctrl + D or `.exit` to exit.");
    loop {
        print!("> ");
        io::stdout().flush().expect("Could not flush stdout.");
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
        if value == ".exit" {
            break;
        }
        run(&mut interpreter, value)?;
    }
    Ok(())
}

pub fn run_string(contents: &str) -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    run(&mut interpreter, contents)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let result = match args.len() {
        3 if args[1] == "e" => run_string(&args[2]),
        2 => run_file(&args[1]),
        1 => run_prompt(),
        _ => {
            println!("Usage: lox [script]");
            exit(64)
        }
    };

    match result {
        Ok(_) => exit(0),
        Err(msg) => {
            println!("Error:\n{msg}");
            exit(1)
        }
    }
}
