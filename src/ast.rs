use crate::lexer::Token;

#[derive(Debug)]
pub enum Node {
    Literal(Token),
    Array(Vec<Node>),
    Variable(String),
    Type,
    Binary { left: Box<Node>, op: Token, right: Box<Node>, }
}
