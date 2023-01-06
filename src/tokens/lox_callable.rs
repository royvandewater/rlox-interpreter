use crate::stmt::Stmt;
use std::fmt::Display;

use super::{Literal, Token};

pub(crate) type Native = fn() -> Literal;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Callable {
    Native(Native),
    Function((Vec<Stmt>, Vec<Token>)),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LoxCallable {
    pub name: String,
    pub callable: Callable,
}

impl LoxCallable {
    pub fn new(name: String, callable: Callable) -> LoxCallable {
        LoxCallable { callable, name }
    }

    pub fn arity(&self) -> usize {
        match &self.callable {
            Callable::Native(_) => 0,
            Callable::Function((_, params)) => params.len(),
        }
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<fn {}>", self.name))
    }
}
