mod environments;

use std::collections::BTreeMap;

use crate::environment::Environment;
use crate::expr::*;
use crate::resolver::Locals;
use crate::stmt::*;
use crate::tokens::{Callable, Class, Function, LoxCallable, LoxInstance, TokenType};
use crate::{expr, tokens::Literal};

use environments::Environments;

use Literal as L;
use TokenType as TT;

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

#[derive(Debug)]
enum Error {
    ReturnValue(Literal),
    SingleError(String),
}

use Error::ReturnValue;
use Error::SingleError;

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

pub(crate) fn interpret(
    globals: Environment,
    locals: Locals,
    statements: &Vec<Stmt>,
) -> Result<(), Vec<String>> {
    Interpreter::new(globals, locals).interpret(statements)
}

struct Interpreter {
    environments: Environments,
}

impl Interpreter {
    fn new(globals: Environment, locals: Locals) -> Interpreter {
        Interpreter {
            environments: Environments::new(globals, locals),
        }
    }

    fn interpret(&self, statements: &Vec<Stmt>) -> Result<(), Vec<String>> {
        for statement in statements.iter() {
            match self.execute(statement) {
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

    fn execute(&self, statement: &Stmt) -> Result<(), Error> {
        walk_stmt(self, statement)
    }

    fn evaluate(&self, expression: &Expr) -> Result<Literal, Error> {
        walk_expr(self, expression)
    }

    fn execute_block<'a>(&self, statements: &Vec<Stmt>) -> Result<(), Error> {
        for statement in statements {
            self.execute(statement)?;
        }

        Ok(())
    }

    fn call(&self, callable: LoxCallable, arguments: Vec<Literal>) -> Result<Literal, Error> {
        if callable.arity() != arguments.len() {
            return Err(SingleError(format!(
                "Expected {} arguments but got {}.",
                callable.arity(),
                arguments.len()
            )));
        }

        match &callable.callable {
            Callable::Class(c) => {
                let instance = LoxInstance::new(c.clone());
                if let Some(initializer) = instance.find_method("init") {
                    let function = initializer.bind(instance.clone());
                    self.call(
                        LoxCallable::new("init".to_string(), Callable::Function(function)),
                        arguments,
                    )?;
                }
                Ok(Literal::ClassInstance(instance))
            }
            Callable::Function(f) => {
                let mut env = Environment::with_enclosing(f.env.clone());

                for (param, arg) in f.params.iter().zip(arguments) {
                    env.define(&param.lexeme, arg);
                }

                self.environments.push_scope(env);

                let result = match self.execute_block(&f.body) {
                    Ok(_) if f.is_initializer => {
                        Ok(self.environments.get_at_distance(0, "this").unwrap())
                    }
                    Ok(_) => Ok(Literal::Nil),
                    Err(e) => match e {
                        ReturnValue(value) => Ok(value),
                        e => Err(e),
                    },
                };

                self.environments.pop_scope();
                result
            }
            Callable::Native(n) => Ok(n()),
        }
    }

    fn look_up_variable(&self, name: &str, expr: &VariableExpr) -> Result<Literal, Error> {
        Ok(self.environments.look_up_variable(name, expr)?)
    }
}

impl expr::Visitor<Result<Literal, Error>> for Interpreter {
    fn visit_assign(&self, expression: &AssignExpr) -> Result<Literal, Error> {
        let name = &expression.name.lexeme.to_string();
        let value = self.evaluate(&expression.value)?;

        self.environments
            .assign_expression(expression.clone(), name, value.clone())?;

        Ok(value)
    }

    fn visit_binary(&self, expr: &BinaryExpr) -> Result<Literal, Error> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

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

    fn visit_call(&self, expr: &CallExpr) -> Result<Literal, Error> {
        let callee = self.evaluate(&expr.callee)?;

        let mut arguments: Vec<Literal> = Vec::new();

        for arg in &expr.arguments {
            arguments.push(self.evaluate(&arg)?);
        }

        match callee {
            L::Callable(f) => self.call(f, arguments),
            _ => Err(SingleError(format!(
                "visit_call called with non function literal callee"
            ))),
        }
    }

    fn visit_get(&self, expr: &GetExpr) -> Result<Literal, Error> {
        match self.evaluate(&expr.object)? {
            L::ClassInstance(i) => Ok(i.get(&expr.name.lexeme)?),
            _ => Err(Error::SingleError(
                "Only instances have properties.".to_string(),
            )),
        }
    }

    fn visit_grouping(&self, expr: &GroupingExpr) -> Result<Literal, Error> {
        self.evaluate(&expr.expression)
    }

    fn visit_literal(&self, expr: &LiteralExpr) -> Result<Literal, Error> {
        Ok(expr.value.clone())
    }

    fn visit_logical(&self, expr: &LogicalExpr) -> Result<Literal, Error> {
        let left = self.evaluate(&expr.left)?;

        match (evaluate_truthy(&left), expr.operator.token_type) {
            (true, TokenType::And) => self.evaluate(&expr.right),
            (false, TokenType::And) => Ok(left),
            (true, TokenType::Or) => Ok(left),
            (false, TokenType::Or) => self.evaluate(&expr.right),
            _ => Err(SingleError(format!(
                "visit_logical called with non and/or token: {}",
                expr.operator
            ))),
        }
    }

    fn visit_set(&self, expr: &SetExpr) -> Result<Literal, Error> {
        let mut object = match self.evaluate(&expr.object)? {
            L::ClassInstance(o) => o,
            _ => Err("Only instances have fields.")?,
        };

        let value = self.evaluate(&expr.value)?;
        object.set(&expr.name.lexeme, value.clone());
        Ok(value)
    }

    fn visit_this(&self, expr: &ThisExpr) -> Result<Literal, Error> {
        self.look_up_variable(
            &expr.keyword.lexeme,
            &VariableExpr::new(expr.id, expr.keyword.clone()),
        )
    }

    fn visit_unary(&self, expr: &UnaryExpr) -> Result<Literal, Error> {
        let right = self.evaluate(&expr.right)?;

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

    fn visit_variable(&self, expr: &VariableExpr) -> Result<Literal, Error> {
        self.look_up_variable(&expr.name.lexeme, expr)
    }
}

impl crate::stmt::Visitor<Result<(), Error>> for Interpreter {
    fn visit_block<'a>(&self, stmt: &BlockStmt) -> Result<(), Error> {
        let scope = Environment::with_enclosing(self.environments.peek());

        self.environments.push_scope(scope);
        let result = self.execute_block(&stmt.statements);
        self.environments.pop_scope();
        result
    }

    fn visit_class(&self, stmt: &ClassStmt) -> Result<(), Error> {
        let superclass = match &stmt.superclass {
            None => None,
            Some(expression) => match self.evaluate(&Expr::Variable(expression.clone()))? {
                L::Callable(callable) => match callable.callable {
                    Callable::Class(class) => Some(LoxInstance::new(class)),
                    c => {
                        return Err(SingleError(format!(
                            "Superclass must be a class. got {:?}",
                            c
                        )))
                    }
                },
                c => {
                    return Err(SingleError(format!(
                        "Superclass must be a class. got {}",
                        c
                    )))
                }
            },
        };

        let name = stmt.name.lexeme.clone();
        self.environments.peek().define(&name, L::Nil);

        let mut methods: BTreeMap<String, Function> = BTreeMap::new();

        for method in stmt.methods.iter() {
            let body = method.body.clone();
            let params = method.params.clone();

            let function = match method.name.lexeme.as_str() {
                "init" => Function::new_initializer(body, params, self.environments.peek()),
                _ => Function::new(body, params, self.environments.peek()),
            };
            methods.insert(method.name.lexeme.clone(), function);
        }

        let class = LoxCallable::new(
            name.clone(),
            Callable::Class(Class::new(name.clone(), superclass, methods)),
        );
        self.environments.assign(&name, Literal::Callable(class))?;
        Ok(())
    }

    fn visit_expression(&self, stmt: &ExpressionStmt) -> Result<(), Error> {
        self.evaluate(&stmt.expression).map(|_| ())
    }

    fn visit_function(&self, stmt: &FunctionStmt) -> Result<(), Error> {
        let mut env = self.environments.peek();

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

    fn visit_if(&self, stmt: &IfStmt) -> Result<(), Error> {
        let condition_result = self.evaluate(&stmt.condition)?;

        match evaluate_truthy(&condition_result) {
            true => self.execute(&stmt.then_branch),
            false => self.execute(&stmt.else_branch),
        }
    }

    fn visit_print(&self, stmt: &PrintStmt) -> Result<(), Error> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_return(&self, stmt: &ReturnStmt) -> Result<(), Error> {
        Err(ReturnValue(self.evaluate(&stmt.value)?))
    }

    fn visit_var(&self, stmt: &VarStmt) -> Result<(), Error> {
        let mut env = self.environments.peek();
        let name = &stmt.name.lexeme;
        let value = self.evaluate(&stmt.initializer)?;
        env.define(name, value);
        Ok(())
    }

    fn visit_while(&self, stmt: &WhileStmt) -> Result<(), Error> {
        loop {
            let condition_result = self.evaluate(&stmt.condition)?;

            if !evaluate_truthy(&condition_result) {
                return Ok(());
            }

            self.execute(&stmt.body)?;
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
