mod lexer;
mod ast;
mod parser;
mod macros;

fn main() {
    let program = r#"
        sys screen = 20;
        sys type = 0;

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
    println!("{tokens:?}");
    let mut parser = parser::Parser::new(tokens.unwrap());
    let ast = parser.parse();
    println!("{ast:?}");
}
