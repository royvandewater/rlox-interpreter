#[macro_use]
extern crate lazy_static;

mod ast_printer;
mod expr;
mod tokens;

use std::{env, fs, io, process};

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

    return run(contents);
}

fn run_prompt() {
    for line in io::stdin().lines() {
        match run(line.unwrap()) {
            Ok(_) => continue,
            Err(errors) => format!("Error running line: {:?}", errors),
        };
    }
}

fn run(contents: String) -> Result<(), Vec<String>> {
    let tokens: Tokens = contents.parse()?;
    let expression: expr::Expr = tokens.try_into()?;

    println!("{}", ast_printer::print(expression));

    Ok(())
}
