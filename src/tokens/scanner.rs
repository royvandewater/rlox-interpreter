use std::collections::HashMap;

use super::{Literal, Token, TokenType};

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> = {
        HashMap::from([
            ("and", TokenType::And),
            ("class", TokenType::Class),
            ("else", TokenType::Else),
            ("false", TokenType::False),
            ("for", TokenType::For),
            ("fun", TokenType::Fun),
            ("if", TokenType::If),
            ("nil", TokenType::Nil),
            ("or", TokenType::Or),
            ("print", TokenType::Print),
            ("return", TokenType::Return),
            ("super", TokenType::Super),
            ("this", TokenType::This),
            ("true", TokenType::True),
            ("var", TokenType::Var),
            ("while", TokenType::While),
        ])
    };
}

pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source: source.to_string(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub(crate) fn scan_tokens(&mut self) -> Result<Vec<Token>, Vec<String>> {
        let mut tokens = Vec::<Token>::new();
        let mut errors = Vec::<String>::new();

        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(None) => continue,
                Ok(token) => tokens.push(token.unwrap()),
                Err(error) => errors.push(error),
            }
        }

        tokens.push(self.new_token(TokenType::Eof, Literal::None));

        match errors.len() {
            0 => Ok(tokens),
            _ => Err(errors),
        }
    }

    fn scan_token(&mut self) -> Result<Option<Token>, String> {
        match self.advance() {
            '(' => Ok(Some(self.new_token(TokenType::LeftParen, Literal::None))),
            ')' => Ok(Some(self.new_token(TokenType::RightParen, Literal::None))),
            '{' => Ok(Some(self.new_token(TokenType::LeftBrace, Literal::None))),
            '}' => Ok(Some(self.new_token(TokenType::RightBrace, Literal::None))),
            ',' => Ok(Some(self.new_token(TokenType::Comma, Literal::None))),
            '.' => Ok(Some(self.new_token(TokenType::Dot, Literal::None))),
            '-' => Ok(Some(self.new_token(TokenType::Minus, Literal::None))),
            '+' => Ok(Some(self.new_token(TokenType::Plus, Literal::None))),
            ';' => Ok(Some(self.new_token(TokenType::Semicolon, Literal::None))),
            '*' => Ok(Some(self.new_token(TokenType::Star, Literal::None))),
            '!' => match self.peek() {
                '=' => {
                    self.advance();
                    Ok(Some(self.new_token(TokenType::BangEqual, Literal::None)))
                }
                _ => Ok(Some(self.new_token(TokenType::Bang, Literal::None))),
            },
            '=' => match self.peek() {
                '=' => {
                    self.advance();
                    Ok(Some(self.new_token(TokenType::EqualEqual, Literal::None)))
                }
                _ => Ok(Some(self.new_token(TokenType::Equal, Literal::None))),
            },
            '<' => match self.peek() {
                '=' => {
                    self.advance();
                    Ok(Some(self.new_token(TokenType::LessEqual, Literal::None)))
                }
                _ => Ok(Some(self.new_token(TokenType::Less, Literal::None))),
            },
            '>' => match self.peek() {
                '=' => {
                    self.advance();
                    Ok(Some(self.new_token(TokenType::GreaterEqual, Literal::None)))
                }
                _ => Ok(Some(self.new_token(TokenType::Greater, Literal::None))),
            },
            '/' => match self.peek() {
                '/' => {
                    self.advance();
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                    Ok(None)
                }
                _ => Ok(Some(self.new_token(TokenType::Slash, Literal::None))),
            },
            ' ' => Ok(None),
            '\r' => Ok(None),
            '\t' => Ok(None),
            '\n' => {
                self.line += 1;
                Ok(None)
            }
            '"' => self.parse_string(),
            c if self.is_digit(c) => self.parse_number(),
            c if self.is_alpha(c) => self.parse_identifier(),
            c => Err(format!("Unexpected charater: {}", c)),
        }
    }

    fn parse_string(&mut self) -> Result<Option<Token>, String> {
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            return Err("unterminated string".to_string());
        }

        // the closing "
        self.advance();

        // Trim the surrounding quotes
        let source = self.source.as_str();

        let value = &source[self.start + 1..self.current - 1];
        return Ok(Some(
            self.new_token(TokenType::String, Literal::String(value.to_string())),
        ));
    }

    fn parse_number(&mut self) -> Result<Option<Token>, String> {
        while !self.is_at_end() && self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            // consume the '.'
            self.advance();

            while !self.is_at_end() && self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let value: f64 = self.source[self.start..self.current]
            .parse()
            .map_err(|e| format!("Failed to parse number: {}", e))?;

        Ok(Some(
            self.new_token(TokenType::Number, Literal::Number(value)),
        ))
    }

    fn parse_identifier(&mut self) -> Result<Option<Token>, String> {
        while !self.is_at_end() && self.is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text: &str = &self.source[self.start..self.current];
        let token = match KEYWORDS.get(text) {
            Some(&token_type) => self.new_token(token_type, Literal::None),
            None => self.new_token(TokenType::Identifier, Literal::None),
        };

        Ok(Some(token))
    }

    fn advance(&mut self) -> char {
        let value = self.peek();
        self.current += 1;
        return value;
    }

    fn peek(&self) -> char {
        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn new_token(&self, token_type: TokenType, literal: Literal) -> Token {
        let text = &self.source[self.start..self.current];

        Token {
            token_type,
            lexeme: text.to_string(),
            literal,
            line_number: self.line,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn is_alpha(&self, c: char) -> bool {
        match c {
            'a'..='z' => true,
            'A'..='Z' => true,
            '_' => true,
            _ => false,
        }
    }

    fn is_digit(&self, c: char) -> bool {
        '0' <= c && c <= '9'
    }
}
