use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::tokens::Literal;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }))
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
                Some(e) => e.borrow_mut().assign(name, value),
                None => Err(format!("Undefined variable '{}'", name)),
            },
        }
    }

    pub fn get(&self, name: &str) -> Option<Literal> {
        match self.values.get(name) {
            Some(v) => Some(v.clone()),
            None => match &self.enclosing {
                Some(e) => e.borrow().get(name),
                None => None,
            },
        }
    }
}
