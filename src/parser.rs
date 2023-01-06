use std::collections::VecDeque;

use crate::stmt::{
    BlockStmt, ExpressionStmt, FunctionStmt, IfStmt, PrintStmt, Stmt, Stmts, VarStmt, WhileStmt,
};
use crate::tokens::{Literal, Token, TokenType, Tokens};
use crate::{expr, expr::*, stmt};

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
            TokenType::Fun => {
                _ = self.advance();
                self.function("function")
            }
            TokenType::Var => {
                _ = self.advance();
                self.var_declaration()
            }
            _ => self.statement(),
        }
    }

    fn function(&mut self, kind: &str) -> Result<Stmt, Vec<String>> {
        let name = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?;

        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;

        let mut params: Vec<Token> = Vec::new();

        loop {
            if params.len() > 255 {
                return Err(vec![format!("Can't have more than 255 parameters.")]);
            }

            match self.peek_token_type() {
                TokenType::Comma => self.advance_and_discard()?,
                TokenType::Identifier => {
                    params.push(self.advance()?);
                }
                TokenType::RightParen => {
                    self.advance()?;
                    break;
                }
                _ => se("Expect parameter name, comma, or right paren.")?,
            }
        }

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body", kind),
        )?;

        let body = self.block()?;

        return Ok(Stmt::Function(FunctionStmt::new(name, params, body)));
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
                TokenType::For => {
                    self.advance()?;
                    self.for_statement()
                }
                TokenType::If => {
                    self.advance()?;
                    self.if_statement()
                }
                TokenType::Print => {
                    self.advance()?;
                    self.print_statement()
                }
                TokenType::While => {
                    self.advance()?;
                    self.while_statement()
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

    fn for_statement(&mut self) -> Result<Stmt, Vec<String>> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = match self.peek_token_type() {
            TokenType::Semicolon => {
                self.advance()?;
                stmt::noop()
            }
            TokenType::Var => {
                self.advance()?;
                self.var_declaration()?
            }
            _ => self.expression_statement()?,
        };

        let condition = match self.peek_token_type() {
            TokenType::Semicolon => expr::nil(),
            _ => self.expression()?,
        };
        self.consume(TokenType::Semicolon, "Expect ';' after 'for' condition.")?;

        let increment = match self.peek_token_type() {
            TokenType::RightParen => expr::nil(),
            _ => self.expression()?,
        };

        self.consume(TokenType::RightParen, "Expect ')' after 'for' clauses.")?;

        let original_body = self.statement()?;

        #[rustfmt::skip]
        Ok(Stmt::Block(BlockStmt::new(vec![
            initializer,
            Stmt::While(WhileStmt::new(
                condition,
                Stmt::Block(BlockStmt::new(vec![
                    original_body,
                    Stmt::Expression(ExpressionStmt::new(increment)),
                ])),
            )),
        ])))
    }

    fn while_statement(&mut self) -> Result<Stmt, Vec<String>> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after while condition.")?;

        let body = self.statement()?;

        Ok(Stmt::While(WhileStmt::new(condition, body)))
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
            _ => stmt::noop(),
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
        let expr = self.or()?;

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

    fn or(&mut self) -> Result<Expr, Vec<String>> {
        let mut expression = self.and()?;

        while self.check_one(TokenType::Or) {
            let operator = self.advance()?;
            let right = self.and()?;
            expression = Expr::Logical(LogicalExpr::new(expression, operator, right))
        }

        Ok(expression)
    }

    fn and(&mut self) -> Result<Expr, Vec<String>> {
        let mut expression = self.equality()?;

        while self.check_one(TokenType::And) {
            let operator = self.advance()?;
            let right = self.equality()?;
            expression = Expr::Logical(LogicalExpr::new(expression, operator, right))
        }

        Ok(expression)
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
            return self.call();
        }

        let operator = self.advance()?;
        let right = self.unary()?;

        Ok(Expr::Unary(UnaryExpr::new(operator, right)))
    }

    fn call(&mut self) -> Result<Expr, Vec<String>> {
        let mut expr = self.primary()?;

        while TokenType::LeftParen == self.peek_token_type() {
            self.advance()?;
            expr = self.finish_call(expr)?;
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, Vec<String>> {
        let mut arguments: Vec<Expr> = Vec::new();

        loop {
            if arguments.len() > 255 {
                return Err(vec![format!("Can't have more than 255 arguments")]);
            }

            match self.peek_token_type() {
                TokenType::Comma => self.advance_and_discard()?,
                TokenType::RightParen => break,
                _ => {
                    arguments.push(self.expression()?);
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call(CallExpr::new(callee, arguments)))
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

    fn check_one(&self, token_type: TokenType) -> bool {
        self.check(&[token_type])
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

    fn advance_and_discard(&mut self) -> Result<(), Vec<String>> {
        self.advance()?;
        Ok(())
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

fn se(s: &str) -> Result<(), Vec<String>> {
    Err(vec![s.to_string()])
}
