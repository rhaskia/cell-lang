use crate::positioned::Positioned;
use crate::{
    lexer::{Token},
    value::Value,
};

type PNode = Positioned<Node>;
type PToken = Positioned<Token>;

#[derive(Debug, Clone)]
pub enum Node {
    Literal(Value),
    Array(Vec<PNode>),
    Variable(String),
    Binary { left: Box<PNode>, op: Token, right: Box<PNode> },
    Call { expr: Box<PNode>, args: Vec<PNode> },
    Tuple(Vec<PNode>),
    Function { name: String, params: Vec<String>, body: Box<PNode> },
    Memory(Vec<Vec<String>>),
    Definition { name: String, value: Box<PNode> },
    Directional(String),
    Sum(Box<PNode>),
    Main { centre: Box<PNode>, conditional: Box<PNode>, result: Box<PNode>},
}

impl PNode {
    pub fn new_binary(left: PNode, op: Token, right: PNode) -> Self {
        let start = left.start.min(right.start);
        let end = left.start.max(right.end);
        Positioned {
            inner: Node::Binary { left: Box::new(left), op, right: Box::new(right) },
            start,
            end,
        }
    }
}
