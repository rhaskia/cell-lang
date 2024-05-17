use crate::ast::Node;
use crate::lexer::{Token, TokenWrapper, Keyword, Error, Position};
use fehler::throws;

pub struct Parser {
    tokens: Vec<TokenWrapper>,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWrapper>) -> Self {
        Self { tokens, index: 0 }
    }

    pub fn error<T>(token: &TokenWrapper, msg: &str) -> Result<T, Error> {
        Err(Error { msg: msg.to_string(), start: token.start, end: token.end })
    }

    pub fn eof_error(msg: &str) -> Error {
        Error { msg: msg.to_string(), start: Position::new(), end: Position::new() }
    }

    #[throws]
    pub fn parse(&mut self) -> Vec<Node> {
        let mut ast = Vec::new();

        while self.peek().is_ok() {
            ast.push(self.statement()?);
            println!("{ast:?}");
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
            println!(" next is ::: {:?}", self.peek());
        }
        self.next_ensure(Token::CloseBrace)?; 

        Node::Function { name, params, body }
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
    pub fn statement(&mut self) -> Node {
        let next = self.next()?.token.clone();
        match &next {
            Token::Keyword(Keyword::Fn) => self.func_statement()?,
            Token::Keyword(Keyword::Return) => self.return_statement()?,
            Token::Keyword(Keyword::For) => self.for_statement()?,
            Token::Keyword(Keyword::If) => self.if_statement()?,
            Token::Keyword(keyword) => self.variable(keyword.clone())?,
            _ => self.expr()?,
        }
    }

    #[throws]
    pub fn return_statement(&mut self) -> Node {
        let expr = self.expr()?;
        self.next_ensure(Token::Semicolon)?;
        Node::Return(Box::new(expr))
    } 

    #[throws]
    pub fn if_statement(&mut self) -> Node {
        let expr = Box::new(self.expr()?);
        self.next_ensure(Token::OpenBrace)?;
        let mut body = Vec::new();
        while !self.peek()?.is_close_brace() {
            body.push(self.statement()?);
        }
        self.next_ensure(Token::CloseBrace)?;

        Node::If { expr, body }
    }

    #[throws]
    pub fn for_statement(&mut self) -> Node {
        let item = self.next_ident()?;
        self.next_ensure(Token::Keyword(Keyword::In))?;
        let iterator = Box::new(self.expr()?);

        self.next_ensure(Token::OpenBrace)?; 
        let mut body = Vec::new();
        while !self.peek()?.is_close_brace() {
            body.push(self.statement()?);
        }
        self.next_ensure(Token::CloseBrace)?; 

        Node::ForLoop { item, iterator, body }
    }

    #[throws]
    pub fn variable(&mut self, var_type: Keyword) -> Node {
        let name = self.next_ident()?;
        self.next_ensure(Token::Define)?;
        let value = Box::new(self.expr()?);
        self.next_ensure(Token::Semicolon)?;

        Node::Definition { name, var_type, value }
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

    pub fn peek(&mut self) -> Result<&TokenWrapper, Error> {
        self.tokens.get(self.index).ok_or(Self::eof_error("Token expected, EOF found"))
    }

    pub fn next_ensure(&mut self, token: Token) -> Result<&TokenWrapper, Error> {
        let next = self.next()?;
        if next.token != token {
            return Self::error(next, &format!("Expected {token:?} found {:?}", next.token));
        }
        Ok(next)
    }

    pub fn next_ident(&mut self) -> Result<String, Error> {
        let next = self.next()?;
        if let Token::Identifier(ident) = &next.token {
            Ok(ident.clone())
        } else {
            Self::error(next, &format!("Exptected identifier found {:?}", next.token))
        }
    }

    pub fn next(&mut self) -> Result<&TokenWrapper, Error> {
        self.index += 1;
        self.tokens.get(self.index - 1).ok_or(Self::eof_error("Token expected, EOF found"))
    }
}
