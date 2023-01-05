use std::collections::HashMap;

use crate::tokens::Literal;

pub(crate) struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn with_enclosing(enclosing: Environment) -> Environment {
        Environment {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn enclosing(&mut self) -> Option<Environment> {
        match self.enclosing.take() {
            Some(e) => Some(*e),
            None => None,
        }
    }

    pub fn define(&mut self, name: &str, value: Literal) {
        self.values.insert(name.to_string(), value);
    }

    pub fn assign(&mut self, name: &str, value: Literal) -> Result<(), String> {
        match self.values.contains_key(name) {
            true => {
                self.values.insert(name.to_string(), value);
                Ok(())
            }
            false => match &mut self.enclosing {
                Some(e) => e.assign(name, value),
                None => Err(format!("Undefined variable '{}'", name)),
            },
        }
    }

    pub fn get(&self, name: &str) -> Option<Literal> {
        match self.values.get(name) {
            Some(v) => Some(v.clone()),
            None => match &self.enclosing {
                Some(e) => e.get(name),
                None => None,
            },
        }
    }
}
