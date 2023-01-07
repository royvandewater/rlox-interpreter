use std::{cell::RefCell, rc::Rc};

use crate::{environment::Environment, expr::*};

#[allow(dead_code)]
pub(crate) fn print(expression: Expr) -> String {
    let env_ref = Rc::new(RefCell::new(Environment::new()));
    walk_expr(&mut AstPrinter, env_ref, expression)
}

struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&self, name: &str, exprs: Vec<Box<Expr>>) -> String {
        let mut builder = String::new();

        builder.push('(');
        builder.push_str(name);

        for expr in exprs {
            let env_ref = Rc::new(RefCell::new(Environment::new()));
            builder.push(' ');
            builder.push_str(&walk_expr(self, env_ref, *expr))
        }
        builder.push(')');

        return builder;
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary(&self, _environment: Rc<RefCell<Environment>>, expr: BinaryExpr) -> String {
        self.parenthesize(&expr.operator.lexeme, vec![expr.left, expr.right])
    }

    fn visit_literal(&self, _environment: Rc<RefCell<Environment>>, expr: LiteralExpr) -> String {
        format!("{}", &expr.value)
    }

    fn visit_grouping(&self, _environment: Rc<RefCell<Environment>>, expr: GroupingExpr) -> String {
        self.parenthesize("group", vec![expr.expression])
    }

    fn visit_unary(&self, _environment: Rc<RefCell<Environment>>, expr: UnaryExpr) -> String {
        self.parenthesize(&expr.operator.lexeme, vec![expr.right])
    }

    fn visit_variable(
        &self,
        _environment: Rc<RefCell<Environment>>,
        _expr: VariableExpr,
    ) -> String {
        todo!()
    }

    fn visit_assign(&self, _environment: Rc<RefCell<Environment>>, _expr: AssignExpr) -> String {
        todo!()
    }

    fn visit_logical(&self, _environment: Rc<RefCell<Environment>>, _expr: LogicalExpr) -> String {
        todo!()
    }

    fn visit_call(&self, _environment: Rc<RefCell<Environment>>, _expr: CallExpr) -> String {
        todo!()
    }
}
