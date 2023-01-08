use std::{collections::HashMap, slice::Iter};

use crate::{
    expr::{self, *},
    stmt::{self, *},
};

#[derive(Debug)]
pub(crate) struct Scopes(Vec<HashMap<String, bool>>);

impl Scopes {
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

    Resolver::new()
        .resolve((scopes, locals), statements)
        .map(|(_, locals)| locals)
}

struct Resolver;

impl Resolver {
    fn new() -> Resolver {
        Resolver
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
        (mut scopes, mut locals): (Scopes, Locals),
        stmt: &FunctionStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        scopes.begin_scope();

        for param in stmt.params.iter() {
            scopes.declare(param.lexeme.clone());
            scopes.define(param.lexeme.clone());
        }

        (scopes, locals) = self.resolve((scopes, locals), &stmt.body)?;
        scopes.end_scope();
        Ok((scopes, locals))
    }

    fn resolve_local(
        &self,
        (scopes, mut locals): (Scopes, Locals),
        expression: Expr,
        name: &str,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        for (i, scope) in scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                locals.resolve(expression, i);
                break;
            }
        }

        Ok((scopes, locals))
    }

    fn resolve_statement(
        &self,
        args: (Scopes, Locals),
        statement: &Stmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        walk_stmt(self, args, statement)
    }
}

impl stmt::Visitor<(Scopes, Locals), Result<(Scopes, Locals), Vec<String>>> for Resolver {
    fn visit_block(
        &self,
        (mut scopes, mut locals): (Scopes, Locals),
        stmt: &stmt::BlockStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        scopes.begin_scope();
        (scopes, locals) = self.resolve((scopes, locals), &stmt.statements)?;
        scopes.end_scope();
        Ok((scopes, locals))
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
        (mut scopes, locals): (Scopes, Locals),
        stmt: &stmt::FunctionStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        scopes.declare(stmt.name.lexeme.clone());
        scopes.define(stmt.name.lexeme.clone());

        self.resolve_function((scopes, locals), stmt)
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
        (mut scopes, mut locals): (Scopes, Locals),
        stmt: &stmt::VarStmt,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        scopes.declare(stmt.name.lexeme.clone());
        (scopes, locals) = self.resolve_expression((scopes, locals), &stmt.initializer)?;
        scopes.define(stmt.name.lexeme.clone());
        Ok((scopes, locals))
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
        mut bundle: (Scopes, Locals),
        expr: &LogicalExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        bundle = self.resolve_expression(bundle, &expr.left)?;
        bundle = self.resolve_expression(bundle, &expr.right)?;

        Ok(bundle)
    }

    fn visit_unary(
        &self,
        mut bundle: (Scopes, Locals),
        expr: &UnaryExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        bundle = self.resolve_expression(bundle, &expr.right)?;

        Ok(bundle)
    }

    fn visit_variable(
        &self,
        (scopes, locals): (Scopes, Locals),
        expr: &VariableExpr,
    ) -> Result<(Scopes, Locals), Vec<String>> {
        println!("looking for {} in scopes: {:?}", &expr.name.lexeme, scopes);
        let name = &expr.name.lexeme;
        match scopes.get(name) {
            Some(v) if v == false => {
                return Err(vec![format!(
                    "Can't read local variable in its own initializer."
                )]);
            }
            _ => (),
        }

        self.resolve_local(
            (scopes, locals),
            Expr::Variable(expr.clone()),
            &expr.name.lexeme,
        )
    }
}
