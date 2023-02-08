use std::{cell::RefCell, collections::HashMap, slice::Iter};

use crate::{
    expr::{self, *},
    stmt::{self, *},
    tokens::Literal,
};

struct SingleError(String);

impl From<String> for SingleError {
    fn from(e: String) -> Self {
        SingleError(e)
    }
}

impl From<&str> for SingleError {
    fn from(e: &str) -> Self {
        SingleError(e.to_string())
    }
}

#[derive(Debug)]
pub(crate) struct Scopes(Vec<HashMap<String, bool>>);

impl Scopes {
    fn new() -> Scopes {
        Scopes(Vec::new())
    }

    fn begin_scope(&mut self) {
        self.0.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.0.pop();
    }

    fn declare(&mut self, name: String) {
        match self.0.last_mut() {
            None => (),
            Some(scope) => {
                scope.insert(name, false);
            }
        };
    }

    fn define(&mut self, name: String) {
        match self.0.last_mut() {
            None => (),
            Some(scope) => {
                scope.insert(name, true);
            }
        }
    }

    fn force_define(&mut self, name: String) {
        self.0.last_mut().unwrap().insert(name, true);
    }

    fn top_contains(&self, name: &str) -> bool {
        match self.0.last() {
            None => false,
            Some(map) => map.contains_key(name),
        }
    }

    fn get(&self, name: &str) -> Option<bool> {
        match self.0.last() {
            None => None,
            Some(map) => map.get(name).cloned(),
        }
    }

    fn iter(&self) -> Iter<HashMap<String, bool>> {
        self.0.iter()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Locals(HashMap<Expr, usize>);
impl Locals {
    fn new() -> Locals {
        Locals(HashMap::new())
    }

    pub(crate) fn get(&self, expression: &Expr) -> Option<usize> {
        self.0.get(expression).map(|i| *i)
    }

    fn resolve(&mut self, expression: Expr, i: usize) {
        self.0.insert(expression, i);
    }
}

pub(crate) fn resolve_locals(statements: &Vec<Stmt>) -> Result<Locals, Vec<String>> {
    let resolver = Resolver::new();
    resolver
        .resolve(statements)
        .map_err(prepend_resolver_error)?;
    Ok(resolver.locals.into_inner())
}

fn prepend_resolver_error(error: SingleError) -> Vec<String> {
    vec![format!("Resolver Error: {}", error.0)]
}

enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Clone, Copy)]
enum ClassType {
    None,
    Class,
    Subclass,
}

struct Resolver {
    locals: RefCell<Locals>,
    scopes: RefCell<Scopes>,
    current_function: RefCell<FunctionType>,
    current_class: RefCell<ClassType>,
}

impl Resolver {
    fn new() -> Resolver {
        Resolver {
            locals: RefCell::new(Locals::new()),
            scopes: RefCell::new(Scopes::new()),
            current_function: RefCell::new(FunctionType::None),
            current_class: RefCell::new(ClassType::None),
        }
    }

    fn begin_scope(&self) {
        self.scopes.borrow_mut().begin_scope()
    }

    fn end_scope(&self) {
        self.scopes.borrow_mut().end_scope()
    }

    fn force_define(&self, name: &str) {
        self.scopes.borrow_mut().force_define(name.to_string());
    }

    fn declare(&self, name: &str) -> Result<(), SingleError> {
        let mut scope = self.scopes.borrow_mut();

        if scope.top_contains(name) {
            return Err("Already a variable with this name in this scope.".into());
        }

        scope.declare(name.to_string());
        Ok(())
    }

    fn define(&self, name: &str) {
        self.scopes.borrow_mut().define(name.to_string())
    }

    fn resolve(&self, statements: &Vec<Stmt>) -> Result<(), SingleError> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }

        Ok(())
    }

    fn resolve_expression(&self, expression: &Expr) -> Result<(), SingleError> {
        walk_expr(self, expression)
    }

    fn resolve_function(
        &self,
        stmt: &FunctionStmt,
        function_type: FunctionType,
    ) -> Result<(), SingleError> {
        let enclosing_function = self.current_function.replace(function_type);
        self.begin_scope();

        for param in stmt.params.iter() {
            self.declare(&param.lexeme)?;
            self.define(&param.lexeme);
        }

        self.resolve(&stmt.body)?;
        self.end_scope();
        self.current_function.replace(enclosing_function);
        Ok(())
    }

    fn resolve_local(&self, expression: Expr, name: &str) -> Result<(), SingleError> {
        let scopes = self.scopes.borrow();

        for (i, scope) in scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.locals.borrow_mut().resolve(expression, i);
                break;
            }
        }

        Ok(())
    }

    fn resolve_statement(&self, statement: &Stmt) -> Result<(), SingleError> {
        walk_stmt(self, statement)
    }
}

impl stmt::Visitor<Result<(), SingleError>> for Resolver {
    fn visit_block(&self, stmt: &stmt::BlockStmt) -> Result<(), SingleError> {
        self.begin_scope();
        self.resolve(&stmt.statements)?;
        self.end_scope();

        Ok(())
    }

