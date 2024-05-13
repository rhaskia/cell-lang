use crate::ast::Node;
use crate::lexer::{Token, TokenWrapper, Keyword};
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

    pub fn error(token: &TokenWrapper, msg: &str) -> Result<Node, Error> {
        Err(format!("Syntax Error on {}:{}: {}", token.line, token.col, msg))
    }

    pub fn parse(&mut self) -> Vec<Node> {
        let ast = Vec::new();

        ast
    }

    pub fn op_order(token: &Token) -> usize {
        use Token::*;
        match token {
            Lesser | LesserThan | Greater | GreaterThan | Or | And | NotEquals | Equals => 0,
            Plus | Minus => 1,
            Asterisk | Slash => 2,
            _ => 100,
        }
    }

    pub fn is_op(token: &Token) -> bool {
        Self::op_order(token) < 50
    }

    #[throws]
    pub fn expr(&mut self) -> Node {
        let left = self.simple()?;

        if Self::is_op(&self.peek()?.token) {
            let op = self.next()?.token.clone();
            let right = self.expr()?;

            // ordering switching
            if let Node::Binary { left: ref r_left, op: ref r_op, right: ref r_right } = right {
                if Self::op_order(&op) > Self::op_order(r_op) {
                    return Node::new_binary( 
                        Node::new_binary(left, op, *r_left.clone()),
                        r_op.clone(),
                        *r_right.clone()
                    );
                }
            }

            return Node::new_binary(left, op, right);
        }

        left
    }

    #[throws]
    fn func_statement(&mut self) -> Node {
        let next = self.next()?;
        let name = if let Token::Identifier(ident) = next.token {
            ident
        } else { 
            self::error(&next, &format!("Expected identifier got {}", next.token))?
        };


    }

    #[throws]
    pub fn statement(&mut self) -> Node {
        match self.next()?.token {
            Token::Keyword(Keyword::Fn) => self.func_statement(),
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
            Token::OpenParen => self.expr()?,
            _ => Self::error(token, &format!("Expected expression but got {}", token.token))?,
        }
    }

    #[throws]
    pub fn expr_list(&mut self) -> Vec<Node> {
        let mut exprs = Vec::new();
        exprs.push(self.expr()?);
        while self.peek()?.token == Token::Comma {
            self.next()?;
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
