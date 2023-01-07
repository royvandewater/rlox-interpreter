use std::time::SystemTime;

use crate::environment::Environment;
use crate::stmt::{walk_stmt, Stmt, Stmts};
use crate::tokens::{Callable, Function, LoxCallable, TokenType};
use crate::{expr, tokens::Literal};
use crate::{expr::*, stmt};

use Literal as L;
use TokenType as TT;

enum Error {
    ReturnValue(Literal),
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

    pub(crate) fn interpret(&self, env_ref: EnvRef, statements: Stmts) -> Result<(), Vec<String>> {
        for statement in statements {
            match self.execute(env_ref.clone(), statement) {
                Ok(_) => (),
                Err(e) => {
                    return match e {
                        ReturnValue(v) => Err(vec![format!("Unexpected return value: {}", v)]),
                        SingleError(e) => Err(vec![e]),
                    }
                }
            }
        }

        Ok(())
    }

    fn execute(&self, environment: EnvRef, statement: Stmt) -> Result<(), Error> {
        walk_stmt(self, environment, statement)
    }

    fn evaluate(&self, environment: EnvRef, expression: expr::Expr) -> Result<Literal, Error> {
        walk_expr(self, environment, expression)
    }

    fn execute_block<'a>(&self, environment: EnvRef, statements: Vec<Stmt>) -> Result<(), Error> {
        for statement in statements {
            self.execute(environment.clone(), statement)?;
        }

        Ok(())
    }

    fn call(
        &self,
        _env_ref: EnvRef,
        callable: LoxCallable,
        arguments: Vec<Literal>,
    ) -> Result<Literal, Error> {
        if callable.arity() != arguments.len() {
            return Err(SingleError(format!(
                "Expected {} arguments but got {}.",
                callable.arity(),
                arguments.len()
            )));
        }

        match callable.callable {
            Callable::Native(n) => Ok(n()),
            Callable::Function(f) => {
                let env_ref = Environment::with_enclosing(f.env_ref.clone());

                for (param, arg) in f.params.iter().zip(arguments) {
                    env_ref.borrow_mut().define(&param.lexeme, arg);
                }

                match self.execute_block(env_ref, f.body) {
                    Ok(_) => Ok(Literal::Nil),
                    Err(e) => match e {
                        ReturnValue(value) => Ok(value),
                        e => Err(e),
                    },
                }
            }
        }
    }
}

impl expr::Visitor<Result<Literal, Error>> for Interpreter {
    fn visit_assign(&self, env_ref: EnvRef, expression: AssignExpr) -> Result<Literal, Error> {
        let name = &expression.name.lexeme.to_string();
        let value = self.evaluate(env_ref.clone(), *expression.value)?;

        match env_ref.borrow_mut().assign(&name, value.clone()) {
            Ok(_) => Ok(value),
            Err(e) => Err(Error::SingleError(e)),
        }
    }

