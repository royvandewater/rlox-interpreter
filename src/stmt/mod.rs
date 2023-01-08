include!(concat!(env!("OUT_DIR"), "/stmt_generated.rs"));

use std::collections::{vec_deque::Iter, VecDeque};

#[allow(unused_imports)]
pub use stmt_generated::*;

pub(crate) struct Stmts(VecDeque<Stmt>);

impl From<Vec<Stmt>> for Stmts {
    fn from(value: Vec<Stmt>) -> Self {
        Stmts(value.into())
    }
}

impl Stmts {
    pub(crate) fn iter(&self) -> Iter<Stmt> {
        self.0.iter()
    }
}

pub(crate) fn noop() -> Stmt {
    Stmt::Block(BlockStmt::new(Vec::new()))
}
