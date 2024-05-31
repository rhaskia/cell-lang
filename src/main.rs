#![feature(never_type)]
use lexer::Error;
mod ast;
mod interpreter;
mod lexer;
mod parser;
mod positioned;
mod value;

fn main() {
    let arg = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("No file supplied. Please specify a file to run.");
            return;
        }
    };
    
    let program = match std::fs::read_to_string(arg) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open file: {err:?}");
            return;
        }
    };

    match run_program(&program) {
        Ok(_) => {},
        Err(err) => eprintln!("{}", build_error(&program, err)),
    }
}

fn run_program(program: &str) -> Result<!, Error> {
    let mut lexer = lexer::Lexer::new(program.to_string());
    let tokens = lexer.scan_tokens()?;

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;

    let mut interp = interpreter::Interpreter::new(ast);
    interp.interpret()?;
}

pub fn build_error(program: &str, error: Error) -> String {
    let mut lines = program.lines();

    if error.start.end {
        return String::from(error.msg);
    }

    let Error { msg, start, end } = error;
    let error_point =
        format!("{}{}", " ".repeat(start.col - 2), "^".repeat(end.col - start.col + 1));
    let line = lines.nth(start.line - 1).unwrap();

    format!(
        "
\x1b[91m\x1b[1mError:\x1b[m {}:
-> main.cell:{}:{} 
| {}
| \x1b[92m\x1b[1m{}
    ",
        msg, start.line, start.col, line, error_point
    )
}
