use std::time::SystemTime;

use crate::environment::Environment;
use crate::stmt::{walk_stmt, Stmt, Stmts};
use crate::tokens::{Callable, LoxCallable, TokenType};
use crate::{expr, tokens::Literal};
use crate::{expr::*, stmt};

use Literal as L;
use TokenType as TT;

enum Error {
    ReturnValue((Environment, Literal)),
    SingleError(String),
}

use Error::ReturnValue;
use Error::SingleError;

pub(crate) struct Interpreter;

pub(crate) fn add_clock_to_environment(mut environment: Environment) -> Environment {
    environment.define(
        "clock",
        Literal::Callable(LoxCallable::new(
            "clock".to_string(),
            Callable::Native(|| {
                let now = SystemTime::now();
                let duration = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
                Literal::Number(duration.as_secs_f64())
            }),
        )),
    );
    environment
}

impl Interpreter {
    pub(crate) fn new() -> Interpreter {
        Interpreter {}
    }

    pub(crate) fn interpret(
        &self,
        mut environment: Environment,
        statements: Stmts,
    ) -> Result<Environment, Vec<String>> {
        for statement in statements {
            match self.execute(environment, statement) {
                Ok(e) => environment = e,
                Err(e) => {
                    return match e {
                        ReturnValue((_, v)) => Err(vec![format!("Unexpected return value: {}", v)]),
                        SingleError(e) => Err(vec![e]),
                    }
                }
            }
        }

        Ok(environment)
    }

    fn execute(&self, environment: Environment, statement: Stmt) -> Result<Environment, Error> {
        walk_stmt(self, environment, statement)
    }

    fn evaluate(
        &self,
        environment: Environment,
        expression: expr::Expr,
    ) -> Result<(Environment, Literal), Error> {
        walk_expr(self, environment, expression)
    }

    fn execute_block<'a>(
        &self,
        mut environment: Environment,
        statements: Vec<Stmt>,
    ) -> Result<Environment, Error> {
        for statement in statements {
            environment = self.execute(environment, statement)?;
        }

        Ok(environment)
    }

    fn call(
        &self,
        environment: Environment,
        callable: LoxCallable,
        arguments: Vec<Literal>,
    ) -> Result<(Environment, Literal), Error> {
        if callable.arity() != arguments.len() {
            return Err(SingleError(format!(
                "Expected {} arguments but got {}.",
                callable.arity(),
                arguments.len()
            )));
        }

        match callable.callable {
            Callable::Native(n) => Ok((environment, n())),
            Callable::Function((block, params)) => {
                let mut environment = Environment::with_enclosing(environment);

                for (param, arg) in params.iter().zip(arguments) {
                    environment.define(&param.lexeme, arg);
                }

                match self.execute_block(environment, block) {
                    Ok(environment) => Ok((environment, Literal::Nil)),
                    Err(e) => match e {
                        ReturnValue((environment, value)) => Ok((environment, value)),
                        e => Err(e),
                    },
                }
            }
        }
    }
}

impl expr::Visitor<Result<(Environment, Literal), Error>> for Interpreter {
    fn visit_assign(
        &self,
        environment: Environment,
        expression: AssignExpr,
    ) -> Result<(Environment, Literal), Error> {
        let name = &expression.name.lexeme.to_string();
        let (mut environment, value) = self.evaluate(environment, *expression.value)?;

        match environment.assign(&name, value.clone()) {
            Ok(_) => Ok((environment, value)),
            Err(e) => Err(Error::SingleError(e)),
        }
    }

