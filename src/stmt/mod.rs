include!(concat!(env!("OUT_DIR"), "/stmt_generated.rs"));

#[allow(unused_imports)]
pub use stmt_generated::*;

pub(crate) fn noop(id: usize) -> Stmt {
    Stmt::Block(BlockStmt::new(id, Vec::new()))
}
