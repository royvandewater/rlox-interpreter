#[macro_use]
extern crate lazy_static;

mod ast_printer;
mod environment;
mod expr;
mod interpreter;
mod parser;
mod stmt;
mod tokens;

use std::{env, fs, io, process};

use environment::Environment;
use interpreter::Interpreter;
use parser::Parser;
use stmt::Stmts;
use tokens::Tokens;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        eprintln!(" usage: rlox [script]");
        process::exit(64);
    }

    if args.len() == 2 {
        match run_file(args.last().unwrap()) {
            Ok(_) => process::exit(0),
            Err(errors) => {
                eprintln!("Error running file: {:?}", errors);
                process::exit(1);
            }
        };
    }

    run_prompt();
}

fn run_file(filename: &String) -> Result<(), Vec<String>> {
    let contents = fs::read_to_string(filename)
        .map_err(|e| Vec::from([format!("Failed to read file '{}': '{}'", filename, e)]))?;

    run(Environment::new(), contents).map(|_| ())
}

fn run_prompt() {
    let mut environment = Environment::new();

    for line in io::stdin().lines() {
        let failsafe_environment = environment.clone();

        match run(environment, line.unwrap()) {
            Ok(e) => {
                environment = e;
            }
            Err(errors) => {
                environment = failsafe_environment;
                format!("Error running line: {:?}", errors);
            }
        };
    }
}

fn run(environment: Environment, contents: String) -> Result<Environment, Vec<String>> {
    let interpreter = Interpreter::new();
    let tokens: Tokens = contents.parse()?;
    let mut parser: Parser = tokens.into();
    let statements: Stmts = parser.parse()?;

    interpreter.interpret(environment, statements)
}
