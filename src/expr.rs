use crate::tokens::{self, Token};

pub(crate) trait Visitor<T> {
    fn visit_binary(&self, expr: &Binary) -> T;
    fn visit_literal(&self, expr: &Literal) -> T;
}

pub(crate) enum Expr {
    Binary(Binary),
    Literal(Literal),
}

pub(crate) fn walk_expr<T>(visitor: &dyn Visitor<T>, expr: &Expr) -> T {
    match expr {
        Expr::Binary(b) => visitor.visit_binary(b),
        Expr::Literal(l) => visitor.visit_literal(l),
    }
}

pub(crate) struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Binary {
    pub(crate) fn new(left: Expr, operator: Token, right: Expr) -> Self {
        Self {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

pub(crate) struct Literal {
    pub value: tokens::Literal,
}

impl Literal {
    pub(crate) fn new(value: tokens::Literal) -> Self {
        Self { value }
    }
}
