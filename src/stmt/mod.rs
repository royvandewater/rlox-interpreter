include!(concat!(env!("OUT_DIR"), "/stmt_generated.rs"));

use std::collections::VecDeque;

#[allow(unused_imports)]
pub use stmt_generated::*;

pub(crate) struct Stmts(VecDeque<Stmt>);

impl From<Vec<Stmt>> for Stmts {
    fn from(value: Vec<Stmt>) -> Self {
        Stmts(value.into())
    }
}

impl Iterator for Stmts {
    type Item = Stmt;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}
