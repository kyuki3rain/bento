use super::{lexer, token};
use std::io::{stdin, stdout, Write};

pub fn start() {
    loop {
        print!(">> ");
        let _ = stdout().flush();
        let mut s = String::new();
        stdin()
            .read_line(&mut s)
            .expect("Did not enter a correct string");
        let mut l = lexer::Lexer::new(s);
        loop {
            let tok = l.next_token();
            if tok.token_type == token::TokenType::EOF {
                break;
            }
            println!("{}", tok);
        }
    }
}
