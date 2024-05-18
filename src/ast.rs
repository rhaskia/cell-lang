use crate::positioned::Positioned;
use crate::{
    lexer::{Keyword, Token},
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
    Function { name: String, params: Vec<String>, body: Vec<PNode> },
    Return(Box<PNode>),
    Definition { name: String, var_type: Keyword, value: Box<PNode> },
    ForLoop { item: String, iterator: Box<PNode>, body: Vec<PNode> },
    If { expr: Box<PNode>, body: Vec<PNode> },
    Call { expr: Box<PNode>, args: Vec<PNode> },
    Get { expr: Box<PNode>, arg: Box<PNode> },
    Tuple(Vec<PNode>),
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
