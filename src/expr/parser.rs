use super::*;
use crate::tokens::{TokenType, Tokens};

pub(super) struct Parser {
    current: usize,
    tokens: Vec<Token>,
}

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
            let operator = self.advance().clone();
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
            let operator = self.advance().clone();
            let right = self.term()?;

            expression = Expr::Binary(BinaryExpr::new(expression, operator, right));
        }

        Ok(expression)
    }

    fn term(&mut self) -> Result<Expr, Vec<String>> {
        let mut expression = self.factor()?;

        while self.check(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.advance().clone();
            let right = self.factor()?;

            expression = Expr::Binary(BinaryExpr::new(expression, operator, right));
        }

        Ok(expression)
    }

    fn factor(&mut self) -> Result<Expr, Vec<String>> {
        let mut expression = self.unary()?;

        while self.check(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.advance().clone();
            let right = self.unary()?;

            expression = Expr::Binary(BinaryExpr::new(expression, operator, right));
        }

        Ok(expression)
    }

    fn unary(&mut self) -> Result<Expr, Vec<String>> {
        if !self.check(&[TokenType::Bang, TokenType::Minus]) {
            return self.primary();
        }

        let operator = self.advance().clone();
        let right = self.unary()?;

        Ok(Expr::Unary(UnaryExpr::new(operator, right)))
    }

    fn primary(&mut self) -> Result<Expr, Vec<String>> {
        let next_token = self.advance();

        let expression = match next_token.token_type {
            TokenType::False => Expr::Literal(LiteralExpr::new(Literal::Bolean(false))),
            TokenType::True => Expr::Literal(LiteralExpr::new(Literal::Bolean(true))),
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
        if self.is_at_end() {
            return false;
        };

        let token = self.peek();

        token_types.iter().any(|&t| t == token.token_type)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn advance(&mut self) -> &Token {
        let val = self.tokens.get(self.current).unwrap();

        self.current += 1;

        return val;
    }

    fn is_at_end(&self) -> bool {
        match self.peek().token_type {
            TokenType::Eof => true,
            _ => false,
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<(), Vec<String>> {
        match self.check(&[token_type]) {
            true => {
                self.advance();
                Ok(())
            }
            false => Err(Vec::from([format!(
                "Could not consume: {}. \"{}\"",
                self.peek(),
                message
            )])),
        }
    }
}

impl From<Tokens> for Parser {
    fn from(tokens: Tokens) -> Self {
        Parser {
            current: 0,
            tokens: tokens.iter().cloned().collect(),
        }
    }
}
