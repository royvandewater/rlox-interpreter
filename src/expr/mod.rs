include!(concat!(env!("OUT_DIR"), "/expr_generated.rs"));

#[allow(unused_imports)]
pub use expr_generated::*;

pub(crate) fn nil() -> Expr {
    Expr::Literal(LiteralExpr::new(Literal::Nil))
}
