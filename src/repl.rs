extern crate termion;

use super::{evaluator, lexer, parser};
use std::cell::RefCell;
use std::io::{stdin, stdout, Write};
use termion::cursor::DetectCursorPos;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub struct Repl {
    evaluator: RefCell<evaluator::Evaluator>,
    commands: Vec<String>,
    view: Vec<Vec<char>>,
    row_offset: u16,
    cur_x: u16,
    cur_y: u16,
}

impl Repl {
    pub fn new() -> Self {
        let evaluator = evaluator::Evaluator::new();
        let commands = vec![];
        let view = vec![vec![]];

        return Repl {
            evaluator: RefCell::new(evaluator),
            commands,
            view,
            row_offset: 0,
            cur_x: 0,
            cur_y: 0,
        };
    }

    pub fn start(&mut self) {
        let mut stdout = stdout().into_raw_mode().unwrap();

        let stdin = stdin();
        self.fetch_cursor_position(&mut stdout);
        self.disp(&mut stdout);

        for c in stdin.events() {
            self.fetch_cursor_position(&mut stdout);
            let evt = c.unwrap();
            match evt {
                Event::Key(Key::Char('\n')) => {
                    self.enter(&mut stdout);
                }
                Event::Key(Key::Char(value)) => {
                    self.view[(self.cur_y - self.row_offset) as usize]
                        .insert((self.cur_x - 5) as usize, value);
                }
                Event::Key(Key::Ctrl('c')) => {
                    return;
                }
                _ => {}
            }
            self.disp(&mut stdout);
        }
    }

    pub fn fetch_cursor_position<T: Write>(&mut self, out: &mut T) {
        let (x, y) = out.cursor_pos().unwrap();
        self.cur_x = x;
        self.cur_y = y;
        self.row_offset = y - self.view.len() as u16 + 1;
    }

    pub fn get_command(&self) -> String {
        let mut input = "".to_string();
        for row in &self.view {
            for c in row {
                input.push(*c);
            }
            input += "\r\n";
        }

        return input;
    }

    pub fn enter<T: Write>(&mut self, out: &mut T) {
        let input = self.get_command();

        let l = lexer::Lexer::new(&input);
        let mut p = parser::Parser::new(l);
        let program = p.parse_program();
        if program.need_next() {
            self.view.push(vec![]);
            return;
        }

        write!(out, "\r\n");
        if p.errors.len() != 0 {
            print!("parser errors:\r\n");
            for err in p.errors {
                write!(out, "\t{}\r\n", err).unwrap();
            }
        } else {
            match self.evaluator.borrow_mut().eval_program(program) {
                Some(evaluated) => {
                    write!(out, "{}\r\n", evaluated.string()).unwrap();
                    self.commands.push(input);
                }
                None => print!("cannot evaluate error!\r\n"),
            }
        }

        self.view = vec![vec![]];
        self.fetch_cursor_position(out);
    }

    pub fn disp<T: Write>(&self, out: &mut T) {
        write!(
            out,
            "{}{}",
            termion::cursor::Goto(1, self.row_offset),
            termion::clear::AfterCursor
        )
        .unwrap();
        for (i, row) in self.view.iter().enumerate() {
            if i == 0 {
                write!(out, ">>> ").unwrap();
            } else {
                write!(out, "\r\n... ").unwrap();
            }
            for c in row {
                write!(out, "{}", c).unwrap();
            }
        }
        out.flush().unwrap();
    }
}
