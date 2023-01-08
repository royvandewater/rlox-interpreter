#![feature(stmt_expr_attributes)]
#[macro_use]
extern crate lazy_static;

mod ast_printer;
mod environment;
mod expr;
mod interpreter;
mod native;
mod parser;
mod resolver;
mod stmt;
mod tokens;

use std::{env, fs, io, process};

use environment::EnvRef;
use stmt::Stmt;
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

fn init_env_ref() -> EnvRef {
    let env_ref = EnvRef::new();
    native::define_native_functions(env_ref.clone());
    env_ref
}

fn run_file(filename: &String) -> Result<(), Vec<String>> {
    let env_ref = init_env_ref();
    let contents = fs::read_to_string(filename)
        .map_err(|e| Vec::from([format!("Failed to read file '{}': '{}'", filename, e)]))?;

    run(env_ref, contents).map(|_| ())
}

fn run_prompt() {
    let env_ref = init_env_ref();

    for line in io::stdin().lines() {
        match run(env_ref.clone(), line.unwrap()) {
            Ok(_) => {}
            Err(errors) => {
                format!("Error running line: {:?}", errors);
            }
        };
    }
}

fn run(env_ref: EnvRef, contents: String) -> Result<(), Vec<String>> {
    let tokens: Tokens = contents.parse()?;
    let statements: Vec<Stmt> = parser::parse(tokens)?;

    let locals = resolver::resolve_locals(&statements)?;

    interpreter::interpret(env_ref, locals, &statements)
}
