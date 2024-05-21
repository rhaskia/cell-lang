use crate::positioned::Position;
use crate::positioned::Positioned;
use crate::value::Value;
use fehler::throws;
use macros::{match_tokens, match_two};
use std::ops::{Deref, DerefMut};
use std::{f32, str::FromStr};
use strum_macros::{EnumIs, EnumString};

#[derive(Debug)]
pub struct Error {
    pub msg: String,
    pub start: Position,
    pub end: Position,
}

pub struct Lexer {
    current: Position,
    token_start: Position,
    program: Vec<char>,
    index: usize,
}

impl Lexer {
    pub fn new(program: String) -> Self {
        let program: Vec<char> = program.chars().collect();
        Self { current: Position::new(), token_start: Position::new(), index: 0, program }
    }

    #[throws]
    pub fn scan_tokens(&mut self) -> Vec<Positioned<Token>> {
        let mut tokens = Vec::new();

        while let Ok(token) = self.next() {
            self.token_start = self.current;
            match token {
                ',' => tokens.push(self.wrap(Token::Comma)),
                ':' => tokens.push(self.wrap(Token::Colon)),
                ';' => tokens.push(self.wrap(Token::Semicolon)),
                '.' => tokens.push(self.wrap(Token::Period)),

                '-' => tokens.push(self.wrap(Token::Minus)),
                '+' => tokens.push(self.wrap(Token::Plus)),
                '*' => tokens.push(self.wrap(Token::Asterisk)),
                '^' => tokens.push(self.wrap(Token::Carat)),
                '~' => tokens.push(self.wrap(Token::Tilde)),
                '!' => tokens.push(self.wrap(Token::Tilde)),

                '{' => tokens.push(self.wrap(Token::OpenBrace)),
                '}' => tokens.push(self.wrap(Token::CloseBrace)),
                '[' => tokens.push(self.wrap(Token::OpenBracket)),
                ']' => tokens.push(self.wrap(Token::CloseBracket)),
                '(' => tokens.push(self.wrap(Token::OpenParen)),
                ')' => tokens.push(self.wrap(Token::CloseParen)),

                '&' => match_two!(self, tokens, '&', And),
                '|' => match_tokens!(self, tokens, Pipe, '>' => Pipeline, '|' => Or),
                '>' => match_tokens!(self, tokens, Greater, '=' => GreaterThan),
                '<' => match_tokens!(self, tokens, Lesser, '=' => LesserThan),
                '=' => match_tokens!(self, tokens, Define, '=' => Equals, '>' => Arrow),

                '"' => {
                    let mut string = String::new();
                    while self.peek().unwrap() != '"' {
                        string.push(self.next().unwrap());
                    }
                    let _ = self.next();
                    tokens.push(self.wrap(Token::Literal(Value::String(string))));
                }

                '\'' => {
                    let char = self.next().unwrap();
                    if let '\'' = self.next()? {
                        tokens.push(self.wrap(Token::Literal(Value::Char(char))));
                    } else {
                        self.error("Char closing tag not found.")?;
                    }
                }

                '/' => {
                    if self.matches('/') {
                        while self.peek_not('\n') {
                            let _ = self.next();
                        }
                    } else {
                        tokens.push(self.wrap(Token::Slash));
                    }
                }

                '0'..='9' => {
                    let mut number = token.to_string();
                    while let Ok(c) = self.peek() {
                        if !(c.is_numeric() || c == '.' || c == '_') {
                            break;
                        }
                        number.push(self.next().unwrap());
                    }
                    tokens.push(self.wrap(self.number(number)?));
                }

                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = token.to_string();
                    while let Ok(peeked) = self.peek() {
                        if !is_alphanumeric(peeked) {
                            break;
                        }
                        ident.push(self.next().unwrap());
                    }

                    match Keyword::from_str(&ident) {
                        Ok(keyword) => tokens.push(self.wrap(Token::Keyword(keyword))),
                        Err(_) => tokens.push(self.wrap(Token::Identifier(ident))),
                    }
                }

                _ => {}
            };
        }

