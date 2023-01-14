use crate::stmt::Stmt;
use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use super::{Literal, LoxInstance, Token};
use crate::environment::Environment;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Class {
    pub name: String,
    pub methods: BTreeMap<String, Function>,
}

impl Class {
    pub(crate) fn new(name: String, methods: BTreeMap<String, Function>) -> Self {
        Self { name, methods }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Function {
    pub body: Vec<Stmt>,
    pub params: Vec<Token>,
    pub env: Environment,
    pub is_initializer: bool,
}

impl Function {
    pub(crate) fn new(body: Vec<Stmt>, params: Vec<Token>, env: Environment) -> Self {
        Self {
            body,
            params,
            env,
            is_initializer: false,
        }
    }

    pub(crate) fn new_initializer(
        body: Vec<Stmt>,
        params: Vec<Token>,
        env: Environment,
    ) -> Function {
        Self {
            body,
            params,
            env,
            is_initializer: true,
        }
    }

    pub(crate) fn bind(&self, instance: LoxInstance) -> Function {
        let mut env = Environment::with_enclosing(self.env.clone());
        env.define("this", Literal::ClassInstance(instance));
        Function::new(self.body.clone(), self.params.clone(), env)
    }
}

pub(crate) type Native = fn() -> Literal;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Callable {
    Class(Class),
    Function(Function),
    Native(Native),
}

#[derive(Clone, Eq, Hash, PartialEq)]
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
            Callable::Class(class) => match class.methods.get("init") {
                Some(method) => method.params.len(),
                None => 0,
            },
            Callable::Function(f) => f.params.len(),
            Callable::Native(_) => 0,
        }
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self.callable {
            Callable::Class(_) => format!("<class {}>", self.name),
            Callable::Function(_) => format!("<fn {}>", self.name),
            Callable::Native(_) => todo!("<native-fn {}>", self.name),
        })
    }
    // format_args!("<fn {}>", self.name)
}

impl Debug for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoxCallable")
            .field("name", &self.name)
            .finish()
    }
}
