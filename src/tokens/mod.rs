mod scanner;

use std::{fmt::Display, slice::Iter, str::FromStr};

use self::scanner::Scanner;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Nil,
    Number(f64),
    String(String),
    Boolean(bool),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Nil => f.write_str("nil"),
            Literal::Number(n) => f.write_fmt(format_args!("{}", n)),
            Literal::String(s) => f.write_str(s.as_str()),
            Literal::Boolean(b) => f.write_fmt(format_args!("{}", b)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
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

pub struct Tokens(Vec<Token>);

impl Tokens {
    pub fn iter(&self) -> Iter<Token> {
        self.0.iter()
    }
}

impl FromStr for Tokens {
    type Err = Vec<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = Scanner::new(s).scan_tokens()?;

        Ok(Tokens(tokens))
    }
}
