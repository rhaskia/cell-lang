use lexer::Error;

mod lexer;
mod ast;
mod parser;
mod macros;

fn main() {
    let program = r#"
        sys screen = 20;
        sys type = 0;
        sys jklsjfl dsfjkal;

        fn update(x, neighbours) {
            int count = 0;
            for neighbour in neighbours {
                if neightbour > 1 {
                    count = count + 1;
                }
            }
            return count;
        }

        fn out(buffer) {
            print()
        }
        "#;

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
    let line = lines.nth(start.line).unwrap();
    println!("{start:?}, {end:?}");

    format!("
\x1b[91m\x1b[1mError:\x1b[m {}:
-> main.cell:{}:{} 
| {}
| \x1b[92m\x1b[1m{}
    ", msg, start.col, start.line, line, error_point)
}
