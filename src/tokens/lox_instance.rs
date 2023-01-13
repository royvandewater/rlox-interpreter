use std::{cell::RefCell, collections::BTreeMap, fmt::Display, rc::Rc};

use super::{Callable, Class, Function, Literal, LoxCallable};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Inner {
    class: Class,
    fields: BTreeMap<String, Literal>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LoxInstance(Rc<RefCell<Inner>>);

impl LoxInstance {
    pub(crate) fn new(class: Class) -> LoxInstance {
        LoxInstance(Rc::new(RefCell::new(Inner {
            class,
            fields: BTreeMap::new(),
        })))
    }

    pub(crate) fn get(&self, name: &str) -> Result<Literal, String> {
        match self.0.borrow().fields.get(name) {
            Some(value) => Ok(value.clone()),
            None => match self.find_method(name) {
                None => Err(format!("Undefined property: '{}'", name)),
                Some(method) => Ok(Literal::Callable(LoxCallable::new(
                    name.to_string(),
                    Callable::Function(method.bind(self.clone())),
                ))),
            },
        }
    }

    pub(crate) fn set(&mut self, name: &str, value: Literal) {
        self.0.borrow_mut().fields.insert(name.to_string(), value);
    }

    pub(crate) fn find_method(&self, name: &str) -> Option<Function> {
        self.0.borrow().class.methods.get(name).cloned()
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<instance {}>", self.0.borrow().class.name))
    }
}

impl std::hash::Hash for LoxInstance {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.0.borrow().hash(state);
    }
}
