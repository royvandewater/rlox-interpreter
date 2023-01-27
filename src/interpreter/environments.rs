use crate::{expr::*, tokens::LoxInstance};
use std::cell::RefCell;

use crate::{environment::Environment, resolver::Locals, tokens::Literal};

pub(crate) struct Environments {
    locals: Locals,
    stack: RefCell<Vec<Environment>>,
}

impl Environments {
    pub fn new(globals: Environment, locals: Locals) -> Environments {
        Environments {
            locals,
            stack: RefCell::new(vec![globals]),
        }
    }

    pub(crate) fn assign(&self, name: &str, value: Literal) -> Result<(), String> {
        self.peek().assign(&name, value)
    }

    pub(crate) fn assign_expression(
        &self,
        expression: AssignExpr,
        name: &str,
        value: Literal,
    ) -> Result<(), String> {
        match self.locals.get(&Expr::Assign(expression)) {
            Some(distance) => self.assign_at_distance(distance, name, value),
            None => self.peek().assign_global(name, value),
        }
    }

    pub(crate) fn assign_at_distance(
        &self,
        distance: usize,
        name: &str,
        value: Literal,
    ) -> Result<(), String> {
        self.peek().assign_at_distance(distance, name, value)
    }

    pub fn push_scope(&self, scope: Environment) {
        self.stack.borrow_mut().push(scope)
    }

    pub fn pop_scope(&self) {
        self.stack.borrow_mut().pop();
    }

    pub(crate) fn get_at_distance(&self, distance: usize, name: &str) -> Option<Literal> {
        self.peek().get_at_distance(distance, name)
    }

    fn get_global(&self, name: &str) -> Option<Literal> {
        self.peek().get_global(name)
    }

    pub(crate) fn look_up_variable(
        &self,
        name: &str,
        expr: &VariableExpr,
    ) -> Result<Literal, String> {
        let value = match self.locals.get(&Expr::Variable(expr.clone())) {
            None => self.get_global(name),
            Some(distance) => self.get_at_distance(distance, name),
        };

        match value {
            None => panic!("variable with name '{}' not defined", &expr.name.lexeme),
            Some(literal) => Ok(literal),
        }
    }

    pub(crate) fn look_up_super_and_object(
        &self,
        expr: &SuperExpr,
    ) -> Result<(LoxInstance, LoxInstance), String> {
        let distance = self.locals.get(&Expr::Super(expr.clone())).unwrap();
        let superclass = self.get_at_distance(distance, "super").unwrap();
        let object = self.get_at_distance(distance - 1, "this").unwrap();

        match (superclass, object) {
            (Literal::ClassInstance(s), Literal::ClassInstance(o)) => Ok((s, o)),
            (Literal::ClassInstance(_), _) => {
                Err("Could not resolve 'this' when looking up superclass".into())
            }
            _ => Err("Could not resolve 'super' when looking up superclass".into()),
        }
    }

    pub(crate) fn peek(&self) -> Environment {
        self.stack.borrow().last().unwrap().clone()
    }
}
