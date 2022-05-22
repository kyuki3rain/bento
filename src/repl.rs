extern crate termion;

use super::{evaluator, lexer, parser};
use std::io::{stdin, stdout, Write};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub fn start() {
    let mut evaluator = evaluator::Evaluator::new();
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut input = "".to_string();

    print!(">>> ");
    stdout.flush().unwrap();

    for c in stdin.events() {
        let evt = c.unwrap();
        match evt {
            Event::Key(Key::Char('\n')) => {
                print!("\r\n");
                let l = lexer::Lexer::new(&input);
                let mut p = parser::Parser::new(l);
                let program = p.parse_program();
                if program.need_next() {
                    input += "\r\n";
                    print!("... ");
                    stdout.flush().unwrap();
                    continue;
                }

                if p.errors.len() != 0 {
                    print!("parser errors:\r\n");
                    for err in p.errors {
                        write!(stdout, "\t{}\r\n", err).unwrap();
                    }
                } else {
                    match evaluator.eval_program(program) {
                        Some(evaluated) => {
                            write!(stdout, "{}\r\n", evaluated.string()).unwrap();
                        }
                        None => print!("cannot evaluate error!\r\n"),
                    }
                }
                input = "".to_string();
                print!(">>> ");
                stdout.flush().unwrap();
            }
            Event::Key(Key::Backspace) => {
                input.pop();
                write!(
                    stdout,
                    "{}{}",
                    termion::cursor::Left(1),
                    termion::clear::UntilNewline
                );
                stdout.flush().unwrap();
            }
            Event::Key(Key::Left) => {
                input.pop();
                write!(stdout, "{}", termion::cursor::Left(1));
                stdout.flush().unwrap();
            }
            Event::Key(Key::Right) => {
                input.pop();
                write!(stdout, "{}", termion::cursor::Right(1));
                stdout.flush().unwrap();
            }
            Event::Key(Key::Char(value)) => {
                write!(stdout, "{}", value).unwrap();
                stdout.flush().unwrap();
                input += &value.to_string();
            }
            Event::Key(Key::Ctrl('c')) => {
                return;
            }
            _ => {}
        }
    }
}
