use std::collections::VecDeque;

use super::*;
use crate::tokens::{TokenType, Tokens};

pub(super) struct Parser(VecDeque<Token>);

impl Parser {
    pub(crate) fn parse(&mut self) -> Result<Expr, Vec<String>> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, Vec<String>> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, Vec<String>> {
        let mut expression = self.comparison()?;

        while self.check(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.advance()?;
            let right = self.comparison()?;

            expression = Expr::Binary(BinaryExpr::new(expression, operator, right));
        }

        Ok(expression)
    }

    fn comparison(&mut self) -> Result<Expr, Vec<String>> {
        let mut expression = self.term()?;

        while self.check(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.advance()?;
            let right = self.term()?;

            expression = Expr::Binary(BinaryExpr::new(expression, operator, right));
        }

        Ok(expression)
    }

    fn term(&mut self) -> Result<Expr, Vec<String>> {
        let mut expression = self.factor()?;

        while self.check(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.advance()?;
            let right = self.factor()?;

            expression = Expr::Binary(BinaryExpr::new(expression, operator, right));
        }

        Ok(expression)
    }

    fn factor(&mut self) -> Result<Expr, Vec<String>> {
        let mut expression = self.unary()?;

        while self.check(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.advance()?;
            let right = self.unary()?;

            expression = Expr::Binary(BinaryExpr::new(expression, operator, right));
        }

        Ok(expression)
    }

    fn unary(&mut self) -> Result<Expr, Vec<String>> {
        if !self.check(&[TokenType::Bang, TokenType::Minus]) {
            return self.primary();
        }

        let operator = self.advance()?;
        let right = self.unary()?;

        Ok(Expr::Unary(UnaryExpr::new(operator, right)))
    }

    fn primary(&mut self) -> Result<Expr, Vec<String>> {
        let next_token = self.advance()?;

        let expression = match next_token.token_type {
            TokenType::False => Expr::Literal(LiteralExpr::new(Literal::Boolean(false))),
            TokenType::True => Expr::Literal(LiteralExpr::new(Literal::Boolean(true))),
            TokenType::Nil => Expr::Literal(LiteralExpr::new(Literal::Nil)),
            TokenType::Number => Expr::Literal(LiteralExpr::new(next_token.literal.clone())),
            TokenType::String => Expr::Literal(LiteralExpr::new(next_token.literal.clone())),
            TokenType::LeftParen => {
                let inner_expression = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression")?;
                Expr::Grouping(GroupingExpr::new(inner_expression))
            }
            _ => Err(Vec::from([format!(
                "Unrecognized primary token: {}",
                next_token
            )]))?,
        };

        return Ok(expression);
    }

    fn check(&self, token_types: &[TokenType]) -> bool {
        match self.peek() {
            None => false,
            Some(token) => token_types.iter().any(|&t| t == token.token_type),
        }
    }

    fn peek(&self) -> Option<&Token> {
        match self.0.front() {
            None => None,
            Some(eof) if TokenType::Eof == eof.token_type => None,
            Some(token) => Some(token),
        }
    }

    fn advance(&mut self) -> Result<Token, Vec<String>> {
        match self.0.pop_front() {
            None => Err(Vec::from([
                "Tried to pop_front on empty dequeue".to_string()
            ])),
            Some(eof) if TokenType::Eof == eof.token_type => Err(Vec::from([
                "Tried to pop_front with only EOF left".to_string(),
            ])),
            Some(token) => Ok(token),
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<(), Vec<String>> {
        match self.check(&[token_type]) {
            true => {
                self.advance()?;
                Ok(())
            }
            false => Err(Vec::from([format!(
                "Could not consume: {}. \"{}\"",
                self.peek().unwrap(),
                message
            )])),
        }
    }
}

impl From<Tokens> for Parser {
    fn from(tokens: Tokens) -> Self {
        Parser(tokens.iter().cloned().collect())
    }
}
