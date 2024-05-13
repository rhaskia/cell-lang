use crate::lexer::Token;

pub enum Node {
    Literal(Token),
    Array(Vec<Node>),
    Variable(String),
    Type,
}
