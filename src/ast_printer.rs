use crate::{environment::EnvRef, expr::*};

#[allow(dead_code)]
pub(crate) fn print(expression: &Expr) -> String {
    let env_ref = EnvRef::new();
    walk_expr(&mut AstPrinter, env_ref, expression)
}

struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&self, name: &str, exprs: Vec<&Expr>) -> String {
        let mut builder = String::new();

        builder.push('(');
        builder.push_str(name);

        for expr in exprs {
            let env_ref = EnvRef::new();
            builder.push(' ');
            builder.push_str(&walk_expr(self, env_ref, expr))
        }
        builder.push(')');

        return builder;
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary(&self, _env_ref: EnvRef, expr: &BinaryExpr) -> String {
        self.parenthesize(&expr.operator.lexeme, vec![&expr.left, &expr.right])
    }

    fn visit_literal(&self, _env_ref: EnvRef, expr: &LiteralExpr) -> String {
        format!("{}", &expr.value)
    }

    fn visit_grouping(&self, _env_ref: EnvRef, expr: &GroupingExpr) -> String {
        self.parenthesize("group", vec![&expr.expression])
    }

    fn visit_unary(&self, _env_ref: EnvRef, expr: &UnaryExpr) -> String {
        self.parenthesize(&expr.operator.lexeme, vec![&expr.right])
    }

    fn visit_variable(&self, _env_ref: EnvRef, _expr: &VariableExpr) -> String {
        todo!()
    }

    fn visit_assign(&self, _env_ref: EnvRef, _expr: &AssignExpr) -> String {
        todo!()
    }

    fn visit_logical(&self, _env_ref: EnvRef, _expr: &LogicalExpr) -> String {
        todo!()
    }

    fn visit_call(&self, _env_ref: EnvRef, _expr: &CallExpr) -> String {
        todo!()
    }
}