    fn visit_binary(&self, env_ref: EnvRef, expr: BinaryExpr) -> Result<Literal, Error> {
        let left = self.evaluate(env_ref.clone(), *expr.left)?;
        let right = self.evaluate(env_ref.clone(), *expr.right)?;

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

            (l, _, r) => Err(SingleError(format!(
                "Unsupported types for binary operation: {} {} {}",
                l, expr.operator.lexeme, r
            ))),
        }
    }

    fn visit_call(&self, env_ref: EnvRef, expr: CallExpr) -> Result<Literal, Error> {
        let callee = self.evaluate(env_ref.clone(), *expr.callee)?;

        let mut arguments: Vec<Literal> = Vec::new();

        for arg in expr.arguments {
            arguments.push(self.evaluate(env_ref.clone(), arg)?);
        }

        match callee {
            L::Callable(f) => self.call(env_ref, f, arguments),
            _ => Err(SingleError(format!(
                "visit_call called with non function literal callee"
            ))),
        }
    }

    fn visit_grouping(&self, env_ref: EnvRef, expr: GroupingExpr) -> Result<Literal, Error> {
        self.evaluate(env_ref, *expr.expression)
    }

    fn visit_literal(&self, _env_ref: EnvRef, expr: LiteralExpr) -> Result<Literal, Error> {
        Ok(expr.value)
    }

    fn visit_logical(&self, env_ref: EnvRef, expr: LogicalExpr) -> Result<Literal, Error> {
        let left = self.evaluate(env_ref.clone(), *expr.left)?;

        match (evaluate_truthy(&left), expr.operator.token_type) {
            (true, TokenType::And) => self.evaluate(env_ref, *expr.right),
            (false, TokenType::And) => Ok(left),
            (true, TokenType::Or) => Ok(left),
            (false, TokenType::Or) => self.evaluate(env_ref, *expr.right),
            _ => Err(SingleError(format!(
                "visit_logical called with non and/or token: {}",
                expr.operator
            ))),
        }
    }

    fn visit_variable(&self, env_ref: EnvRef, expr: VariableExpr) -> Result<Literal, Error> {
        match env_ref.borrow_mut().get(&expr.name.lexeme) {
            None => Err(SingleError(format!(
                "variable with name '{}' not defined",
                &expr.name.lexeme
            ))),
            Some(literal) => Ok(literal),
        }
    }

    fn visit_unary(&self, env_ref: EnvRef, expr: UnaryExpr) -> Result<Literal, Error> {
        let right = self.evaluate(env_ref, *expr.right)?;

        match (expr.operator.token_type, right) {
            (TokenType::Bang, v) => Ok(Literal::Boolean(!evaluate_truthy(&v))),
            (TokenType::Minus, Literal::Number(n)) => Ok(Literal::Number(-1.0 * n)),
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

impl stmt::Visitor<Result<(), Error>> for Interpreter {
    fn visit_block<'a>(&self, env_ref: EnvRef, stmt: stmt::BlockStmt) -> Result<(), Error> {
        let scope_ref = Environment::with_enclosing(env_ref);

        self.execute_block(scope_ref, stmt.statements)?;

        Ok(())
    }

    fn visit_expression(&self, env_ref: EnvRef, stmt: stmt::ExpressionStmt) -> Result<(), Error> {
        self.evaluate(env_ref, *stmt.expression).map(|_| ())
    }

    fn visit_function(&self, env_ref: EnvRef, stmt: stmt::FunctionStmt) -> Result<(), Error> {
        let function = LoxCallable::new(
            stmt.name.lexeme.clone(),
            Callable::Function(Function::new(stmt.body, stmt.params, env_ref.clone())),
        );

        env_ref
            .borrow_mut()
            .define(&stmt.name.lexeme, Literal::Callable(function));

        Ok(())
    }

    fn visit_if(&self, env_ref: EnvRef, stmt: stmt::IfStmt) -> Result<(), Error> {
        let condition_result = self.evaluate(env_ref.clone(), *stmt.condition)?;

        match evaluate_truthy(&condition_result) {
            true => self.execute(env_ref, *stmt.then_branch),
            false => self.execute(env_ref, *stmt.else_branch),
        }
    }

    fn visit_print(&self, env_ref: EnvRef, stmt: stmt::PrintStmt) -> Result<(), Error> {
        let value = self.evaluate(env_ref, *stmt.expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_return(&self, env_ref: EnvRef, stmt: stmt::ReturnStmt) -> Result<(), Error> {
        Err(ReturnValue(self.evaluate(env_ref, *stmt.value)?))
    }

    fn visit_var(&self, env_ref: EnvRef, stmt: stmt::VarStmt) -> Result<(), Error> {
        let value = match stmt.initializer {
            Some(expression) => self.evaluate(env_ref.clone(), expression)?,
            None => Literal::Nil,
        };

        env_ref.borrow_mut().define(&stmt.name.lexeme, value);
        Ok(())
    }

    fn visit_while(&self, env_ref: EnvRef, stmt: stmt::WhileStmt) -> Result<(), Error> {
        loop {
            let condition_result = self.evaluate(env_ref.clone(), *stmt.condition.clone())?;

            if !evaluate_truthy(&condition_result) {
                return Ok(());
            }

            self.execute(env_ref.clone(), *stmt.body.clone())?;
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
