use std::env;

fn run_file(file: &str) {
    println!("Running file {}...", file);
}

fn run_prompt() {
    println!("Running repl");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: lox [script]");
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}
