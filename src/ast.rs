use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Node {
    Literal(Token),
    Array(Vec<Node>),
    Variable(String),
    Type,
    Binary { left: Box<Node>, op: Token, right: Box<Node>, }
}

impl Node {
    pub fn new_binary(left: Node, op: Token, right: Node) -> Self {
        Self::Binary { left: Box::new(left), op, right: Box::new(right) } 
    }
}
