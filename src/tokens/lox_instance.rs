use std::fmt::Display;

use super::Class;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct LoxInstance(Class);

impl LoxInstance {
    pub(crate) fn new(class: Class) -> LoxInstance {
        LoxInstance(class)
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<instance {}>", self.0.name))
    }
}
