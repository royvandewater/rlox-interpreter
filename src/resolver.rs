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
    let scopes = Scopes(Vec::new());
    let locals = Locals::new();

    let resolver = Resolver::new();
    resolver.resolve((scopes, locals), statements)?;
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

    fn resolve(
        &self,
        mut bundle: (Scopes, Locals),
        statements: &Vec<Stmt>,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        for statement in statements {
            bundle = self.resolve_statement(bundle, statement)?;
        }

        Ok(bundle)
    }

    fn resolve_expression(
        &self,
        bundle: (Scopes, Locals),
        expression: &Expr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        walk_expr(self, bundle, expression)
    }

    fn resolve_function(
        &self,
        mut bundle: (Scopes, Locals),
        stmt: &FunctionStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        self.begin_scope();

        for param in stmt.params.iter() {
            self.declare(&param.lexeme);
            self.define(&param.lexeme);
        }

        bundle = self.resolve(bundle, &stmt.body)?;
        self.end_scope();
        Ok(bundle)
    }

    fn resolve_local(
        &self,
        bundle: (Scopes, Locals),
        expression: Expr,
        name: &str,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        let scopes = self.scopes.borrow();

        for (i, scope) in scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.locals.borrow_mut().resolve(expression, i);
                break;
            }
        }

        Ok(bundle)
    }

    fn resolve_statement(
        &self,
        bundle: (Scopes, Locals),
        statement: &Stmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        walk_stmt(self, bundle, statement)
    }
}

impl stmt::Visitor<(Scopes, Locals), Result<(Scopes, Locals), Vec<String>>> for Resolver {
    fn visit_block(
        &self,
        mut bundle: (Scopes, Locals),
        stmt: &stmt::BlockStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        self.begin_scope();
        bundle = self.resolve(bundle, &stmt.statements)?;
        self.end_scope();

        Ok(bundle)
    }

    fn visit_class(
        &self,
        mut bundle: (Scopes, Locals),
        stmt: &ClassStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        self.declare(&stmt.name.lexeme);
        self.define(&stmt.name.lexeme);

        self.begin_scope();
        self.force_define("this");

        for method in stmt.methods.iter() {
            bundle = self.resolve_function(bundle, method)?;
        }

        self.end_scope();

        Ok(bundle)
    }

    fn visit_expression(
        &self,
        bundle: (Scopes, Locals),
        stmt: &stmt::ExpressionStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        self.resolve_expression(bundle, &stmt.expression)
    }

    fn visit_function(
        &self,
        bundle: (Scopes, Locals),
        stmt: &stmt::FunctionStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        self.declare(&stmt.name.lexeme);
        self.define(&stmt.name.lexeme);

        self.resolve_function(bundle, stmt)
    }

    fn visit_if(
        &self,
        mut bundle: (Scopes, Locals),
        stmt: &stmt::IfStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        bundle = self.resolve_expression(bundle, &stmt.condition)?;
        bundle = self.resolve_statement(bundle, &stmt.then_branch)?;
        bundle = self.resolve_statement(bundle, &stmt.else_branch)?;

        Ok(bundle)
    }

    fn visit_print(
        &self,
        bundle: (Scopes, Locals),
        stmt: &stmt::PrintStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        self.resolve_expression(bundle, &stmt.expression)
    }

    fn visit_return(
        &self,
        bundle: (Scopes, Locals),
        stmt: &stmt::ReturnStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        self.resolve_expression(bundle, &stmt.value)
    }

    fn visit_var(
        &self,
        mut bundle: (Scopes, Locals),
        stmt: &stmt::VarStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        self.declare(&stmt.name.lexeme);
        bundle = self.resolve_expression(bundle, &stmt.initializer)?;
        self.define(&stmt.name.lexeme);

        Ok(bundle)
    }

    fn visit_while(
        &self,
        mut bundle: (Scopes, Locals),
        stmt: &stmt::WhileStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        bundle = self.resolve_expression(bundle, &stmt.condition)?;
        bundle = self.resolve_statement(bundle, &stmt.body)?;

        Ok(bundle)
    }
}

impl expr::Visitor<(Scopes, Locals), Result<(Scopes, Locals), Vec<String>>> for Resolver {
    fn visit_assign(
        &self,
        mut bundle: (Scopes, Locals),
        expr: &AssignExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        bundle = self.resolve_expression(bundle, &expr.value)?;
        bundle = self.resolve_local(bundle, Expr::Assign(expr.clone()), &expr.name.lexeme)?;

        Ok(bundle)
    }

    fn visit_binary(
        &self,
        mut bundle: (Scopes, Locals),
        expr: &BinaryExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        bundle = self.resolve_expression(bundle, &expr.left)?;
        bundle = self.resolve_expression(bundle, &expr.right)?;

        Ok(bundle)
    }

    fn visit_call(
        &self,
        mut bundle: (Scopes, Locals),
        expr: &CallExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        bundle = self.resolve_expression(bundle, &expr.callee)?;

        for arg in expr.arguments.iter() {
            bundle = self.resolve_expression(bundle, arg)?;
        }

        Ok(bundle)
    }

    fn visit_get(
        &self,
        bundle: (Scopes, Locals),
        expr: &GetExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        self.resolve_expression(bundle, &expr.object)
    }

    fn visit_grouping(
        &self,
        bundle: (Scopes, Locals),
        expr: &GroupingExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        self.resolve_expression(bundle, &expr.expression)
    }

    fn visit_literal(
        &self,
        bundle: (Scopes, Locals),
        _expr: &LiteralExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        Ok(bundle)
    }

    fn visit_logical(
        &self,
        bundle: (Scopes, Locals),
        expr: &LogicalExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        let bundle = self.resolve_expression(bundle, &expr.left)?;
        let bundle = self.resolve_expression(bundle, &expr.right)?;

        Ok(bundle)
    }

    fn visit_set(
        &self,
        bundle: (Scopes, Locals),
        expr: &SetExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        let bundle = self.resolve_expression(bundle, &expr.value)?;
        let bundle = self.resolve_expression(bundle, &expr.object)?;

        Ok(bundle)
    }

    fn visit_this(
        &self,
        bundle: (Scopes, Locals),
        expr: &ThisExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        self.resolve_local(
            bundle,
            Expr::Variable(VariableExpr::new(expr.keyword.clone())),
            &expr.keyword.lexeme,
        )
    }

    fn visit_unary(
        &self,
        bundle: (Scopes, Locals),
        expr: &UnaryExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        self.resolve_expression(bundle, &expr.right)
    }

    fn visit_variable(
        &self,
        bundle: (Scopes, Locals),
        expr: &VariableExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        let name = &expr.name.lexeme;
        match self.scopes.borrow().get(name) {
            Some(v) if v == false => {
                return Err(vec![format!(
                    "Can't read local variable in its own initializer."
                )]);
            }
            _ => (),
        }

        self.resolve_local(bundle, Expr::Variable(expr.clone()), &expr.name.lexeme)
    }
}
