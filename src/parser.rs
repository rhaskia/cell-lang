use fehler::throws;

use crate::lexer::{TokenWrapper, Token};

type Error = String;

struct Parser {
    tokens: Vec<TokenWrapper>,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWrapper>) {
        
    }

    pub fn error(token: &TokenWrapper, msg: &str) -> Error {
        format!("Syntax Error on {}:{}: {}", token.line, token.col, msg)
    }

    pub fn parse(&mut self) -> Vec<Node> {
        let ast = Vec::new();


        ast
    }

    pub fn expr(&mut self) -> Result<(), String> {
        let left = self.simple();

        left
    }

    pub fn statement(&mut self) -> Result<(), String> {
        match self.peek() {
            _ => self.expr(),
        }
    }

    #[throws]
    pub fn simple(&mut self) {
        let token = self.next();
        match token.token {
            Token::String(_) | Token::Float(_, _) | Token::Int(_) => {
                Node::Literal(token.token.clone())
            }
            _ => Err(Self::error(token, "Expected expression but got "))?
        }
    }

    pub fn peek(&mut self) -> &TokenWrapper {
        &self.tokens[self.index + 1]
    }

    pub fn next(&mut self) -> &TokenWrapper {
        self.index += 1;
        &self.tokens[self.index]
    }
}

enum Node {
    Literal(Token)
}