    fn visit_class(&self, stmt: &ClassStmt) -> Result<(), SingleError> {
        self.declare(&stmt.name.lexeme)?;
        self.define(&stmt.name.lexeme);

        let enclosing_class = self.current_class.replace(ClassType::Class);

        if let Some(superclass) = &stmt.superclass {
            if stmt.name.lexeme == superclass.name.lexeme {
                return Err("A class can't inherit from itself.".into());
            }

            self.current_class.replace(ClassType::Subclass);
            self.resolve_expression(&Expr::Variable(superclass.clone()))?;
            self.begin_scope();
            self.define("super");
        }

        self.begin_scope();
        self.force_define("this");

        for method in stmt.methods.iter() {
            let function_type = match method.name.lexeme.as_str() {
                "init" => FunctionType::Initializer,
                _ => FunctionType::Method,
            };
            self.resolve_function(method, function_type)?;
        }

        self.end_scope();
        if stmt.superclass.is_some() {
            self.end_scope()
        }

        self.current_class.replace(enclosing_class);

        Ok(())
    }

    fn visit_expression(&self, stmt: &stmt::ExpressionStmt) -> Result<(), SingleError> {
        self.resolve_expression(&stmt.expression)
    }

    fn visit_function(&self, stmt: &stmt::FunctionStmt) -> Result<(), SingleError> {
        self.declare(&stmt.name.lexeme)?;
        self.define(&stmt.name.lexeme);

        self.resolve_function(stmt, FunctionType::Function)
    }

    fn visit_if(&self, stmt: &stmt::IfStmt) -> Result<(), SingleError> {
        self.resolve_expression(&stmt.condition)?;
        self.resolve_statement(&stmt.then_branch)?;
        self.resolve_statement(&stmt.else_branch)?;

        Ok(())
    }

    fn visit_print(&self, stmt: &stmt::PrintStmt) -> Result<(), SingleError> {
        self.resolve_expression(&stmt.expression)
    }

    fn visit_return(&self, stmt: &stmt::ReturnStmt) -> Result<(), SingleError> {
        if let FunctionType::None = *self.current_function.borrow() {
            return Err("Cannot return from top-level code.".into());
        }

        if let FunctionType::Initializer = *self.current_function.borrow() {
            if !is_literal_nil(&stmt.value) {
                return Err("Cannot return a value from an initializer.".into());
            }
        }

        self.resolve_expression(&stmt.value)
    }

    fn visit_var(&self, stmt: &stmt::VarStmt) -> Result<(), SingleError> {
        self.declare(&stmt.name.lexeme)?;
        self.resolve_expression(&stmt.initializer)?;
        self.define(&stmt.name.lexeme);

        Ok(())
    }

    fn visit_while(&self, stmt: &stmt::WhileStmt) -> Result<(), SingleError> {
        self.resolve_expression(&stmt.condition)?;
        self.resolve_statement(&stmt.body)?;

        Ok(())
    }
}

fn is_literal_nil(expr: &Expr) -> bool {
    match expr {
        Expr::Literal(literal) => match literal.value {
            Literal::Nil => true,
            _ => false,
        },
        _ => false,
    }
}

impl expr::Visitor<Result<(), SingleError>> for Resolver {
    fn visit_assign(&self, expr: &AssignExpr) -> Result<(), SingleError> {
        self.resolve_expression(&expr.value)?;
        self.resolve_local(Expr::Assign(expr.clone()), &expr.name.lexeme)?;

        Ok(())
    }

    fn visit_binary(&self, expr: &BinaryExpr) -> Result<(), SingleError> {
        self.resolve_expression(&expr.left)?;
        self.resolve_expression(&expr.right)?;

        Ok(())
    }

    fn visit_call(&self, expr: &CallExpr) -> Result<(), SingleError> {
        self.resolve_expression(&expr.callee)?;

        for arg in expr.arguments.iter() {
            self.resolve_expression(arg)?;
        }

        Ok(())
    }

    fn visit_get(&self, expr: &GetExpr) -> Result<(), SingleError> {
        self.resolve_expression(&expr.object)
    }

    fn visit_grouping(&self, expr: &GroupingExpr) -> Result<(), SingleError> {
        self.resolve_expression(&expr.expression)
    }

    fn visit_literal(&self, _expr: &LiteralExpr) -> Result<(), SingleError> {
        Ok(())
    }

    fn visit_logical(&self, expr: &LogicalExpr) -> Result<(), SingleError> {
        self.resolve_expression(&expr.left)?;
        self.resolve_expression(&expr.right)?;

        Ok(())
    }

    fn visit_set(&self, expr: &SetExpr) -> Result<(), SingleError> {
        self.resolve_expression(&expr.value)?;
        self.resolve_expression(&expr.object)?;

        Ok(())
    }

    fn visit_super(&self, expr: &SuperExpr) -> Result<(), SingleError> {
        match *self.current_class.borrow() {
            ClassType::None => Err("Can't use 'super' outside of a class.".into()),
            ClassType::Class => Err("Can't use 'super' in a class with no superclass.".into()),
            _ => self.resolve_local(Expr::Super(expr.clone()), &expr.keyword.lexeme),
        }
    }

    fn visit_this(&self, expr: &ThisExpr) -> Result<(), SingleError> {
        if let ClassType::None = *self.current_class.borrow() {
            return Err("Can't use 'this' outside of a class.".into());
        }

        self.resolve_local(
            Expr::Variable(VariableExpr::new(expr.id, expr.keyword.clone())),
            &expr.keyword.lexeme,
        )
    }

    fn visit_unary(&self, expr: &UnaryExpr) -> Result<(), SingleError> {
        self.resolve_expression(&expr.right)
    }

    fn visit_variable(&self, expr: &VariableExpr) -> Result<(), SingleError> {
        let name = &expr.name.lexeme;
        match self.scopes.borrow().get(name) {
            Some(v) if v == false => {
                return Err("Can't read local variable in its own initializer.".into());
            }
            _ => (),
        }

        self.resolve_local(Expr::Variable(expr.clone()), &expr.name.lexeme)
    }
}
