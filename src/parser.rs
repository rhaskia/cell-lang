use crate::ast::Node;
use crate::lexer::{Token, TokenWrapper};
use fehler::throws;

type Error = String;

pub struct Parser {
    tokens: Vec<TokenWrapper>,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWrapper>) -> Self {
        Self { tokens, index: 0 }
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
            Lesser | LesserThan | Greater | GreaterThan | Or | And | NotEquals | Equals => 0,
            Plus | Minus => 1,
            Asterisk | Slash => 2,
            _ => 100,
        }
    }

    pub fn is_op(token: Token) -> bool {
        Self::op_order(token) < 50
    }

    #[throws]
    pub fn expr(&mut self) -> Node {
        let left = self.simple()?;
        if Self::is_op(self.peek()?.token) {
            let op = self.next()?.token;
            let right = self.expr()?;
            if let Node::Binary { left: r_left, op: r_op, right: r_right } = right {
                return Node::Binary { left, op, right }
            }
            return Node::Binary { left, op, right }
        }

        left
    }

    pub fn statement(&mut self) -> Result<Node, String> {
        match self.peek() {
            _ => self.expr(),
        }
    }

    #[throws]
    pub fn simple(&mut self) -> Node {
        let token = self.next()?;
        match &token.token {
            Token::Str(_) | Token::Float(_, _) | Token::Int(_) => {
                Node::Literal(token.token.clone())
            }
            Token::OpenBracket => {
                let mut items = Vec::new();
                if !self.peek()?.token.is_close_bracket() {
                    items = self.expr_list()?;
                }
                Node::Array(items)
            }
            Token::Identifier(ident) => Node::Variable(ident.clone()),
            _ => Err(Self::error(token, &format!("Expected expression but got {}", token.token)))?,
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

    pub fn peek(&mut self) -> Result<&TokenWrapper, String> {
        self.tokens.get(self.index + 1).ok_or(String::from("Token expected, EOF found"))
    }

    pub fn next(&mut self) -> Result<&TokenWrapper, String> {
        self.index += 1;
        self.tokens.get(self.index).ok_or(String::from("Token expected, EOF found"))
    }
}