    fn visit_binary(
        &self,
        e: Environment,
        expr: BinaryExpr,
    ) -> Result<(Environment, Literal), Error> {
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

            (l, _, r) => Err(SingleError(format!(
                "Unsupported types for binary operation: {} {} {}",
                l, expr.operator.lexeme, r
            ))),
        }
    }

    fn visit_call(
        &self,
        environment: Environment,
        expr: CallExpr,
    ) -> Result<(Environment, Literal), Error> {
        let (mut environment, callee) = self.evaluate(environment, *expr.callee)?;

        let mut arguments: Vec<Literal> = Vec::new();

        for arg in expr.arguments {
            let (e, value) = self.evaluate(environment, arg)?;
            environment = e;
            arguments.push(value);
        }

        match callee {
            L::Callable(f) => self.call(environment, f, arguments),
            _ => Err(SingleError(format!(
                "visit_call called with non function literal callee"
            ))),
        }
    }

    fn visit_grouping(
        &self,
        environment: Environment,
        expr: GroupingExpr,
    ) -> Result<(Environment, Literal), Error> {
        self.evaluate(environment, *expr.expression)
    }

    fn visit_literal(
        &self,
        environment: Environment,
        expr: LiteralExpr,
    ) -> Result<(Environment, Literal), Error> {
        Ok((environment, expr.value))
    }

    fn visit_logical(
        &self,
        environment: Environment,
        expr: LogicalExpr,
    ) -> Result<(Environment, Literal), Error> {
        let (environment, left) = self.evaluate(environment, *expr.left)?;

        match (evaluate_truthy(&left), expr.operator.token_type) {
            (true, TokenType::And) => self.evaluate(environment, *expr.right),
            (false, TokenType::And) => Ok((environment, left)),
            (true, TokenType::Or) => Ok((environment, left)),
            (false, TokenType::Or) => self.evaluate(environment, *expr.right),
            _ => Err(SingleError(format!(
                "visit_logical called with non and/or token: {}",
                expr.operator
            ))),
        }
    }

    fn visit_variable(
        &self,
        environment: Environment,
        expr: VariableExpr,
    ) -> Result<(Environment, Literal), Error> {
        match environment.get(&expr.name.lexeme) {
            None => Err(SingleError(format!(
                "variable with name '{}' not defined",
                &expr.name.lexeme
            ))),
            Some(literal) => Ok((environment, literal)),
        }
    }

    fn visit_unary(
        &self,
        e: Environment,
        expr: UnaryExpr,
    ) -> Result<(Environment, Literal), Error> {
        let (e, right) = self.evaluate(e, *expr.right)?;

        match (expr.operator.token_type, right) {
            (TokenType::Bang, v) => Ok((e, Literal::Boolean(!evaluate_truthy(&v)))),
            (TokenType::Minus, Literal::Number(n)) => Ok((e, Literal::Number(-1.0 * n))),
            (TokenType::Minus, v) => Err(SingleError(format!(
                "Invalid attempt to perform numerical negation on non-number: {}",
                v
            ))),
            (_, v) => Err(SingleError(format!(
                "The value '{}' does not support the unary operation '{}'",
                v, expr.operator.lexeme
            ))),
        }
    }
}

impl stmt::Visitor<Result<Environment, Error>> for Interpreter {
    fn visit_block<'a>(
        &self,
        environment: Environment,
        stmt: stmt::BlockStmt,
    ) -> Result<Environment, Error> {
        let mut scope = Environment::with_enclosing(environment);

        scope = self.execute_block(scope, stmt.statements)?;

        Ok(scope.enclosing().unwrap())
    }

    fn visit_expression(
        &self,
        environment: Environment,
        stmt: stmt::ExpressionStmt,
    ) -> Result<Environment, Error> {
        let (environment, _) = self.evaluate(environment, *stmt.expression)?;
        Ok(environment)
    }

    fn visit_function(
        &self,
        mut environment: Environment,
        stmt: stmt::FunctionStmt,
    ) -> Result<Environment, Error> {
        let function = LoxCallable::new(
            stmt.name.lexeme.clone(),
            Callable::Function((stmt.body, stmt.params)),
        );

        environment.define(&stmt.name.lexeme, Literal::Callable(function));
        Ok(environment)
    }

    fn visit_if(&self, environment: Environment, stmt: stmt::IfStmt) -> Result<Environment, Error> {
        let (environment, condition_result) = self.evaluate(environment, *stmt.condition)?;

        match evaluate_truthy(&condition_result) {
            true => self.execute(environment, *stmt.then_branch),
            false => self.execute(environment, *stmt.else_branch),
        }
    }

    fn visit_print(
        &self,
        environment: Environment,
        stmt: stmt::PrintStmt,
    ) -> Result<Environment, Error> {
        let (environment, value) = self.evaluate(environment, *stmt.expression)?;
        println!("{}", value);
        Ok(environment)
    }

    fn visit_return(
        &self,
        environment: Environment,
        stmt: stmt::ReturnStmt,
    ) -> Result<Environment, Error> {
        Err(ReturnValue(self.evaluate(environment, *stmt.value)?))
    }

    fn visit_var(
        &self,
        environment: Environment,
        stmt: stmt::VarStmt,
    ) -> Result<Environment, Error> {
        let (mut environment, value) = match stmt.initializer {
            Some(expression) => self.evaluate(environment, expression)?,
            None => (environment, Literal::Nil),
        };

        environment.define(&stmt.name.lexeme, value);
        Ok(environment)
    }

    fn visit_while(
        &self,
        mut environment: Environment,
        stmt: stmt::WhileStmt,
    ) -> Result<Environment, Error> {
        loop {
            let (e, condition_result) = self.evaluate(environment, *stmt.condition.clone())?;
            environment = e;

            if !evaluate_truthy(&condition_result) {
                return Ok(environment);
            }

            environment = self.execute(environment, *stmt.body.clone())?;
        }
    }
}

fn evaluate_truthy(v: &Literal) -> bool {
    match v {
        Literal::Nil => false,
        Literal::Boolean(b) => *b,
        _ => true,
    }
}