        tokens
    }

    pub fn wrap(&self, inner: Token) -> Positioned<Token> {
        Positioned { inner, start: self.token_start, end: self.current }
    }

    pub fn peek(&mut self) -> Result<char, Error> {
        let peeked = self.program.get(self.index);
        match peeked {
            Some(p) => Ok(*p),
            None => self.error("EOF found unexpectedly"),
        }
    }

    pub fn peek_not(&mut self, c: char) -> bool {
        match self.peek() {
            Ok(peeked) if peeked != c => true,
            _ => false,
        }
    }

    pub fn next(&mut self) -> Result<char, Error> {
        self.current.col += 1;
        self.index += 1;
        let next_item_op = self.program.get(self.index - 1).copied();
        let next_item = match next_item_op {
            Some(c) => c,
            None => self.error("EOF found unexpectedly")?,
        };
        if next_item == '\n' {
            self.next_line();
        }
        Ok(next_item)
    }

    pub fn next_line(&mut self) {
        self.current.line += 1;
        self.current.col = 1;
    }

    pub fn matches(&mut self, item: char) -> bool {
        if self.program.get(self.index) == Some(&&item) {
            self.next();
            return true;
        }
        return false;
    }

    #[throws]
    pub fn number(&self, string: String) -> Token {
        let cleaned = string.replace("_", "");
        let is_frac = string.contains('.');
        if is_frac {
            let parts = cleaned.split(".").collect::<Vec<&str>>();
            if parts.len() > 2 {
                self.error("More than one decimal point found in number")?;
            }
            let value = str::parse::<f32>(&cleaned).unwrap();
            Token::Literal(Value::Float(value))
        } else {
            let value = str::parse::<i32>(&cleaned).unwrap();
            Token::Literal(Value::Int(value))
        }
    }

    pub fn error<T>(&self, msg: &str) -> Result<T, Error> {
        Err(Error { msg: msg.to_string(), start: self.token_start, end: self.current })
    }
}

fn is_alphanumeric(c: char) -> bool {
    if c == '_' {
        return true;
    }
    c.is_alphanumeric()
}

#[derive(Debug, EnumIs, Clone, strum_macros::Display, PartialEq)]
pub enum Token {
    Comma,
    Colon,
    Semicolon,
    Period,
    Tilde,

    Minus,
    Plus,
    Asterisk,
    Slash,
    Carat,

    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    OpenParen,
    CloseParen,

    Define,
    Arrow,

    Or,
    And,
    Not,
    Equals,
    NotEquals,
    Greater,
    Lesser,
    GreaterThan,
    LesserThan,

    Pipe, 
    Pipeline,

    Literal(Value),

    Keyword(Keyword),
    Identifier(String),
}

#[derive(Debug, PartialEq, EnumString, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum Keyword {
    Fn,
    Rule,
    Sys,
    Float,
    Int,
    String,
    Char,
    Return,
    For,
    In,
    If,
    Array,
}

pub mod macros {
    macro_rules! match_tokens {
        ($s:ident, $tokens:expr, $base_token:ident, $($extra_char:literal => $extra_token:ident),*) => {
            {
                let mut base = true;
                $(
                    if $s.matches($extra_char) {
                        $tokens.push($s.wrap(Token::$extra_token));
                        base = false;
                    }
                )*
                if base {
                    $tokens.push($s.wrap(Token::$base_token));
                }
            }
        }
    }

    macro_rules! match_two {
        ($s:ident, $tokens:expr, $add_char:expr, $token:ident) => {{
            if $s.matches($add_char) {
                $tokens.push($s.wrap(Token::$token))
            }
        }};
    }

    pub(crate) use match_tokens;
    pub(crate) use match_two;
}
