mod lox_callable;
mod lox_instance;
mod scanner;

use std::{collections::VecDeque, fmt::Display, str::FromStr};

use self::scanner::Scanner;
pub(crate) use lox_callable::*;
pub(crate) use lox_instance::*;
use rust_decimal::Decimal;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
    None,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Literal {
    Nil,
    Boolean(bool),
    Callable(LoxCallable),
    ClassInstance(LoxInstance),
    Number(Decimal),
    String(String),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Nil => f.write_str("nil"),
            Literal::Boolean(b) => f.write_fmt(format_args!("{}", b)),
            Literal::Callable(c) => f.write_fmt(format_args!("{}", c)),
            Literal::ClassInstance(c) => f.write_fmt(format_args!("{}", c)),
            Literal::Number(n) => f.write_fmt(format_args!("{}", n)),
            Literal::String(s) => f.write_str(s.as_str()),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line_number: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Literal,
        line_number: usize,
    ) -> Token {
        Token {
            token_type,
            lexeme,
            literal,
            line_number,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token{{token_type: {:?}, lexeme: {}, literal: {:?}, line_number: {}}}",
            self.token_type, self.lexeme, self.literal, self.line_number
        )
    }
}

pub(crate) struct Tokens(Vec<Token>);

impl FromStr for Tokens {
    type Err = Vec<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = Scanner::new(s).scan_tokens()?;

        Ok(Tokens(tokens))
    }
}

impl Into<VecDeque<Token>> for Tokens {
    fn into(self) -> VecDeque<Token> {
        self.0.into()
    }
}
