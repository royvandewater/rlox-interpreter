use crate::expr::*;

#[allow(dead_code)]
pub(crate) fn print(expression: &Expr) -> String {
    walk_expr(&mut AstPrinter, (), expression)
}

struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&self, name: &str, exprs: Vec<&Expr>) -> String {
        let mut builder = String::new();

        builder.push('(');
        builder.push_str(name);

        for expr in exprs {
            builder.push(' ');
            builder.push_str(&walk_expr(self, (), expr))
        }
        builder.push(')');

        return builder;
    }
}

impl Visitor<(), String> for AstPrinter {
    fn visit_binary(&self, _: (), expr: &BinaryExpr) -> String {
        self.parenthesize(&expr.operator.lexeme, vec![&expr.left, &expr.right])
    }

    fn visit_literal(&self, _: (), expr: &LiteralExpr) -> String {
        format!("{}", &expr.value)
    }

    fn visit_grouping(&self, _: (), expr: &GroupingExpr) -> String {
        self.parenthesize("group", vec![&expr.expression])
    }

    fn visit_unary(&self, _: (), expr: &UnaryExpr) -> String {
        self.parenthesize(&expr.operator.lexeme, vec![&expr.right])
    }

    fn visit_variable(&self, _: (), _expr: &VariableExpr) -> String {
        todo!()
    }

    fn visit_assign(&self, _: (), _expr: &AssignExpr) -> String {
        todo!()
    }

    fn visit_logical(&self, _: (), _expr: &LogicalExpr) -> String {
        todo!()
    }

    fn visit_call(&self, _: (), _expr: &CallExpr) -> String {
        todo!()
    }

    fn visit_get(&self, _: (), _expr: &GetExpr) -> String {
        todo!()
    }

    fn visit_set(&self, _: (), _expr: &SetExpr) -> String {
        todo!()
    }

    fn visit_this(&self, _: (), _expr: &ThisExpr) -> String {
        todo!()
    }
}
