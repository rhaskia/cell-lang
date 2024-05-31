use crate::ast::Node;
use crate::value::Value;
use crate::lexer::{Error, Token};
use crate::positioned::{Position, Positioned};
use fehler::throws;

type PNode = Positioned<Node>;

pub struct Parser {
    tokens: Vec<Positioned<Token>>,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Positioned<Token>>) -> Self {
        Self { tokens, index: 0 }
    }

    pub fn error<T>(token: &Positioned<Token>, msg: &str) -> Result<T, Error> {
        Err(Self::raw_error(token, msg))
    }

    pub fn raw_error(token: &Positioned<Token>, msg: &str) -> Error {
        Error { msg: msg.to_string(), start: token.start, end: token.end }
    }

    #[throws]
    pub fn parse(&mut self) -> Vec<PNode> {
        let mut ast = Vec::new();

        while !self.peek().is_eof() {
            ast.push(self.statement()?);
        }

        ast
    }

    pub fn op_order(token: &Token) -> usize {
        use Token::*;
        match token {
            Lesser | LesserThan | Greater | GreaterThan | Or | And | NotEquals | Equals | Mod => 0,
            Plus | Minus => 1,
            Asterisk | Slash => 2,
            _ => 100,
        }
    }

    pub fn is_op(token: &Token) -> bool {
        Self::op_order(token) < 50
    }

    #[throws]
    pub fn expr(&mut self) -> PNode {
        let left = self.call()?;

        if Self::is_op(&self.peek().inner) {
            let op = self.next().inner.clone();
            let right = self.expr()?;

            // ordering switching
            if let Node::Binary { left: ref r_left, op: ref r_op, right: ref r_right } = *right {
                if Self::op_order(&op) > Self::op_order(r_op) {
                    return PNode::new_binary(
                        PNode::new_binary(left, op, *r_left.clone()),
                        r_op.clone(),
                        *r_right.clone(),
                    );
                }
            }

            return PNode::new_binary(left, op, right);
        }

        left
    }

    #[throws]
    pub fn call(&mut self) -> PNode {
        let mut expr = self.simple()?;
        let start = expr.start;
        loop {
            match self.peek().inner {
                Token::OpenParen => {
                    self.next();
                    let args =
                        if self.peek().is_close_paren() { Vec::new() } else { self.expr_list()? };
                    let end = self.next_ensure(Token::CloseParen)?.end;
                    expr = Positioned {
                        inner: Node::Call { expr: Box::new(expr.clone()), args },
                        start,
                        end,
                    };
                }
                _ => break,
            };
        }
        expr
    }

    #[throws]
    fn func_statement(&mut self) -> PNode {
        let start = self.last().start;
        let name = self.next_ident()?;

        let mut params = Vec::new();
        if self.peek().is_open_paren() {
            self.next();
            params = self.identifier_list()?;
            self.next_ensure(Token::CloseParen)?;
        }

        self.next_ensure(Token::Define)?;
        let body = Box::new(self.expr()?);
        let end = body.end;

        Positioned { inner: Node::Function { name, params, body }, start, end }
    }

    #[throws]
    pub fn identifier_list(&mut self) -> Vec<String> {
        let mut idents = Vec::new();

        idents.push(self.next_ident()?);
        while self.peek().is_comma() {
            self.next();
            idents.push(self.next_ident()?);
        }

        idents
    }

    #[throws]
    pub fn statement(&mut self) -> PNode {
        let next = self.next().inner.clone();
        match &next {
            Token::Tilde => self.memory_statement()?,
            Token::Not => self.variable()?,
            Token::Pipeline => self.func_statement()?,
            Token::At => self.main_statement(false)?,
            Token::Sign => self.main_statement(true)?,
            _ => {
                self.backtrack();
                let expr = self.expr()?;
                self.next_ensure(Token::Semicolon)?;
                expr
            }
        }
    }

    #[throws]
    pub fn main_statement(&mut self, print: bool) -> PNode {
        let start = self.last().start;
        let centre = Box::new(self.expr()?);
        self.next_ensure(Token::Pipeline)?;
        let conditional = Box::new(self.expr()?);
        self.next_ensure(Token::Arrow)?;
        let result = Box::new(self.expr()?);
        let end = result.end;
        Positioned { inner: Node::Main { centre, conditional, result, print }, start, end }
    }

    #[throws]
    pub fn memory_statement(&mut self) -> PNode {
        let start = self.last().start;
        let mut data = vec![vec![]];
        let mut row = 0;
        while !self.peek().is_tilde() {
            let next = self.next();
            match next.inner {
                Token::Identifier(ident) => {
                    if self.peek().is_semicolon() {
                        self.next();
                        let count = self.next_number()?;
                        for _ in 0..count {
                            data[row].push(ident.clone());
                        }
                    }
                },
                Token::Pipe => { data.push(vec![]); row += 1; }
                Token::Semicolon => {
                    let repeat = self.next_number()?;
                    for _ in 1..repeat {
                        data.push(data[data.len() - 1].clone());
                        row += 1;
                    }
                }
                _ => {}
            }
        }
        let end = self.next_ensure(Token::Tilde)?.end;
        Positioned { inner: Node::Memory(data), start, end }
    }

    #[throws]
    pub fn variable(&mut self) -> PNode {
        let start = self.last().start;
        let name = self.next_ident()?;
        self.next_ensure(Token::Define)?;
        let value = Box::new(self.expr()?);
        let end = value.end;

        Positioned { inner: Node::Definition { name, value }, start, end }
    }

    #[throws]
    pub fn simple(&mut self) -> PNode {
        let token = self.next();
        let start = token.start;
        let end = token.end;
        match &token.inner {
            Token::Literal(literal) => {
                Positioned { inner: Node::Literal(literal.clone()), start, end }
            }
            Token::OpenBracket => {
                let mut items = Vec::new();
                if !self.peek().inner.is_close_bracket() {
                    items = self.expr_list()?;
                }
                let end = self.next_ensure(Token::CloseBracket)?.end;
                Positioned { inner: Node::Array(items), start, end }
            }
            Token::Identifier(ident) => {
                Positioned { inner: Node::Variable(ident.clone()), start, end }
            }
            Token::OpenParen => {
                let expr = self.expr()?;
                match self.peek().inner {
                    Token::CloseParen => {
                        self.next();
                        expr
                    }
                    Token::Comma => {
                        self.backtrack();
                        let list = self.expr_list()?;
                        let end = self.next_ensure(Token::CloseParen)?.end;
                        Positioned { inner: Node::Tuple(list), start, end }
                    }
                    _ => Self::error(&token, &format!("Expected , or ), found {}", token.inner))?,
                }
            }
            Token::At => {
                let start = self.last().start;
                let name = self.next_ident()?;
                let end = self.last().end;
                Positioned { inner: Node::Directional(name), start, end }
            }
            Token::Underscore => {
                let last = self.last();
                let end = last.end;
                let start = last.start;
                Positioned { inner: Node::Literal(Value::Unknown), start, end }
            }
            Token::Hash => {
                let start = self.last().end;
                self.next_ensure(Token::OpenParen)?;
                let expr = Box::new(self.expr()?);
                let end = self.next_ensure(Token::CloseParen)?.end;
                Positioned { inner: Node::Sum(expr), start, end }
            }
            _ => Self::error(&token, &format!("Expected expression but got {}", token.inner))?,
        }
    }

    #[throws]
    pub fn expr_list(&mut self) -> Vec<PNode> {
        let mut exprs = Vec::new();
        exprs.push(self.expr()?);
        while self.peek().inner == Token::Comma {
            self.next();
            exprs.push(self.expr()?);
        }
        exprs
    }

    pub fn peek(&mut self) -> Positioned<Token> {
        match self.tokens.get(self.index) {
            Some(token) => token.clone(),
            None => Positioned { inner: Token::EOF, start: Position::end(), end: Position::end()},
        }
    }

    pub fn next_ensure(&mut self, token: Token) -> Result<Positioned<Token>, Error> {
        let next = self.next();
        if next.inner != token {
            return Self::error(&next, &format!("Expected {token} found {}", next.inner));
        }
        Ok(next)
    }

    pub fn next_ident(&mut self) -> Result<String, Error> {
        let next = self.next();
        if let Token::Identifier(ident) = &next.inner {
            Ok(ident.clone())
        } else {
            Self::error(&next, &format!("Expected identifier found {:?}", next.inner))
        }
    }

    pub fn next_number(&mut self) -> Result<u8, Error> {
        let next = self.next();
        if let Token::Literal(Value::Int(ident)) = &next.inner {
            Ok(ident.clone())
        } else {
            Self::error(&next, &format!("Expected identifier found {:?}", next.inner))
        }
    }

    pub fn next(&mut self) -> Positioned<Token> {
        self.index += 1;
        match self.tokens.get(self.index - 1) {
            Some(token) => token.clone(),
            None => Positioned { inner: Token::EOF, start: Position::end(), end: Position::end()},
        }
    }

    pub fn last(&mut self) -> Positioned<Token> {
        self.tokens.get(self.index - 1).unwrap().clone()
    }

    pub fn backtrack(&mut self) {
        self.index -= 1;
    }
}
