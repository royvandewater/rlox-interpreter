mod scanner;

use std::{fmt::Display, slice::Iter, str::FromStr};

use self::scanner::Scanner;

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Debug)]
pub enum Literal {
    None,
    Number(f64),
    String(String),
}

#[derive(Clone, Debug)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Literal,
    line_number: usize,
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
