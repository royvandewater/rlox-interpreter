use std::collections::HashMap;

use crate::tokens::Literal;

pub(crate) struct Environment(HashMap<String, Literal>);

impl Environment {
    pub fn new() -> Environment {
        Environment(HashMap::new())
    }

    pub fn define(&mut self, name: &str, value: Literal) {
        self.0.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Literal> {
        match self.0.get(name) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
}
