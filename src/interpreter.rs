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

pub struct Interpreter {
    instructions: Vec<Positioned<Node>>,
    constants: HashMap<String, Value>,
    index: usize,
}

impl Interpreter {
    pub fn new(instructions: Vec<Positioned<Node>>) -> Self {
        Self { instructions, index: 0 }
    }

    #[throws]
    pub fn interpret(&mut self) {
        let mut scope = Scope::new();
        self.load_constants()?;
        self.execute(scope.clone())?;
    }

    #[throws]
    pub fn load_constants(&mut self) {
        for instruction in &self.instructions {
            if let Node::Definition{name, value} = &instruction.inner {
                let value = self.evaluate(&value)?;
                self.constants.insert(name.to_string(), value);
            }
        }
    }

    #[throws]
    pub fn execute(&mut self, mut scope: Scope) -> Scope {
        let node = self.next();
        match node.inner {
            _ => {
                self.evaluate(&node)?;
            }
        };
        scope.clone()
    }

    pub fn next(&mut self) -> Positioned<Node> {
        self.index += 1;
        self.instructions[self.index - 1].clone()
    }

    #[throws]
    pub fn evaluate(&self, value: &Positioned<Node>) -> Value {
        match &value.inner {
            Node::Variable(v) => self.constants[v],
            Node::Array(a) => {
                let evaled: Result<Vec<Value>, Error> =
                    a.iter().map(|item| Self::evaluate(item, scope)).collect();
                Value::Array(evaled?)
            }
            Node::Binary { left, op, right } => Self::evaluate_binary(&left, &op, &right, scope)?,
            Node::Literal(v) => v.clone(),
            _ => Self::error("Expected expression found statement")?,
        }
    }

    #[throws]
    pub fn evaluate_binary(
        left: &Box<Positioned<Node>>,
        op: &Token,
        right: &Box<Positioned<Node>>,
        scope: &mut Scope,
    ) -> Value {
        let start = left.start.min(right.start);
        let end = left.end.max(right.end);

        let left = Self::evaluate(left, scope)?;
        let right = Self::evaluate(right, scope)?;
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
