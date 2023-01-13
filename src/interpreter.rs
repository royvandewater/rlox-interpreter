use std::collections::BTreeMap;

use crate::environment::EnvRef;
use crate::expr::*;
use crate::resolver::Locals;
use crate::stmt::*;
use crate::tokens::{Callable, Class, Function, LoxCallable, LoxInstance, TokenType};
use crate::{expr, tokens::Literal};

use Literal as L;
use TokenType as TT;

#[derive(Debug)]
enum Error {
    ReturnValue(Literal),
    SingleError(String),
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::SingleError(e)
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Error::SingleError(e.to_string())
    }
}

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use Error::ReturnValue;
use Error::SingleError;

pub(crate) fn interpret(
    env: EnvRef,
    locals: Locals,
    statements: &Vec<Stmt>,
) -> Result<(), Vec<String>> {
    Interpreter::new(locals).interpret(env, statements)
}

struct Interpreter {
    locals: Locals,
}

impl Interpreter {
    fn new(locals: Locals) -> Interpreter {
        Interpreter { locals }
    }

    fn interpret(&self, env: EnvRef, statements: &Vec<Stmt>) -> Result<(), Vec<String>> {
        for statement in statements.iter() {
            match self.execute(env.clone(), statement) {
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

    fn execute(&self, environment: EnvRef, statement: &Stmt) -> Result<(), Error> {
        walk_stmt(self, environment, statement)
    }

    fn evaluate(&self, environment: EnvRef, expression: &Expr) -> Result<Literal, Error> {
        walk_expr(self, environment, expression)
    }

    fn execute_block<'a>(&self, environment: EnvRef, statements: &Vec<Stmt>) -> Result<(), Error> {
        for statement in statements {
            self.execute(environment.clone(), statement)?;
        }

        Ok(())
    }

    fn call(
        &self,
        _env: EnvRef,
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

        match &callable.callable {
            Callable::Class(c) => Ok(Literal::ClassInstance(LoxInstance::new(c.clone()))),
            Callable::Function(f) => {
                let mut env = EnvRef::with_enclosing(f.env_ref.clone());

                for (param, arg) in f.params.iter().zip(arguments) {
                    env.define(&param.lexeme, arg);
                }

                match self.execute_block(env, &f.body) {
                    Ok(_) => Ok(Literal::Nil),
                    Err(e) => match e {
                        ReturnValue(value) => Ok(value),
                        e => Err(e),
                    },
                }
            }
            Callable::Native(n) => Ok(n()),
        }
    }

    fn look_up_variable(
        &self,
        env: EnvRef,
        name: &str,
        expr: &VariableExpr,
    ) -> Result<Literal, Error> {
        let value = match self.locals.get(&Expr::Variable(expr.clone())) {
            None => env.get_global(name),
            Some(distance) => env.get_at_distance(distance, name),
        };

        match value {
            None => Err(SingleError(format!(
                "variable with name '{}' not defined",
                &expr.name.lexeme
            ))),
            Some(literal) => Ok(literal),
        }
    }
}

impl expr::Visitor<EnvRef, Result<Literal, Error>> for Interpreter {
    fn visit_assign(&self, mut env: EnvRef, expression: &AssignExpr) -> Result<Literal, Error> {
        let name = &expression.name.lexeme.to_string();
        let value = self.evaluate(env.clone(), &expression.value)?;

        match self.locals.get(&Expr::Assign(expression.clone())) {
            Some(distance) => env.assign_at_distance(distance, name, value.clone()),
            None => env.assign_global(name, value.clone()),
        }?;

        Ok(value)
    }

    fn visit_binary(&self, env: EnvRef, expr: &BinaryExpr) -> Result<Literal, Error> {
        let left = self.evaluate(env.clone(), &expr.left)?;
        let right = self.evaluate(env.clone(), &expr.right)?;

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

    fn visit_call(&self, env: EnvRef, expr: &CallExpr) -> Result<Literal, Error> {
        let callee = self.evaluate(env.clone(), &expr.callee)?;

        let mut arguments: Vec<Literal> = Vec::new();

        for arg in &expr.arguments {
            arguments.push(self.evaluate(env.clone(), &arg)?);
        }

        match callee {
            L::Callable(f) => self.call(env, f, arguments),
            _ => Err(SingleError(format!(
                "visit_call called with non function literal callee"
            ))),
        }
    }

    fn visit_get(&self, env: EnvRef, expr: &GetExpr) -> Result<Literal, Error> {
        match self.evaluate(env, &expr.object)? {
            L::ClassInstance(i) => Ok(i.get(&expr.name.lexeme)?),
            _ => Err(Error::SingleError(
                "Only instances have properties.".to_string(),
            )),
        }
    }

    fn visit_grouping(&self, env: EnvRef, expr: &GroupingExpr) -> Result<Literal, Error> {
        self.evaluate(env, &expr.expression)
    }

    fn visit_literal(&self, _env: EnvRef, expr: &LiteralExpr) -> Result<Literal, Error> {
        Ok(expr.value.clone())
    }

    fn visit_logical(&self, env: EnvRef, expr: &LogicalExpr) -> Result<Literal, Error> {
        let left = self.evaluate(env.clone(), &expr.left)?;

        match (evaluate_truthy(&left), expr.operator.token_type) {
            (true, TokenType::And) => self.evaluate(env, &expr.right),
            (false, TokenType::And) => Ok(left),
            (true, TokenType::Or) => Ok(left),
            (false, TokenType::Or) => self.evaluate(env, &expr.right),
            _ => Err(SingleError(format!(
                "visit_logical called with non and/or token: {}",
                expr.operator
            ))),
        }
    }

    fn visit_set(&self, env: EnvRef, expr: &SetExpr) -> Result<Literal, Error> {
        let mut object = match self.evaluate(env.clone(), &expr.object)? {
            L::ClassInstance(o) => o,
            _ => Err("Only instances have fields.")?,
        };

        let value = self.evaluate(env, &expr.value)?;
        object.set(&expr.name.lexeme, value.clone());
        Ok(value)
    }

    fn visit_unary(&self, env: EnvRef, expr: &UnaryExpr) -> Result<Literal, Error> {
        let right = self.evaluate(env, &expr.right)?;

        match (expr.operator.token_type, right) {
            (TokenType::Bang, v) => Ok(Literal::Boolean(!evaluate_truthy(&v))),
            (TokenType::Minus, Literal::Number(n)) => {
                Ok(Literal::Number(n * Decimal::from_isize(-1).unwrap()))
            }
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

    fn visit_variable(&self, env: EnvRef, expr: &VariableExpr) -> Result<Literal, Error> {
        self.look_up_variable(env, &expr.name.lexeme, expr)
    }
}

impl crate::stmt::Visitor<EnvRef, Result<(), Error>> for Interpreter {
    fn visit_block<'a>(&self, env: EnvRef, stmt: &BlockStmt) -> Result<(), Error> {
        let scope_ref = EnvRef::with_enclosing(env);

        self.execute_block(scope_ref, &stmt.statements)?;

        Ok(())
    }

    fn visit_class(&self, mut env: EnvRef, stmt: &ClassStmt) -> Result<(), Error> {
        let name = stmt.name.lexeme.clone();
        env.define(&name, L::Nil);

        let mut methods: BTreeMap<String, Function> = BTreeMap::new();

        for method in stmt.methods.iter() {
            let function = Function::new(method.body.clone(), method.params.clone(), env.clone());
            methods.insert(method.name.lexeme.clone(), function);
        }

        let class = LoxCallable::new(
            name.clone(),
            Callable::Class(Class::new(name.clone(), methods)),
        );
        env.assign(&name, Literal::Callable(class))?;
        Ok(())
    }

    fn visit_expression(&self, env: EnvRef, stmt: &ExpressionStmt) -> Result<(), Error> {
        self.evaluate(env, &stmt.expression).map(|_| ())
    }

    fn visit_function(&self, mut env: EnvRef, stmt: &FunctionStmt) -> Result<(), Error> {
        let function = LoxCallable::new(
            stmt.name.lexeme.clone(),
            Callable::Function(Function::new(
                stmt.body.clone(),
                stmt.params.clone(),
                env.clone(),
            )),
        );

        env.define(&stmt.name.lexeme, Literal::Callable(function));

        Ok(())
    }

    fn visit_if(&self, env: EnvRef, stmt: &IfStmt) -> Result<(), Error> {
        let condition_result = self.evaluate(env.clone(), &stmt.condition)?;

        match evaluate_truthy(&condition_result) {
            true => self.execute(env, &stmt.then_branch),
            false => self.execute(env, &stmt.else_branch),
        }
    }

    fn visit_print(&self, env: EnvRef, stmt: &PrintStmt) -> Result<(), Error> {
        let value = self.evaluate(env, &stmt.expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_return(&self, env: EnvRef, stmt: &ReturnStmt) -> Result<(), Error> {
        Err(ReturnValue(self.evaluate(env, &stmt.value)?))
    }

    fn visit_var(&self, mut env: EnvRef, stmt: &VarStmt) -> Result<(), Error> {
        let value = self.evaluate(env.clone(), &stmt.initializer)?;
        env.define(&stmt.name.lexeme, value);
        Ok(())
    }

    fn visit_while(&self, env: EnvRef, stmt: &WhileStmt) -> Result<(), Error> {
        loop {
            let condition_result = self.evaluate(env.clone(), &stmt.condition)?;

            if !evaluate_truthy(&condition_result) {
                return Ok(());
            }

            self.execute(env.clone(), &stmt.body)?;
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
