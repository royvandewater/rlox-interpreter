use crate::expr::*;
use crate::tokens::TokenType;
use crate::{expr, tokens::Literal};

use Literal as L;
use TokenType as TT;

pub(crate) struct Interpreter;

impl Interpreter {
    pub(crate) fn new() -> Interpreter {
        Interpreter {}
    }

    pub(crate) fn interpret(&self, expression: expr::Expr) -> Result<(), Vec<String>> {
        let value = self.evaluate(expression)?;
        println!("{}", value);
        Ok(())
    }

    fn evaluate(&self, expression: expr::Expr) -> Result<Literal, Vec<String>> {
        walk_expr(self, expression)
    }
}

impl Visitor<Result<Literal, Vec<String>>> for Interpreter {
    fn visit_binary(&self, expr: BinaryExpr) -> Result<Literal, Vec<String>> {
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

    fn visit_grouping(&self, expr: GroupingExpr) -> Result<Literal, Vec<String>> {
        self.evaluate(*expr.expression)
    }

    fn visit_literal(&self, expr: LiteralExpr) -> Result<Literal, Vec<String>> {
        Ok(expr.value)
    }

    fn visit_unary(&self, expr: UnaryExpr) -> Result<Literal, Vec<String>> {
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
}

fn evaluate_truthy(v: Literal) -> bool {
    match v {
        Literal::Nil => false,
        Literal::Boolean(b) => b,
        Literal::Number(_) => true,
        Literal::String(_) => true,
    }
}
