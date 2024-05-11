use core::str::Chars;
use fehler::throws;
use std::iter::Peekable;
use std::str::FromStr;
use strum_macros::EnumString;

type Error = String;

pub struct Lexer {
    line: usize,
    col: usize,
    current: usize,
    program: String,
}

impl Lexer {
    pub fn new(program: String) -> Self {
        Self { line: 0, col: 0, current: 0, program }
    }

    #[throws]
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut program = self.program.chars().peekable();
        let mut tokens = Vec::new();

        while let Some(token) = program.next() {
            match token {
                ',' => tokens.push(Token::Comma),
                ':' => tokens.push(Token::Colon),
                ';' => tokens.push(Token::Semicolon),

                '-' => tokens.push(Token::Minus),
                '+' => tokens.push(Token::Plus),
                '*' => tokens.push(Token::Asterisk),
                '/' => tokens.push(Token::Slash),
                '^' => tokens.push(Token::Carat),

                '{' => tokens.push(Token::OpenBrace),
                '}' => tokens.push(Token::CloseBrace),
                '[' => tokens.push(Token::OpenBracket),
                ']' => tokens.push(Token::CloseBracket),
                '(' => tokens.push(Token::OpenParen),
                ')' => tokens.push(Token::CloseParen),

                '"' => {
                    let mut string = String::new();
                    while program.peek().ok_or(self.error("EOF found while reading String"))?
                        != &'"'
                    {
                        string.push(program.next().unwrap());
                    }
                    program.next();
                    tokens.push(Token::String(string));
                }

                '\'' => {
                    let char = program.next().ok_or(self.error("Expected char, found EOF"))?;
                    if let Some('\'') = program.next() {
                        tokens.push(Token::Char(char));
                    } else {
                        Err(self.error("Char closing tag not found."))?;
                    }
                }

                '|' => {
                    if program.matches('|') {
                        tokens.push(Token::Or)
                    }
                }

                '>' => {
                    if program.matches('=') {
                        tokens.push(Token::GreaterThan);
                    } else {
                        tokens.push(Token::Greater);
                    }
                }

                '<' => {
                    if program.matches('=') {
                        tokens.push(Token::LesserThan);
                    } else {
                        tokens.push(Token::Lesser);
                    }
                }

                '=' => {
                    if program.matches('=') {
                        tokens.push(Token::Equals);
                    } else {
                        tokens.push(Token::Define);
                    }
                }

                '0'..='9' => {
                    let mut number = token.to_string();
                    while let Some(c) = program.peek() {
                        if !(c.is_numeric() || *c == '.' || *c == '_') {
                            break;
                        }
                        number.push(program.next().unwrap());
                    }
                    tokens.push(self.number(number)?);
                }

                'a'..='z' | 'A'..='Z' => {
                    let mut ident = token.to_string();
                    while program
                        .peek()
                        .ok_or(self.error("EOF found while reading String"))?
                        .is_alphabetic()
                    {
                        ident.push(program.next().unwrap());
                    }

                    match Keyword::from_str(&ident) {
                        Ok(keyword) => tokens.push(Token::Keyword(keyword)),
                        Err(_) => tokens.push(Token::Identifier(ident))
                    } 
                }

                _ => {}
            };
        }

        tokens
    }

    #[throws]
    pub fn number(&self, string: String) -> Token {
        let cleaned = string.replace("_", "");
        let is_frac = string.contains('.');
        if is_frac {
            let parts = cleaned.split(".").collect::<Vec<&str>>();
            if parts.len() > 2 {
                Err(self.error("More than one decimal point found in number"))?;
            }
            return Token::Float(parts[1].to_string(), parts[0].to_string());
        } else {
            return Token::Int(cleaned);
        }
    }

    pub fn error(&self, msg: &str) -> String {
        format!("Error on {}:{}: {}", self.line, self.col, msg)
    }
}

pub trait PeekMatch {
    fn matches(&mut self, item: char) -> bool;
}

impl PeekMatch for Peekable<Chars<'_>> {
    fn matches(&mut self, item: char) -> bool {
        if self.peek() == Some(&&item) {
            self.next();
            return true;
        }
        return false;
    }
}

#[derive(Debug)]
pub enum Token {
    Comma,
    Colon,
    Semicolon,

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

    Or,
    And,
    Equals,
    Greater,
    Lesser,
    GreaterThan,
    LesserThan,

    String(String),
    Char(char),
    Float(String, String),
    Int(String),

    Keyword(Keyword),
    Identifier(String),
}

#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Keyword {
    Rule,
    Any,
    Sys,
    Float,
    Int,
    String,
    Char,
}
