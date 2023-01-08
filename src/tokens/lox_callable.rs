use crate::stmt::Stmt;
use std::fmt::Display;

use super::{Literal, Token};
use crate::environment::EnvRef;

pub(crate) type Native = fn() -> Literal;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Function {
    pub body: Vec<Stmt>,
    pub params: Vec<Token>,
    pub env_ref: EnvRef,
}

impl Function {
    pub(crate) fn new(body: Vec<Stmt>, params: Vec<Token>, env_ref: EnvRef) -> Self {
        Self {
            body,
            params,
            env_ref,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Callable {
    Native(Native),
    Function(Function),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
            Callable::Function(f) => f.params.len(),
        }
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<fn {}>", self.name))
    }
}
