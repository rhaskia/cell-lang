use std::collections::HashMap;

use crate::lexer::{Error};
use crate::{
    ast::Node,
    lexer::{Keyword, Position, Token},
};
use crate::value::Value;
use crate::positioned::Positioned;
use fehler::throws;

type Scope = HashMap<String, Value>;

pub struct Interpreter {
    instructions: Vec<Positioned<Node>>,
    index: usize,
}

impl Interpreter {
    pub fn new(instructions: Vec<Positioned<Node>>) -> Self {
        Self { instructions, index: 0 }
    }

    #[throws]
    pub fn interpret(&mut self) {
        let mut scope = Scope::new();
        self.execute(scope.clone());
    }

    #[throws]
    pub fn execute(&mut self, mut scope: Scope) -> Scope {
        let node = self.next();
        match *node {
            Node::Definition { name, var_type, value } => {
                let evaled = self.evaluate(&value, &mut scope)?;
                scope.insert(name.to_string(), evaled);
                return scope;
            }
            Node::Function { name, params, body } => {}
            Node::Return(value) => {}
            Node::ForLoop { item, iterator, body } => {}
            Node::If { expr, body } => {}
            _ => {
                self.evaluate(&node, &mut scope)?;
            }
        };
        scope.clone()
    }

    pub fn next(&mut self) -> Positioned<Node> {
        self.index += 1;
        self.instructions[self.index - 1].clone()
    }

    pub fn in_scope(scope: &Scope, name: &str) -> bool {
        scope.contains_key(name)
    }

    #[throws]
    pub fn evaluate(&mut self, value: &Positioned<Node>, scope: &mut Scope) -> Value {
        match value {
            Node::Variable(name) => {
                if !Self::in_scope(scope, &name) {
                    self.error(&format!("{name} not found in scope"))?;
                }
                scope[name].clone()
            }
            Node::Array(a) => {
                let evaled: Result<Vec<Value>, Error> = a.iter().map(|item| self.evaluate(item, scope)).collect();
                Value::Array(evaled?)
            }
            Node::Binary { left, op, right } => self.evaluate_binary(left, op, right, scope)?,
            _ => self.error("Expected expression found statement")?,
        }
    }

    #[throws]
    pub fn evaluate_binary(&mut self, left: &Box<Positioned<Node>>, op: &Token, right: &Box<Positioned<Node>>, scope: &mut Scope) -> Value {
        let start = left.start.min(right.start); 
        let end = left.end.max(right.end);

        let left = self.evaluate(left, scope)?;
        let right = self.evaluate(right, scope)?;
        let result = match op {
            Token::Minus => left.sub(right),
            Token::Plus => left.add(right),
            Token::Asterisk => left.mul(right),
            Token::Slash => left.div(right),
            Token::Or => left.or(right),
            Token::And => left.and(right),
            _ => Some(Value::Bool(match op {
                Token::Equals => left == right,
                Token::NotEquals => left != right,
                Token::Greater => left > right,
                Token::Lesser => left < right,
                Token::GreaterThan => left <= right,
                Token::LesserThan => left >= right,
                _ => unreachable!(),
            }))
        };

        match result {
            Some(s) => s,
            None => self.error(&format!("Operand {} cannot be used between types {} and {}", op, left, right))?
        }
    }

    pub fn error<T>(&self, msg: &str) -> Result<T, Error> {
        let msg = msg.to_string();
        Err(Error { msg, start: Position::new(), end: Position::new() })
    }
}

pub struct SystemProps {
    pub memory_size: (usize, usize),
    pub memory_type: usize,
}

impl SystemProps {
    pub fn new() -> Self {
        Self { memory_size: (20, 20), memory_type: 0 }
    }
}
