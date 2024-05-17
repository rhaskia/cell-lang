use crate::lexer::{Token, Keyword};

#[derive(Debug, Clone)]
pub enum Node {
    Literal(Token),
    Array(Vec<Node>),
    Variable(String),
    Type,
    Binary { left: Box<Node>, op: Token, right: Box<Node>, },
    Function { name: String, params: Vec<String>, body: Vec<Node> },
    Return(Box<Node>),
    Definition { name: String, var_type: Keyword, value: Box<Node> },
    ForLoop { item: String, iterator: Box<Node>, body: Vec<Node> },
    If { expr: Box<Node>, body: Vec<Node> },
}

impl Node {
    pub fn new_binary(left: Node, op: Token, right: Node) -> Self {
        Self::Binary { left: Box::new(left), op, right: Box::new(right) } 
    }
}
