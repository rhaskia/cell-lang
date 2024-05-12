use fehler::throws;
use std::str::FromStr;
use strum_macros::{EnumIs, EnumString};

type Error = String;

pub struct Lexer {
    line: usize,
    col: usize,
    program: Vec<char>,
    index: usize,
}

impl Lexer {
    pub fn new(program: String) -> Self {
        let program: Vec<char> = program.chars().collect();
        Self { line: 1, col: 1, index: 0, program }
    }

    #[throws]
    pub fn scan_tokens(&mut self) -> Vec<TokenWrapper> {
        let mut tokens = Vec::new();

        while let Ok(token) = self.next() {
            match token {
                ',' => tokens.push(self.wrap(Token::Comma)),
                ':' => tokens.push(self.wrap(Token::Colon)),
                ';' => tokens.push(self.wrap(Token::Semicolon)),

                '-' => tokens.push(self.wrap(Token::Minus)),
                '+' => tokens.push(self.wrap(Token::Plus)),
                '*' => tokens.push(self.wrap(Token::Asterisk)),
                '^' => tokens.push(self.wrap(Token::Carat)),

                '{' => tokens.push(self.wrap(Token::OpenBrace)),
                '}' => tokens.push(self.wrap(Token::CloseBrace)),
                '[' => tokens.push(self.wrap(Token::OpenBracket)),
                ']' => tokens.push(self.wrap(Token::CloseBracket)),
                '(' => tokens.push(self.wrap(Token::OpenParen)),
                ')' => tokens.push(self.wrap(Token::CloseParen)),

                '"' => {
                    let mut string = String::new();
                    while self.peek().unwrap() != '"' {
                        string.push(self.next().unwrap());
                    }
                    self.next();
                    tokens.push(self.wrap(Token::String(string)));
                }

                '\'' => {
                    let char = self.next().unwrap();
                    if let '\'' = self.next()? {
                        tokens.push(self.wrap(Token::Char(char)));
                    } else {
                        Err(self.error("Char closing tag not found."))?;
                    }
                }

                '|' => {
                    if self.matches('|') {
                        tokens.push(self.wrap(Token::Or))
                    }
                }

                '/' => {
                    if self.matches('/') {
                        while self.peek_not('\n') {
                            self.next();
                        }
                    } else {
                        tokens.push(self.wrap(Token::Slash));
                    }
                }

                '>' => {
                    if self.matches('=') {
                        tokens.push(self.wrap(Token::GreaterThan));
                    } else {
                        tokens.push(self.wrap(Token::Greater));
                    }
                }

                '<' => {
                    if self.matches('=') {
                        tokens.push(self.wrap(Token::LesserThan));
                    } else {
                        tokens.push(self.wrap(Token::Lesser));
                    }
                }

                '=' => {
                    if self.matches('=') {
                        tokens.push(self.wrap(Token::Equals));
                    } else {
                        tokens.push(self.wrap(Token::Define));
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
                        if !is_alphanumeric(peeked) { break; }
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

    pub fn wrap(&self, token: Token) -> TokenWrapper {
        TokenWrapper { token, line: self.line, col: self.col }
    }

    pub fn peek(&mut self) -> Result<char, String> {
        let peeked = self.program.get(self.index + 1);
        peeked.ok_or(self.error("EOF found unexpectedly")).copied()
    }

    pub fn peek_not(&mut self, c: char) -> bool {
        match self.peek() {
            Ok(peeked) if peeked != c => true,
            _ => false,
        }
    }

    pub fn next(&mut self) -> Result<char, String> {
        self.col += 1;
        self.index += 1;
        let next_item_op = self.program.get(self.index).copied();
        let next_item = match next_item_op {
            Some(c) => c,
            None => return Err(self.error("EOF found unexpectedly")),
        };
        if next_item == '\n' {
            self.next_line();
        }
        Ok(next_item)
    }

    pub fn next_line(&mut self) {
        self.line += 1;
        self.col = 1;
    }

    pub fn matches(&mut self, item: char) -> bool {
        if self.program.get(self.index + 1) == Some(&&item) {
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
                Err(self.error("More than one decimal point found in number"))?;
            }
            Token::Float(parts[1].to_string(), parts[0].to_string())
        } else {
            Token::Int(cleaned)
        }
    }

    pub fn error(&self, msg: &str) -> String {
        let program = self.program.iter().collect::<String>();
        let mut lines = program.lines();
        let err_line = lines.nth(self.line - 1).unwrap_or("Out of bounds");

        format!("Error on {}:{}: {}\non line: {}", self.line, self.col, msg, err_line)
    }
}

fn is_alphanumeric(c: char) -> bool {
    if c == '_' {
        return true;
    }
    c.is_alphanumeric()
}

#[derive(Debug, EnumIs, Clone)]
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

#[derive(Debug, PartialEq, EnumString, Clone)]
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

#[derive(Debug)]
pub struct TokenWrapper {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}
