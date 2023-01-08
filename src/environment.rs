use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::tokens::Literal;

#[derive(Clone, Debug, PartialEq)]
struct Inner {
    enclosing: Option<EnvRef>,
    values: HashMap<String, Literal>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EnvRef(Rc<RefCell<Inner>>);

impl EnvRef {
    pub fn new() -> EnvRef {
        EnvRef(Rc::new(RefCell::new(Inner {
            enclosing: None,
            values: HashMap::new(),
        })))
    }

    pub fn with_enclosing(enclosing: EnvRef) -> EnvRef {
        EnvRef(Rc::new(RefCell::new(Inner {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        })))
    }

    pub fn define(&mut self, name: &str, value: Literal) {
        self.0.borrow_mut().values.insert(name.to_string(), value);
    }

    pub fn assign(&mut self, name: &str, value: Literal) -> Result<(), String> {
        let mut env = self.0.borrow_mut();

        match env.values.contains_key(name) {
            true => {
                env.values.insert(name.to_string(), value);
                Ok(())
            }
            false => match &mut env.enclosing {
                Some(e) => e.assign(name, value),
                None => Err(format!("Undefined variable '{}'", name)),
            },
        }
    }

    pub fn get(&self, name: &str) -> Option<Literal> {
        match self.0.borrow().values.get(name) {
            Some(v) => Some(v.clone()),
            None => match &self.0.borrow().enclosing {
                Some(e) => e.get(name),
                None => None,
            },
        }
    }
}
