use crate::environment::Environment;
use crate::stmt::{walk_stmt, Stmt, Stmts};
use crate::tokens::TokenType;
use crate::{expr, tokens::Literal};
use crate::{expr::*, stmt};

use Literal as L;
use TokenType as TT;

pub(crate) struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub(crate) fn new() -> Interpreter {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub(crate) fn interpret(&mut self, statements: Stmts) -> Result<(), Vec<String>> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, statement: Stmt) -> Result<(), Vec<String>> {
        walk_stmt(self, statement)
    }

    fn evaluate(&mut self, expression: expr::Expr) -> Result<Literal, Vec<String>> {
        walk_expr(self, expression)
    }
}

impl expr::Visitor<Result<Literal, Vec<String>>> for Interpreter {
    fn visit_binary(&mut self, expr: BinaryExpr) -> Result<Literal, Vec<String>> {
        let left = self.evaluate(*expr.left)?;
        let right = self.evaluate(*expr.right)?;

        let operator = expr.operator.token_type;

        match (left, operator, right) {
            // Math
            (L::Number(l), TT::Plus, L::Number(r)) => Ok(L::Number(l + r)),
            (L::Number(l), TT::Minus, L::Number(r)) => Ok(L::Number(l - r)),
            (L::Number(l), TT::Slash, L::Number(r)) => Ok(L::Number(l / r)),
            (L::Number(l), TT::Star, L::Number(r)) => Ok(L::Number(l * r)),

            // String concatenation
            (L::String(l), TT::Plus, L::String(r)) => Ok(L::String(format!("{}{}", l, r))),

            // Comparison operators
            (L::Number(l), TT::Greater, L::Number(r)) => Ok(L::Boolean(l > r)),
            (L::Number(l), TT::GreaterEqual, L::Number(r)) => Ok(L::Boolean(l >= r)),
            (L::Number(l), TT::Less, L::Number(r)) => Ok(L::Boolean(l < r)),
            (L::Number(l), TT::LessEqual, L::Number(r)) => Ok(L::Boolean(l <= r)),

            // Equality operators
            (l, TT::EqualEqual, r) => Ok(L::Boolean(l == r)),
            (l, TT::BangEqual, r) => Ok(L::Boolean(l != r)),

            (l, _, r) => Err(vec![format!(
                "Unsupported types for binary operation: {} {} {}",
                l, expr.operator.lexeme, r
            )]),
        }
    }

    fn visit_grouping(&mut self, expr: GroupingExpr) -> Result<Literal, Vec<String>> {
        self.evaluate(*expr.expression)
    }

    fn visit_literal(&mut self, expr: LiteralExpr) -> Result<Literal, Vec<String>> {
        Ok(expr.value)
    }

    fn visit_unary(&mut self, expr: UnaryExpr) -> Result<Literal, Vec<String>> {
        let right = self.evaluate(*expr.right)?;

        match (expr.operator.token_type, right) {
            (TokenType::Bang, v) => Ok(Literal::Boolean(!evaluate_truthy(v))),
            (TokenType::Minus, Literal::Number(n)) => Ok(Literal::Number(-1.0 * n)),
            (TokenType::Minus, v) => Err(vec![format!(
                "Invalid attempt to perform numerical negation on non-number: {}",
                v
            )]),
            (_, v) => Err(vec![format!(
                "The value '{}' does not support the unary operation '{}'",
                v, expr.operator.lexeme
            )]),
        }
    }

    fn visit_variable(&mut self, expr: VariableExpr) -> Result<Literal, Vec<String>> {
        match self.environment.get(&expr.name.lexeme) {
            None => Err(vec![format!(
                "variable with name '{}' not defined",
                &expr.name.lexeme
            )]),
            Some(literal) => Ok(literal),
        }
    }
}

impl stmt::Visitor<Result<(), Vec<String>>> for Interpreter {
    fn visit_expression(&mut self, stmt: stmt::ExpressionStmt) -> Result<(), Vec<String>> {
        self.evaluate(*stmt.expression)?;
        Ok(())
    }

    fn visit_print(&mut self, stmt: stmt::PrintStmt) -> Result<(), Vec<String>> {
        let val = self.evaluate(*stmt.expression)?;
        println!("{}", val);
        Ok(())
    }

    fn visit_var(&mut self, stmt: stmt::VarStmt) -> Result<(), Vec<String>> {
        let value = match stmt.initializer {
            Some(expression) => self.evaluate(expression)?,
            None => Literal::Nil,
        };

        self.environment.define(&stmt.name.lexeme, value);
        Ok(())
    }
}

fn evaluate_truthy(v: Literal) -> bool {
    match v {
        Literal::Nil => false,
        Literal::Boolean(b) => b,
        Literal::Number(_) => true,
        Literal::String(_) => true,
    }
}
