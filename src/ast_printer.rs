use crate::{environment::Environment, expr::*};

#[allow(dead_code)]
pub(crate) fn print(expression: Expr) -> String {
    let environment = Environment::new();
    walk_expr(&mut AstPrinter, environment, expression)
}

struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&self, name: &str, exprs: Vec<Box<Expr>>) -> String {
        let mut builder = String::new();

        builder.push('(');
        builder.push_str(name);

        for expr in exprs {
            let environment = Environment::new();
            builder.push(' ');
            builder.push_str(&walk_expr(self, environment, *expr))
        }
        builder.push(')');

        return builder;
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary(&self, _environment: Environment, expr: BinaryExpr) -> String {
        self.parenthesize(&expr.operator.lexeme, vec![expr.left, expr.right])
    }

    fn visit_literal(&self, _environment: Environment, expr: LiteralExpr) -> String {
        match &expr.value {
            crate::tokens::Literal::Nil => "nil".to_string(),
            crate::tokens::Literal::Number(n) => format!("{}", n),
            crate::tokens::Literal::String(s) => format!("{}", s),
            crate::tokens::Literal::Boolean(b) => format!("{}", b),
        }
    }

    fn visit_grouping(&self, _environment: Environment, expr: GroupingExpr) -> String {
        self.parenthesize("group", vec![expr.expression])
    }

    fn visit_unary(&self, _environment: Environment, expr: UnaryExpr) -> String {
        self.parenthesize(&expr.operator.lexeme, vec![expr.right])
    }

    fn visit_variable(&self, _environment: Environment, _expr: VariableExpr) -> String {
        todo!()
    }

    fn visit_assign(&self, _environment: Environment, _expr: AssignExpr) -> String {
        todo!()
    }
}
