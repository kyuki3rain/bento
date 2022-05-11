use super::{environment, evaluator, lexer, parser};
use std::io::{stdin, stdout, Write};

pub fn start() {
    let mut prev_env = environment::Environment::new();
    loop {
        let mut env = prev_env.clone();
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

        match evaluator::eval_program(program, &mut env) {
            (Some(evaluated), penv) => println!("{}", evaluated.string()),
            (None, penv) => prev_env = *penv,
        }
    }
}
