use crate::environment::Environment;
use crate::stmt::{walk_stmt, Stmt, Stmts};
use crate::tokens::TokenType;
use crate::{expr, tokens::Literal};
use crate::{expr::*, stmt};

use Literal as L;
use TokenType as TT;

pub(crate) struct Interpreter;

impl Interpreter {
    pub(crate) fn new() -> Interpreter {
        Interpreter {}
    }

    pub(crate) fn interpret(&self, statements: Stmts) -> Result<(), Vec<String>> {
        let mut environment = Environment::new();

        for statement in statements {
            environment = self.execute(environment, statement)?;
        }

        Ok(())
    }

    fn execute(
        &self,
        environment: Environment,
        statement: Stmt,
    ) -> Result<Environment, Vec<String>> {
        walk_stmt(self, environment, statement)
    }

    fn evaluate(
        &self,
        environment: Environment,
        expression: expr::Expr,
    ) -> Result<(Environment, Literal), Vec<String>> {
        walk_expr(self, environment, expression)
    }

    fn execute_block<'a>(
        &self,
        mut environment: Environment,
        statements: Vec<Stmt>,
    ) -> Result<Environment, Vec<String>> {
        for statement in statements {
            environment = self.execute(environment, statement)?;
        }

        Ok(environment)
    }
}

impl expr::Visitor<Result<(Environment, Literal), Vec<String>>> for Interpreter {
    fn visit_binary(
        &self,
        e: Environment,
        expr: BinaryExpr,
    ) -> Result<(Environment, Literal), Vec<String>> {
        let (e, left) = self.evaluate(e, *expr.left)?;
        let (e, right) = self.evaluate(e, *expr.right)?;

        let operator = expr.operator.token_type;

        match (left, operator, right) {
            // Math
            (L::Number(l), TT::Plus, L::Number(r)) => Ok((e, L::Number(l + r))),
            (L::Number(l), TT::Minus, L::Number(r)) => Ok((e, L::Number(l - r))),
            (L::Number(l), TT::Slash, L::Number(r)) => Ok((e, L::Number(l / r))),
            (L::Number(l), TT::Star, L::Number(r)) => Ok((e, L::Number(l * r))),

            // String concatenation
            (L::String(l), TT::Plus, L::String(r)) => Ok((e, L::String(format!("{}{}", l, r)))),

            // Comparison operators
            (L::Number(l), TT::Greater, L::Number(r)) => Ok((e, L::Boolean(l > r))),
            (L::Number(l), TT::GreaterEqual, L::Number(r)) => Ok((e, L::Boolean(l >= r))),
            (L::Number(l), TT::Less, L::Number(r)) => Ok((e, L::Boolean(l < r))),
            (L::Number(l), TT::LessEqual, L::Number(r)) => Ok((e, L::Boolean(l <= r))),

            // Equality operators
            (l, TT::EqualEqual, r) => Ok((e, L::Boolean(l == r))),
            (l, TT::BangEqual, r) => Ok((e, L::Boolean(l != r))),

            (l, _, r) => Err(vec![format!(
                "Unsupported types for binary operation: {} {} {}",
                l, expr.operator.lexeme, r
            )]),
        }
    }

    fn visit_grouping(
        &self,
        environment: Environment,
        expr: GroupingExpr,
    ) -> Result<(Environment, Literal), Vec<String>> {
        self.evaluate(environment, *expr.expression)
    }

    fn visit_literal(
        &self,
        environment: Environment,
        expr: LiteralExpr,
    ) -> Result<(Environment, Literal), Vec<String>> {
        Ok((environment, expr.value))
    }

    fn visit_unary(
        &self,
        e: Environment,
        expr: UnaryExpr,
    ) -> Result<(Environment, Literal), Vec<String>> {
        let (e, right) = self.evaluate(e, *expr.right)?;

        match (expr.operator.token_type, right) {
            (TokenType::Bang, v) => Ok((e, Literal::Boolean(!evaluate_truthy(v)))),
            (TokenType::Minus, Literal::Number(n)) => Ok((e, Literal::Number(-1.0 * n))),
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

    fn visit_variable(
        &self,
        environment: Environment,
        expr: VariableExpr,
    ) -> Result<(Environment, Literal), Vec<String>> {
        match environment.get(&expr.name.lexeme) {
            None => Err(vec![format!(
                "variable with name '{}' not defined",
                &expr.name.lexeme
            )]),
            Some(literal) => Ok((environment, literal)),
        }
    }

    fn visit_assign(
        &self,
        environment: Environment,
        expression: AssignExpr,
    ) -> Result<(Environment, Literal), Vec<String>> {
        let name = &expression.name.lexeme.to_string();
        let (mut environment, value) = self.evaluate(environment, *expression.value)?;

        match environment.assign(&name, value.clone()) {
            Ok(_) => Ok((environment, value)),
            Err(e) => Err(vec![e]),
        }
    }
}

impl stmt::Visitor<Result<Environment, Vec<String>>> for Interpreter {
    fn visit_block<'a>(
        &self,
        environment: Environment,
        stmt: stmt::BlockStmt,
    ) -> Result<Environment, Vec<String>> {
        let mut scope = Environment::with_enclosing(environment);

        scope = self.execute_block(scope, stmt.statements)?;

        Ok(scope.enclosing().unwrap())
    }

    fn visit_expression(
        &self,
        environment: Environment,
        stmt: stmt::ExpressionStmt,
    ) -> Result<Environment, Vec<String>> {
        let (environment, _) = self.evaluate(environment, *stmt.expression)?;
        Ok(environment)
    }

    fn visit_print(
        &self,
        environment: Environment,
        stmt: stmt::PrintStmt,
    ) -> Result<Environment, Vec<String>> {
        let (environment, value) = self.evaluate(environment, *stmt.expression)?;
        println!("{}", value);
        Ok(environment)
    }

    fn visit_var(
        &self,
        environment: Environment,
        stmt: stmt::VarStmt,
    ) -> Result<Environment, Vec<String>> {
        let (mut environment, value) = match stmt.initializer {
            Some(expression) => self.evaluate(environment, expression)?,
            None => (environment, Literal::Nil),
        };

        environment.define(&stmt.name.lexeme, value);
        Ok(environment)
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
