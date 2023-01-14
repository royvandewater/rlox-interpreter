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

    fn visit_variable(&self, _: (), expr: &VariableExpr) -> String {
        expr.name.to_string()
    }

    fn visit_assign(&self, _: (), expr: &AssignExpr) -> String {
        self.parenthesize(&format!("let {}", expr.name), vec![&expr.value])
    }

    fn visit_logical(&self, _: (), expr: &LogicalExpr) -> String {
        self.parenthesize(&expr.operator.lexeme, vec![&expr.left, &expr.right])
    }

    fn visit_call(&self, _: (), expr: &CallExpr) -> String {
        let func = walk_expr(self, (), &expr.callee);
        self.parenthesize(&func, expr.arguments.iter().collect())
    }

    fn visit_get(&self, _: (), expr: &GetExpr) -> String {
        let object = walk_expr(self, (), &expr.object);
        format!("{}.{}", object, expr.name)
    }

    fn visit_set(&self, _: (), expr: &SetExpr) -> String {
        let object = walk_expr(self, (), &expr.object);
        format!("{}.{} = ", object, expr.name)
    }

    fn visit_this(&self, _: (), expr: &ThisExpr) -> String {
        expr.keyword.lexeme.to_string()
    }
}
