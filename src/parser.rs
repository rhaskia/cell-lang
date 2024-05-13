use fehler::throws;
use crate::ast::Node;
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

    pub fn op_order(token: Token) -> usize {
        use Token::*;
        match token {
            Lesser | LesserThan | Greater | GreaterThan |
                Or | And | NotEquals | Equals => 0,
            Plus | Minus => 1,
            Asterisk | Slash => 2,
            _ => 100,
        }
    }    pub fn expr(&mut self) -> Result<Node, String> {
        let left = self.simple();

        left
    }

    pub fn statement(&mut self) -> Result<Node, String> {
        match self.peek() {
            _ => self.expr(),
        }
    }

    #[throws]
    pub fn simple(&mut self) -> Node {
        let token = self.next();
        match &token.token {
            Token::Str(_) | Token::Float(_, _) | Token::Int(_) => {
                Node::Literal(token.token.clone())
            }
            Token::OpenBracket => {
                let mut items = Vec::new();
                if !self.peek().token.is_close_bracket() {
                    items = self.expr_list()?;
                }
                Node::Array(items)
            }
            Token::Identifier(ident) => {
                Node::Variable(ident.clone())
            }
            _ => Err(Self::error(token, &format!("Expected expression but got {}", token.token)))?
        }
    }

    #[throws]
    pub fn expr_list(&mut self) -> Vec<Node> {
        let mut exprs = Vec::new();
        exprs.push(self.expr()?);
        while self.peek().token == Token::Comma {
            self.next();
            exprs.push(self.expr()?);
        } 
        exprs
    }

    pub fn peek(&mut self) -> &TokenWrapper {
        &self.tokens[self.index + 1]
    }

    pub fn next(&mut self) -> &TokenWrapper {
        self.index += 1;
        &self.tokens[self.index]
    }
}

