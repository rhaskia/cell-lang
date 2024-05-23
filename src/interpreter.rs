use std::collections::HashMap;

use crate::lexer::Error;
use crate::positioned::Position;
use crate::positioned::Positioned;
use crate::value::Value;
use crate::{
    ast::Node,
    lexer::{Keyword, Token},
};
use fehler::throws;

type PNode = Positioned<Node>;

pub struct Interpreter {
    instructions: Vec<Positioned<Node>>,
    constants: HashMap<String, Value>,
    functions: HashMap<String, (Vec<String>, Box<PNode>)>,
    memory: Vec<Vec<Value>>,
    index: usize,
}

impl Interpreter {
    pub fn new(instructions: Vec<Positioned<Node>>) -> Self {
        Self { 
            instructions, 
            index: 0, 
            constants: HashMap::new(), 
            functions: HashMap::new(),
            memory: Vec::new(),
        }
    }

    #[throws]
    pub fn interpret(&mut self) {
        self.load_instructions()?;
        println!("\x1b[2J"); // Clear
        loop {

        }
    }

    #[throws]
    pub fn load_instructions(&mut self) {
        for node in &self.instructions {
            match node.inner.clone() {
                Node::Definition { value, name } => {
                    let value = self.evaluate(&value)?;
                    self.constants.insert(name.to_string(), value);
                }
                Node::Function { name, params, body } => {
                    self.functions.insert(name.to_string(), (params, body)); 
                }
                Node::Main { centre, conditional, result, print } => {}
                Node::Memory(memory) => {
                    self.memory = memory.into_iter().map(|r| r.into_iter().map(|item| self.constants[&item].clone()).collect()).collect();
                }
                _ => {
                    println!("{:?}", node);
                    self.evaluate(&node)?;
                }
            };
        }
    }

    pub fn next(&mut self) -> Positioned<Node> {
        self.index += 1;
        self.instructions[self.index - 1].clone()
    }

    #[throws]
    pub fn evaluate(&self, value: &Positioned<Node>) -> Value {
        match &value.inner {
            Node::Variable(v) => self.constants[v].clone(),
            Node::Array(a) => {
                let evaled: Result<Vec<Value>, Error> =
                    a.iter().map(|item| self.evaluate(item)).collect();
                Value::Array(evaled?)
            }
            Node::Binary { left, op, right } => self.evaluate_binary(&left, &op, &right)?,
            Node::Literal(v) => v.clone(),
            _ => Self::error("Expected expression found statement")?,
        }
    }

    #[throws]
    pub fn evaluate_binary(
        &self,
        left: &Box<Positioned<Node>>,
        op: &Token,
        right: &Box<Positioned<Node>>,
    ) -> Value {
        let start = left.start.min(right.start);
        let end = left.end.max(right.end);

        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        let result = match op {
            Token::Minus => left.sub(&right),
            Token::Plus => left.add(&right),
            Token::Asterisk => left.mul(&right),
            Token::Slash => left.div(&right),
            Token::Or => left.or(&right),
            Token::And => left.and(&right),
            _ => Some(Value::Bool(match op {
                Token::Equals => left == right,
                Token::NotEquals => left != right,
                Token::Greater => left > right,
                Token::Lesser => left < right,
                Token::GreaterThan => left <= right,
                Token::LesserThan => left >= right,
                _ => unreachable!(),
            })),
        };

        match result {
            Some(s) => s,
            None => Self::error(&format!(
                "Operand {} cannot be used between types {} and {}",
                op, left, right
            ))?,
        }
    }

    pub fn error<T>(msg: &str) -> Result<T, Error> {
        let msg = msg.to_string();
        Err(Error { msg, start: Position::new(), end: Position::new() })
    }
}
