use crate::ast::Node;
use crate::value::Value;
use crate::lexer::{Error, Keyword, Token};
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
        Err(Error { msg: msg.to_string(), start: token.start, end: token.end })
    }

    pub fn eof_error(msg: &str) -> Error {
        Error { msg: msg.to_string(), start: Position::new(), end: Position::new() }
    }

    #[throws]
    pub fn parse(&mut self) -> Vec<PNode> {
        let mut ast = Vec::new();

        while self.peek().is_ok() {
            ast.push(self.statement()?);
        }

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
    pub fn expr(&mut self) -> PNode {
        let left = self.call()?;

        if Self::is_op(&self.peek()?.inner) {
            let op = self.next()?.inner.clone();
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
            match self.peek()?.inner {
                Token::OpenParen => {
                    self.next()?;
                    let args =
                        if self.peek()?.is_close_paren() { Vec::new() } else { self.expr_list()? };
                    let end = self.next_ensure(Token::CloseParen)?.end;
                    expr = Positioned {
                        inner: Node::Call { expr: Box::new(expr.clone()), args },
                        start,
                        end,
                    };
                }
                Token::OpenBracket => {
                    self.next()?;
                    let arg = Box::new(self.expr()?);
                    let end = self.next_ensure(Token::CloseBracket)?.end;
                    expr = Positioned {
                        inner: Node::Get { expr: Box::new(expr.clone()), arg },
                        start,
                        end,
                    };
                }
                Token::Period => {}
                _ => break,
            };
        }
        expr
    }

    #[throws]
    fn func_statement(&mut self) -> PNode {
        let start = self.last()?.start;
        let name = self.next_ident()?;

        let mut params = Vec::new();
        if self.peek()?.is_open_paren() {
            self.next()?;
            params = self.identifier_list()?;
            self.next_ensure(Token::CloseParen)?;
        }

        self.next_ensure(Token::OpenBrace)?;
        let mut body = Vec::new();
        while !self.peek()?.is_close_brace() {
            body.push(self.statement()?);
        }
        let end = self.next_ensure(Token::CloseBrace)?.end;

        Positioned { inner: Node::Function { name, params, body }, start, end }
    }

    #[throws]
    pub fn identifier_list(&mut self) -> Vec<String> {
        let mut idents = Vec::new();

        idents.push(self.next_ident()?);
        while self.peek()?.is_comma() {
            self.next()?;
            idents.push(self.next_ident()?);
        }

        idents
    }

    #[throws]
    pub fn statement(&mut self) -> PNode {
        let next = self.next()?.inner.clone();
        match &next {
            Token::Keyword(Keyword::Fn) => self.func_statement()?,
            Token::Keyword(Keyword::Return) => self.return_statement()?,
            Token::Keyword(Keyword::For) => self.for_statement()?,
            Token::Keyword(Keyword::If) => self.if_statement()?,
            Token::Tilde => self.memory_statement()?,
            Token::Not => self.variable()?,
            _ => {
                self.backtrack();
                let expr = self.expr()?;
                self.next_ensure(Token::Semicolon)?;
                expr
            }
        }
    }


    #[throws]
    pub fn memory_statement(&mut self) -> PNode {
        let start = self.last()?.start;
        let mut data = vec![vec![]];
        let mut row = 0;
        while !self.peek()?.is_tilde() {
            let next = self.next()?;
            match next.inner {
                Token::Identifier(ident) => {
                    if self.peek()?.is_semicolon() {
                        self.next()?;
                        let count = self.next_number()?;
                        for _ in 0..count {
                            data[row].push(ident.clone());
                        }
                    }
                },
                Token::Pipe => { row += 1; }
                _ => {}
            }
        }
        let end = self.next_ensure(Token::Tilde)?.end;
        Positioned { inner: Node::Memory(data), start, end }
    }

    #[throws]
    pub fn return_statement(&mut self) -> PNode {
        let start = self.last()?.start;
        let expr = self.expr()?;
        let end = self.next_ensure(Token::Semicolon)?.end;
        Positioned { inner: Node::Return(Box::new(expr)), start, end }
    }

    #[throws]
    pub fn if_statement(&mut self) -> PNode {
        let start = self.last()?.start;
        let expr = Box::new(self.expr()?);
        self.next_ensure(Token::OpenBrace)?;
        let mut body = Vec::new();
        while !self.peek()?.is_close_brace() {
            body.push(self.statement()?);
        }
        let end = self.next_ensure(Token::CloseBrace)?.end;

        Positioned { inner: Node::If { expr, body }, start, end }
    }

    #[throws]
    pub fn for_statement(&mut self) -> PNode {
        let start = self.last()?.start;
        let item = self.next_ident()?;
        self.next_ensure(Token::Keyword(Keyword::In))?;
        let iterator = Box::new(self.expr()?);

        self.next_ensure(Token::OpenBrace)?;
        let mut body = Vec::new();
        while !self.peek()?.is_close_brace() {
            body.push(self.statement()?);
        }
        let end = self.next_ensure(Token::CloseBrace)?.end;

        Positioned { inner: Node::ForLoop { item, iterator, body }, start, end }
    }

    #[throws]
    pub fn variable(&mut self) -> PNode {
        let start = self.last()?.start;
        let name = self.next_ident()?;
        let value = Box::new(self.expr()?);
        let end = value.end;

        Positioned { inner: Node::Definition { name, value }, start, end }
    }

    #[throws]
    pub fn simple(&mut self) -> PNode {
        let token = self.next()?;
        let start = token.start;
        let end = token.end;
        match &token.inner {
            Token::Literal(literal) => {
                Positioned { inner: Node::Literal(literal.clone()), start, end }
            }
            Token::OpenBracket => {
                let mut items = Vec::new();
                if !self.peek()?.inner.is_close_bracket() {
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
                match self.peek()?.inner {
                    Token::CloseParen => {
                        self.next()?;
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
            _ => Self::error(&token, &format!("Expected expression but got {}", token.inner))?,
        }
    }

    #[throws]
    pub fn expr_list(&mut self) -> Vec<PNode> {
        let mut exprs = Vec::new();
        exprs.push(self.expr()?);
        while self.peek()?.inner == Token::Comma {
            self.next()?;
            exprs.push(self.expr()?);
        }
        exprs
    }

    pub fn peek(&mut self) -> Result<&Positioned<Token>, Error> {
        self.tokens.get(self.index).ok_or(Self::eof_error("Token expected, EOF found"))
    }

    pub fn next_ensure(&mut self, token: Token) -> Result<Positioned<Token>, Error> {
        let next = self.next()?;
        if next.inner != token {
            return Self::error(&next, &format!("Expected {token} found {}", next.inner));
        }
        Ok(next)
    }

    pub fn next_ident(&mut self) -> Result<String, Error> {
        let next = self.next()?;
        if let Token::Identifier(ident) = &next.inner {
            Ok(ident.clone())
        } else {
            Self::error(&next, &format!("Exptected identifier found {:?}", next.inner))
        }
    }

    pub fn next_number(&mut self) -> Result<i32, Error> {
        let next = self.next()?;
        if let Token::Literal(Value::Int(ident)) = &next.inner {
            Ok(ident.clone())
        } else {
            Self::error(&next, &format!("Exptected identifier found {:?}", next.inner))
        }
    }

    pub fn next(&mut self) -> Result<Positioned<Token>, Error> {
        self.index += 1;
        self.tokens.get(self.index - 1).cloned().ok_or(Self::eof_error("Token expected, EOF found"))
    }

    pub fn last(&mut self) -> Result<Positioned<Token>, Error> {
        self.tokens.get(self.index - 1).cloned().ok_or(Self::eof_error("Token expected, EOF found"))
    }

    pub fn backtrack(&mut self) {
        self.index -= 1;
    }
}
