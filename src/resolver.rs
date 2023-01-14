use std::{cell::RefCell, collections::HashMap, slice::Iter};

use crate::{
    expr::{self, *},
    stmt::{self, *},
};

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

#[derive(Clone)]
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
    resolver.resolve(statements)?;
    Ok(resolver.locals.into_inner())
}

struct Resolver {
    locals: RefCell<Locals>,
    scopes: RefCell<Scopes>,
}

impl Resolver {
    fn new() -> Resolver {
        Resolver {
            locals: RefCell::new(Locals::new()),
            scopes: RefCell::new(Scopes::new()),
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

    fn declare(&self, name: &str) {
        self.scopes.borrow_mut().declare(name.to_string())
    }

    fn define(&self, name: &str) {
        self.scopes.borrow_mut().define(name.to_string())
    }

    fn resolve(&self, statements: &Vec<Stmt>) -> Result<(), Vec<String>> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }

        Ok(())
    }

    fn resolve_expression(&self, expression: &Expr) -> Result<(), Vec<String>> {
        walk_expr(self, (), expression)
    }

    fn resolve_function(&self, stmt: &FunctionStmt) -> Result<(), Vec<String>> {
        self.begin_scope();

        for param in stmt.params.iter() {
            self.declare(&param.lexeme);
            self.define(&param.lexeme);
        }

        self.resolve(&stmt.body)?;
        self.end_scope();
        Ok(())
    }

    fn resolve_local(&self, expression: Expr, name: &str) -> Result<(), Vec<String>> {
        let scopes = self.scopes.borrow();

        for (i, scope) in scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.locals.borrow_mut().resolve(expression, i);
                break;
            }
        }

        Ok(())
    }

    fn resolve_statement(&self, statement: &Stmt) -> Result<(), Vec<String>> {
        walk_stmt(self, (), statement)
    }
}

impl stmt::Visitor<(), Result<(), Vec<String>>> for Resolver {
    fn visit_block(&self, _: (), stmt: &stmt::BlockStmt) -> Result<(), Vec<String>> {
        self.begin_scope();
        self.resolve(&stmt.statements)?;
        self.end_scope();

        Ok(())
    }

    fn visit_class(&self, _: (), stmt: &ClassStmt) -> Result<(), Vec<String>> {
        self.declare(&stmt.name.lexeme);
        self.define(&stmt.name.lexeme);

        self.begin_scope();
        self.force_define("this");

        for method in stmt.methods.iter() {
            self.resolve_function(method)?;
        }

        self.end_scope();

        Ok(())
    }

    fn visit_expression(&self, _: (), stmt: &stmt::ExpressionStmt) -> Result<(), Vec<String>> {
        self.resolve_expression(&stmt.expression)
    }

    fn visit_function(&self, _: (), stmt: &stmt::FunctionStmt) -> Result<(), Vec<String>> {
        self.declare(&stmt.name.lexeme);
        self.define(&stmt.name.lexeme);

        self.resolve_function(stmt)
    }

    fn visit_if(&self, _: (), stmt: &stmt::IfStmt) -> Result<(), Vec<String>> {
        self.resolve_expression(&stmt.condition)?;
        self.resolve_statement(&stmt.then_branch)?;
        self.resolve_statement(&stmt.else_branch)?;

        Ok(())
    }

    fn visit_print(&self, _: (), stmt: &stmt::PrintStmt) -> Result<(), Vec<String>> {
        self.resolve_expression(&stmt.expression)
    }

    fn visit_return(&self, _: (), stmt: &stmt::ReturnStmt) -> Result<(), Vec<String>> {
        self.resolve_expression(&stmt.value)
    }

    fn visit_var(&self, _: (), stmt: &stmt::VarStmt) -> Result<(), Vec<String>> {
        self.declare(&stmt.name.lexeme);
        self.resolve_expression(&stmt.initializer)?;
        self.define(&stmt.name.lexeme);

        Ok(())
    }

    fn visit_while(&self, _: (), stmt: &stmt::WhileStmt) -> Result<(), Vec<String>> {
        self.resolve_expression(&stmt.condition)?;
        self.resolve_statement(&stmt.body)?;

        Ok(())
    }
}

impl expr::Visitor<(), Result<(), Vec<String>>> for Resolver {
    fn visit_assign(&self, _: (), expr: &AssignExpr) -> Result<(), Vec<String>> {
        self.resolve_expression(&expr.value)?;
        self.resolve_local(Expr::Assign(expr.clone()), &expr.name.lexeme)?;

        Ok(())
    }

    fn visit_binary(&self, _: (), expr: &BinaryExpr) -> Result<(), Vec<String>> {
        self.resolve_expression(&expr.left)?;
        self.resolve_expression(&expr.right)?;

        Ok(())
    }

    fn visit_call(&self, _: (), expr: &CallExpr) -> Result<(), Vec<String>> {
        self.resolve_expression(&expr.callee)?;

        for arg in expr.arguments.iter() {
            self.resolve_expression(arg)?;
        }

        Ok(())
    }

    fn visit_get(&self, _: (), expr: &GetExpr) -> Result<(), Vec<String>> {
        self.resolve_expression(&expr.object)
    }

    fn visit_grouping(&self, _: (), expr: &GroupingExpr) -> Result<(), Vec<String>> {
        self.resolve_expression(&expr.expression)
    }

    fn visit_literal(&self, _: (), _expr: &LiteralExpr) -> Result<(), Vec<String>> {
        Ok(())
    }

    fn visit_logical(&self, _: (), expr: &LogicalExpr) -> Result<(), Vec<String>> {
        self.resolve_expression(&expr.left)?;
        self.resolve_expression(&expr.right)?;

        Ok(())
    }

    fn visit_set(&self, _: (), expr: &SetExpr) -> Result<(), Vec<String>> {
        self.resolve_expression(&expr.value)?;
        self.resolve_expression(&expr.object)?;

        Ok(())
    }

    fn visit_this(&self, _: (), expr: &ThisExpr) -> Result<(), Vec<String>> {
        self.resolve_local(
            Expr::Variable(VariableExpr::new(expr.keyword.clone())),
            &expr.keyword.lexeme,
        )
    }

    fn visit_unary(&self, _: (), expr: &UnaryExpr) -> Result<(), Vec<String>> {
        self.resolve_expression(&expr.right)
    }

    fn visit_variable(&self, _: (), expr: &VariableExpr) -> Result<(), Vec<String>> {
        let name = &expr.name.lexeme;
        match self.scopes.borrow().get(name) {
            Some(v) if v == false => {
                return Err(vec![format!(
                    "Can't read local variable in its own initializer."
                )]);
            }
            _ => (),
        }

        self.resolve_local(Expr::Variable(expr.clone()), &expr.name.lexeme)
    }
}
