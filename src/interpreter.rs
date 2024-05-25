use core::time;
use std::collections::HashMap;
use std::io::Write;
use std::io::stdout;
use std::thread;

use crate::lexer::Error;
use crate::positioned::Position;
use crate::positioned::Positioned;
use crate::value::Value;
use crate::{
    ast::Node,
    lexer::{Keyword, Token},
};
use fehler::throws;
use crossterm::terminal::size;

type PNode = Positioned<Node>;

pub struct Interpreter {
    instructions: Vec<Positioned<Node>>,
    constants: HashMap<String, Value>,
    functions: HashMap<String, (Vec<String>, Box<PNode>)>,
    memory: Vec<Vec<u8>>,
    match_statements: Vec<(Box<PNode>, Box<PNode>, Box<PNode>)>,
    print_statements: Vec<(Box<PNode>, Box<PNode>, Box<PNode>)>,
    index: usize,
    current_x: usize,
    current_y: usize,
    out: usize,
}

impl Interpreter {
    pub fn new(instructions: Vec<Positioned<Node>>) -> Self {
        Self { 
            instructions, 
            index: 0, 
            constants: HashMap::new(), 
            functions: HashMap::new(),
            memory: Vec::new(),
            match_statements: Vec::new(),
            current_x: 0,
            current_y: 0,
            out: 0,
            print_statements: Vec::new(),
        }
    }

    #[throws]
    pub fn interpret(&mut self) {
        self.load_instructions()?;
        self.init_screen();
        loop {
            self.draw_screen();
            self.match_cells();
            print!("\r\x1b[{}C", self.out + 1);
            stdout().flush();
            thread::sleep(time::Duration::from_millis(200));
        }
    }

    pub fn init_screen(&mut self) {
        let height = (self.memory.len() as f32 / 2.0).round() as usize;
        let width = self.memory[0].len();
        for _ in 0..height {
            println!(" {}", "▒".repeat(width));
        }
    }

    pub fn draw_screen(&mut self) {
        let height = (self.memory.len() as f32 / 2.0).round() as usize;
        let width = self.memory[0].len();

        println!("\x1b[{}A\r", height + 1);
        for y in 0..height {
            print!("\x1b[0m ");
            for x in 0..width {
                let fg = self.get_cell(x, y * 2);
                let bg = self.get_cell(x, y * 2 + 1);
                
                print!("{}{}▀", 
                       self.ansi_colour(fg, true), 
                       self.ansi_colour(bg, false));
            }
            println!("\x1b[0m");
        }
    }

    pub fn match_cells(&mut self) {
        for y in 0..self.memory.len() {
            for x in 0..self.memory[y].len() {
                self.current_y = y;
                self.current_x = x;
                self.match_cell(x, y);
            }
        }

        for y in 0..self.memory.len() {
            for x in 0..self.memory[y].len() {
                self.current_y = y;
                self.current_x = x;
                self.print_cell(x, y);
            }
        }
    }

    #[throws]
    pub fn match_cell(&mut self, x: usize, y: usize) {
        for (centre, body, result) in &self.match_statements {
            let c_eval = self.evaluate(centre)?;
            if c_eval == Value::Unknown || self.memory[y][x] == c_eval.as_num() {
                let b_eval = self.evaluate(body)?;
                if b_eval == Value::Unknown || b_eval.as_bool() {
                    let r_eval = self.evaluate(result)?;
                    self.memory[y][x] = r_eval.as_num();
                    return;
                }
            }
        }
    }

    #[throws]
    pub fn print_cell(&mut self, x: usize, y: usize) {
        for (centre, body, result) in &self.print_statements {
            let c_eval = self.evaluate(centre)?;
            if c_eval == Value::Unknown || self.memory[y][x] == c_eval.as_num() {
                let b_eval = self.evaluate(body)?;
                if b_eval == Value::Unknown || b_eval.as_bool() {
                    let r_eval = self.evaluate(result)?;
                    self.print(r_eval.as_num() as char);
                    return;
                }
            }
        }
    }

    pub fn print(&mut self, c: char) {
        print!("\r\x1b[{}C{c}", self.out + 1);
        self.out += 1;
    }

    pub fn ansi_colour(&self, v: u8, fg: bool) -> String {
        format!("\x1b[{};2;{v};{v};{v}m", if fg { "38" } else { "48" })
    }

    pub fn get_cell(&self, x: usize, y: usize) -> u8 {
        let line = match self.memory.get(y) {
            Some(line) => line,
            None => return 0,
        };

        *line.get(x).unwrap_or(&0)
    }

    pub fn get_cell_signed(&self, x: isize, y: isize) -> u8 {
        if x < 0 { return 0; }
        if y < 0 { return 0; }

        let line = match self.memory.get(y as usize) {
            Some(line) => line,
            None => return 0,
        };

        *line.get(x as usize).unwrap_or(&0)
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
                Node::Main { centre, conditional, result, print } => {
                    if print {
                        self.print_statements.push((centre, conditional, result));
                    } else {
                        self.match_statements.push((centre, conditional, result));
                    }
                }
                Node::Memory(memory) => {
                    self.memory = memory.into_iter().map(|r| r.into_iter().map(|item| self.constants[&item].clone().as_num()).collect()).collect();
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
            Node::Directional(direction) => self.get_direction(direction),
            Node::Sum(expr) => self.get_sum(expr)?,
            Node::Variable(v) => match self.constants.get(v) {
                Some(s) => s.clone(),
                None => panic!("No value for constant {v}"),
            },
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
    pub fn get_sum(&self, expr: &Box<PNode>) -> Value {
        let eval = self.evaluate(&expr)?;
        let neighbours = self.get_neighbours();
        let sum = neighbours.iter().filter(|n| Value::Int(**n) == eval).collect::<Vec<&u8>>();

        Value::Int(sum.len() as u8)
    }

    pub fn get_neighbours(&self) -> Vec<u8> {
        let (x, y) = (self.current_x as isize, self.current_y as isize);
        let offsets = vec![(1, 0), (0, 1), (-1, 0), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)];
        offsets.into_iter().map(|(ox, oy)| self.get_cell_signed(x + ox, y + oy)).collect()
    }    

    pub fn get_direction(&self, direction: &str) -> Value {
        let x = self.current_x;
        let y = self.current_y;
        match direction {
            "south" | "down" => Value::Int(self.get_cell(x, y + 1)),
            "north" | "up" => {
                if y == 0 { return Value::Int(0); }
                Value::Int(self.get_cell(x, y - 1))
            },
            "centre" | "self" => Value::Int(self.get_cell(x, y)),
            "east" | "right" => Value::Int(self.get_cell(x + 1, y)),
            "west" | "left" => {
                if x == 0 { return Value::Int(0); }
                Value::Int(self.get_cell(x - 1, y))
            },
            "northwest" => {
                if x == 0 || y == 0 { return Value::Int(0); }
                Value::Int(self.get_cell(x - 1, y - 1))
            },
            "northeast" => {
                if y == 0 { return Value::Int(0); }
                Value::Int(self.get_cell(x + 1, y - 1))
            },
            _ => Value::Unknown,
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
            Token::Mod => left.modulus(&right),
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
