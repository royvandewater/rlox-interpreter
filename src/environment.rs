use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::tokens::Literal;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Inner {
    enclosing: Option<EnvRef>,
    values: BTreeMap<String, Literal>,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct EnvRef(Rc<RefCell<Inner>>);

impl EnvRef {
    pub fn new() -> EnvRef {
        EnvRef(Rc::new(RefCell::new(Inner {
            enclosing: None,
            values: BTreeMap::new(),
        })))
    }

    pub fn with_enclosing(enclosing: EnvRef) -> EnvRef {
        EnvRef(Rc::new(RefCell::new(Inner {
            enclosing: Some(enclosing),
            values: BTreeMap::new(),
        })))
    }

    pub fn define(&mut self, name: &str, value: Literal) {
        self.0.borrow_mut().values.insert(name.to_string(), value);
    }

    pub(crate) fn assign_at_distance(
        &mut self,
        distance: usize,
        name: &str,
        value: Literal,
    ) -> Result<(), String> {
        match distance {
            0 => self.assign_current(name, value),
            _ => match &mut self.0.borrow_mut().enclosing {
                None => panic!("Tried to assign outside of the scope cactus"),
                Some(e) => e.assign_at_distance(distance - 1, name, value),
            },
        }
    }

    pub fn assign_current(&mut self, name: &str, value: Literal) -> Result<(), String> {
        let mut env = self.0.borrow_mut();

        match env.values.contains_key(name) {
            true => {
                env.values.insert(name.to_string(), value);
                Ok(())
            }
            false => Err(format!("Undefined variable '{}'", name)),
        }
    }

    pub(crate) fn assign_global(&mut self, name: &str, value: Literal) -> Result<(), String> {
        {
            let mut env = self.0.borrow_mut();

            if env.enclosing.is_some() {
                return env.enclosing.as_mut().unwrap().assign_global(name, value);
            }
        }

        self.assign_current(name, value)
    }

    pub fn get_at_distance(&self, distance: usize, name: &str) -> Option<Literal> {
        match distance {
            0 => self.get_current(name),
            _ => match &self.0.borrow().enclosing {
                Some(e) => e.get_at_distance(distance - 1, name),
                None => panic!("Tried to find variable outside the scope cactus"),
            },
        }
    }

    fn get_current(&self, name: &str) -> Option<Literal> {
        self.0.borrow().values.get(name).cloned()
    }

    pub fn get_global(&self, name: &str) -> Option<Literal> {
        match &self.0.borrow().enclosing {
            Some(e) => e.get_global(name),
            None => self.get_current(name),
        }
    }
}

impl std::fmt::Debug for EnvRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.0.borrow();
        f.debug_struct("EnvRef")
            .field("values", &inner.values)
            .field("enclosing", &inner.enclosing)
            .finish()
    }
}

impl std::hash::Hash for EnvRef {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.0.borrow().hash(state);
    }
}
