use super::{lexer, parser};
use std::io::{stdin, stdout, Write};

pub fn start() {
    loop {
        print!(">> ");
        let _ = stdout().flush();
        let mut s = String::new();
        stdin()
            .read_line(&mut s)
            .expect("Did not enter a correct string");
        let l = lexer::Lexer::new(s);
        let mut p = parser::Parser::new(l);

        let program = p.parse_program();
        if p.errors.len() != 0 {
            println!("parser errors:");
            for err in p.errors {
                println!("\t{}", err);
            }
            continue;
        }

        println!("{}", program);
    }
}
