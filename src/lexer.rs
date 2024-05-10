use core::str::Chars;
use fehler::throws;
use std::iter::Peekable;

type Error = String;

struct Lexer {
    line: usize,
    col: usize,
    current: usize,
    program: String,
}

impl Lexer {
    pub fn new() -> Self {
        Self { line: 0, col: 0, current: 0, program: String::new() }
    }

    #[throws]
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut program = self.program.chars().peekable();
        let mut tokens = Vec::new();

        while let Some(token) = program.next() {
            let t = match token {
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

                _ => {}
            };
        }

        tokens
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

enum Token {
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
}

enum Operator {}
