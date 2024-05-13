mod lexer;
mod ast;
mod parser;
mod macros;

fn main() {
    let program = r#"
        sys screen = (20, 20);
        string text_name = "hi";
        float i = 20.1;

        ruleset 3 sand(x, s) {
            _ _ _
            _ s _
            _ x _
            => 
            _ _ _
            _ x _
            _ s _
        }

        update {
            text = "no";
        }"#;

    let test = r#"
        60 > 70;
    60 >= 89;
    89 <=;
    & && & &&
        "#;

    println!("{program}");
    let mut lexer = lexer::Lexer::new(test.to_string());
    let tokens = lexer.scan_tokens();
    println!("{tokens:?}")
}
