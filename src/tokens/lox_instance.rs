use std::{collections::BTreeMap, fmt::Display};

use super::{Class, Literal};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct LoxInstance {
    class: Class,
    fields: BTreeMap<String, Literal>,
}

impl LoxInstance {
    pub(crate) fn new(class: Class) -> LoxInstance {
        LoxInstance {
            class,
            fields: BTreeMap::new(),
        }
    }

    pub(crate) fn get(&self, name: &str) -> Result<Literal, String> {
        self.fields.get(name).cloned().ok_or("err".to_string())
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<instance {}>", self.class.name))
    }
}
