mod lexer;
mod parser;

fn main() {
    let program = r#"
        sys screen = (20, 20);
        string text_name = "hi";
        float i = 20.1;

        main(x) {
            text = "no";
        }"#;

    println!("{program}");
    let mut lexer = lexer::Lexer::new(program.to_string());
    let tokens = lexer.scan_tokens();
    println!("{tokens:?}")
}
