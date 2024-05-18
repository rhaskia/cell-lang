use std::collections::HashMap;

use strum_macros::Display;

#[derive(Debug, Clone, PartialEq, PartialOrd, Display)]
pub enum Value {
    Int(i32),
    Float(f32),
    String(String),
    Char(char),
    Array(Vec<Value>),
    Bool(bool),
    Function(fn(HashMap<String, Value>) -> Value),
    Unit,
}

type Error = String;

impl Value {
    pub fn and(&self, other: &Value) -> Option<Value> {
        Some(Value::Bool(self.as_bool() && other.as_bool()))
    }

    pub fn or(&self, other: &Value) -> Option<Value> {
        Some(Value::Bool(self.as_bool() || other.as_bool()))
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f >= 1.0,
            Value::String(s) => s.len() != 0,
            Value::Char(c) => *c != '\0',
            Value::Array(a) => a.len() != 0,
            Value::Bool(b) => *b,
            Value::Function(_) => true,
            Value::Unit => false,
        }
    }

    pub fn mul(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Self::Int(i), Self::Int(j)) => Some(Value::Int(i * j)),
            (Self::Int(i), Self::Float(j)) => Some(Value::Float((*i as f32) * j)),
            (Self::Float(i), Self::Float(j)) => Some(Value::Float(*i * j)),
            (Self::Float(i), Self::Int(j)) => Some(Value::Float(i * *j as f32)),
            (Self::Array(a), Self::Int(i)) => Some(Self::repeat_array(a.clone(), *i)),
            _ => None,
        }
    }

    pub fn repeat_array(array: Vec<Value>, repeat: i32) -> Value {
        Value::Array(vec![array; repeat as usize].iter().flat_map(|i| i.clone()).collect())
    }

    pub fn div(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Self::Int(i), Self::Int(j)) => Some(Value::Int(i / j)),
            (Self::Int(i), Self::Float(j)) => Some(Value::Float((*i as f32) / j)),
            (Self::Float(i), Self::Float(j)) => Some(Value::Float(*i / j)),
            (Self::Float(i), Self::Int(j)) => Some(Value::Float(i / *j as f32)),
            _ => None,
        }
    }

    pub fn add(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Self::Int(i), Self::Int(j)) => Some(Value::Int(i + j)),
            (Self::Int(i), Self::Float(j)) => Some(Value::Float((*i as f32) + j)),
            (Self::Float(i), Self::Int(j)) => Some(Value::Float(i + *j as f32)),
            (Self::Float(i), Self::Float(j)) => Some(Value::Float(*i + j)),
            (Self::String(s), Self::String(t)) => Some(Value::String(format!("{}{}", s, t))),
            _ => None,
        }
    }

    pub fn sub(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Self::Int(i), Self::Int(j)) => Some(Value::Int(i - j)),
            (Self::Int(i), Self::Float(j)) => Some(Value::Float((*i as f32) - j)),
            (Self::Float(i), Self::Float(j)) => Some(Value::Float(*i - j)),
            (Self::Float(i), Self::Int(j)) => Some(Value::Float(i - *j as f32)),
            _ => None,
        }
    }
}
