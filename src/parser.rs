use std::collections::VecDeque;

use crate::expr::*;
use crate::stmt::{BlockStmt, ExpressionStmt, IfStmt, PrintStmt, Stmt, Stmts, VarStmt};
use crate::tokens::{Literal, Token, TokenType, Tokens};

pub(super) struct Parser(VecDeque<Token>);

impl Parser {
    pub(crate) fn parse(&mut self) -> Result<Stmts, Vec<String>> {
        let mut statements: Vec<Stmt> = Vec::new();

        while self.peek().is_some() {
            statements.push(self.declaration()?);
        }

        Ok(statements.into())
    }

    fn declaration(&mut self) -> Result<Stmt, Vec<String>> {
        let next_token = self.peek().unwrap();
        match next_token.token_type {
            TokenType::Var => {
                _ = self.advance();
                self.var_declaration()
            }
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, Vec<String>> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let initializer = match self.check(&[TokenType::Equal]) {
            true => {
                _ = self.advance();
                Some(self.expression()?)
            }
            false => None,
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        )?;
        Ok(Stmt::Var(VarStmt::new(name, initializer)))
    }

    fn statement(&mut self) -> Result<Stmt, Vec<String>> {
        match self.peek() {
            Some(token) => match token.token_type {
                TokenType::If => {
                    self.advance()?;
                    self.if_statement()
                }
                TokenType::Print => {
                    self.advance()?;
                    self.print_statement()
                }
                TokenType::LeftBrace => {
                    self.advance()?;
                    Ok(Stmt::Block(BlockStmt::new(self.block()?)))
                }
                _ => self.expression_statement(),
            },
            None => self.expression_statement(),
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, Vec<String>> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;

        let else_branch = match self.peek_token_type() {
            TokenType::Else => {
                self.advance()?;
                self.statement()?
            }
            _ => Stmt::Block(BlockStmt::new(Vec::new())), // noop
        };

        Ok(Stmt::If(IfStmt::new(condition, then_branch, else_branch)))
    }

    fn print_statement(&mut self) -> Result<Stmt, Vec<String>> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(PrintStmt::new(value)))
    }

    fn expression_statement(&mut self) -> Result<Stmt, Vec<String>> {
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(ExpressionStmt::new(expression)))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Vec<String>> {
        let mut statements: Vec<Stmt> = Vec::new();

        while self.peek().is_some() && !self.check(&[TokenType::RightBrace]) {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        return Ok(statements);
    }

    fn expression(&mut self) -> Result<Expr, Vec<String>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Vec<String>> {
        let expr = self.equality()?;

        if self.check(&[TokenType::Equal]) {
            _ = self.advance()?;
            let value = self.assignment()?;

            return match expr {
                Expr::Variable(v) => {
                    let name = v.name;
                    Ok(Expr::Assign(AssignExpr::new(name, value)))
                }
                _ => Err(vec![format!("Invalid assignment target.")]),
            };
        }

        Ok(expr)
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
            TokenType::Number => Expr::Literal(LiteralExpr::new(next_token.literal)),
            TokenType::String => Expr::Literal(LiteralExpr::new(next_token.literal)),
            TokenType::Identifier => Expr::Variable(VariableExpr::new(next_token)),
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

    fn peek_token_type(&self) -> TokenType {
        match self.peek() {
            Some(t) => t.token_type,
            None => TokenType::None,
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

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, Vec<String>> {
        match self.check(&[token_type]) {
            true => self.advance(),
            false => Err(Vec::from([format!(
                "Could not consume: {}. \"{}\"",
                self.peek().unwrap_or(&Token::new(
                    TokenType::None,
                    "<nothing>".to_string(),
                    Literal::Nil,
                    0
                )),
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
