use lexer::Error;

mod lexer;
mod ast;
mod parser;
mod macros;

fn main() {
    let program = &std::fs::read_to_string("main.cell").unwrap();

    println!("{program}");
    let mut lexer = lexer::Lexer::new(program.to_string());
    let tokens = lexer.scan_tokens();
    match tokens {
        Ok(ref nodes) => println!("{nodes:?}"),
        Err(err) => {
            eprintln!("{}", build_error(program, err));
            return;
        },
    }

    let mut parser = parser::Parser::new(tokens.unwrap());
    let ast = parser.parse();
    match ast {
        Ok(nodes) => println!("{nodes:?}"),
        Err(err) => eprintln!("{}", build_error(program, err)),
    }
}

pub fn build_error(program: &str, error: Error) -> String {
    let mut lines = program.lines();
    let Error { msg, start, end } = error;
    let error_point = format!("{}{}", " ".repeat(start.col - 2), "^".repeat(end.col - start.col + 1));
    let line = lines.nth(start.line - 1).unwrap();
    println!("{start:?}, {end:?}");

    format!("
\x1b[91m\x1b[1mError:\x1b[m {}:
-> main.cell:{}:{} 
| {}
| \x1b[92m\x1b[1m{}
    ", msg, start.line, start.col, line, error_point)
}
