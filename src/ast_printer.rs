// use crate::expr::{walk_expr, Expr, Visitor};

use crate::expr::*;

pub(crate) fn print(expression: Expr) -> String {
    walk_expr(&AstPrinter, &expression)
}

struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&self, name: &str, exprs: &[&Expr]) -> String {
        let mut builder = String::new();

        builder.push('(');
        builder.push_str(name);

        for expr in exprs.iter() {
            builder.push(' ');
            builder.push_str(&walk_expr(self, expr))
        }
        builder.push(')');

        return builder;
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary(&self, expr: &BinaryExpr) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&expr.left, &expr.right])
    }

    fn visit_literal(&self, expr: &LiteralExpr) -> String {
        match &expr.value {
            crate::tokens::Literal::None => "nil".to_string(),
            crate::tokens::Literal::Number(n) => format!("{}", n),
            crate::tokens::Literal::String(s) => format!("{}", s),
        }
    }

    fn visit_grouping(&self, expr: &GroupingExpr) -> String {
        self.parenthesize("group", &[&expr.expression])
    }

    fn visit_unary(&self, expr: &UnaryExpr) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&expr.right])
    }
}
