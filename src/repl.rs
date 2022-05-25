extern crate termion;

use super::{evaluator, lexer, object, parser};
use std::cell::RefCell;
use std::cmp;
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
    i: usize,
    max_view_len: u16,
}

impl Repl {
    pub fn new() -> Self {
        let evaluator = evaluator::Evaluator::new();
        let commands = vec![];
        let view = vec![vec![]];
        let len = view.len() as u16;

        return Repl {
            evaluator: RefCell::new(evaluator),
            commands,
            view,
            row_offset: 0,
            cur_x: 5,
            cur_y: 0,
            i: 0,
            max_view_len: len,
        };
    }

    fn last_x(&self) -> u16 {
        5 + self.view[(self.cur_y - self.row_offset) as usize].len() as u16
    }
    fn last_y(&self) -> u16 {
        self.row_offset + self.view.len() as u16 - 1
    }

    pub fn start(&mut self) {
        let mut stdout = stdout().into_raw_mode().unwrap();
        let stdin = stdin();
        self.fetch_row_offset();
        self.fetch_cursor_position(&mut stdout);
        self.disp(&mut stdout);

        for c in stdin.events() {
            let y = self.cur_y - self.row_offset;
            self.fetch_row_offset();
            self.cur_y = y + self.row_offset;
            if let Event::Key(key) = c.unwrap() {
                self.set_max_view_len();
                match key {
                    Key::Char('\n') => {
                        write!(
                            stdout,
                            "{}\r\n",
                            termion::cursor::Goto(self.last_x(), self.last_y())
                        )
                        .unwrap();
                        let (need_next, output) = self.enter();
                        if need_next {
                            self.set_max_view_len();
                            self.fetch_row_offset();
                        } else {
                            if output == String::from("Exit") {
                                return;
                            }
                            write!(stdout, "{}\r\n", output).unwrap();
                            let (_, y) = stdout.cursor_pos().unwrap();
                            self.row_offset = y;
                            self.view = vec![vec![]];
                        }
                        self.fetch_cursor_position(&mut stdout);
                    }
                    Key::Up => {
                        if self.cur_y > self.row_offset {
                            self.cur_y -= 1;
                            if self.cur_x > self.last_x() {
                                self.cur_x = self.last_x();
                            }
                        } else {
                            if self.commands.len() - self.i > 0 {
                                self.i += 1;
                            }
                            if self.commands.len() != 0 {
                                let view = self
                                    .get_view(self.commands[self.commands.len() - self.i].clone());
                                self.view = view;
                            }
                            self.disp(&mut stdout);
                            self.set_max_view_len();
                            self.fetch_row_offset();
                            self.cur_y = self.last_y();
                            self.cur_x = self.last_x();
                            write!(stdout, "{}", termion::cursor::Goto(self.cur_x, self.cur_y),)
                                .unwrap();
                            stdout.flush().unwrap();
                            continue;
                        }
                    }
                    Key::Down => {
                        if self.cur_y < self.last_y() - 1 {
                            self.cur_y += 1;
                            if self.cur_x > self.last_x() {
                                self.cur_x = self.last_x();
                            }
                        } else if self.i != 0 {
                            if self.i > 1 {
                                self.i -= 1;
                            }
                            if self.commands.len() != 0 {
                                let view = self
                                    .get_view(self.commands[self.commands.len() - self.i].clone());
                                self.view = view;
                            }
                            self.disp(&mut stdout);
                            self.set_max_view_len();
                            self.fetch_row_offset();
                            self.cur_y = self.last_y();
                            self.cur_x = self.last_x();
                            write!(stdout, "{}", termion::cursor::Goto(self.cur_x, self.cur_y),)
                                .unwrap();
                            stdout.flush().unwrap();
                            continue;
                        }
                    }
                    Key::Left => {
                        if self.cur_x > 5 {
                            self.cur_x -= 1;
                        }
                    }
                    Key::Right => {
                        if self.cur_x
                            < 5 + self.view[(self.cur_y - self.row_offset) as usize].len() as u16
                        {
                            self.cur_x += 1;
                        }
                    }
                    Key::Char(value) => {
                        self.view[(self.cur_y - self.row_offset) as usize]
                            .insert((self.cur_x - 5) as usize, value);
                        self.cur_x += 1;
                    }
                    Key::Backspace => {
                        if self.cur_x > 5 {
                            self.view[(self.cur_y - self.row_offset) as usize]
                                .remove((self.cur_x - 6) as usize);
                            self.cur_x -= 1;
                        }
                    }
                    Key::Ctrl('c') => {
                        write!(stdout, "\r\n").unwrap();
                        self.i = 0;
                        self.view = vec![vec![]];
                        self.max_view_len = self.view.len() as u16;
                        self.fetch_row_offset();
                        self.cur_y = self.row_offset;
                        self.cur_x = 5;
                    }
                    _ => {}
                }
                self.disp(&mut stdout);
            }
        }
    }

    fn enter(&mut self) -> (bool, String) {
        let mut output = String::new();

        let input = self.get_command();
        let l = lexer::Lexer::new(&input);
        let mut p = parser::Parser::new(l);
        let program = p.parse_program();

        if program.need_next() {
            self.view.push(vec![]);
            return (true, output);
        }

        if p.errors.len() != 0 {
            output += "parser errors:\r\n";
            for err in p.errors {
                output += &format!("\t{}\r\n", err);
            }
        } else {
            match self.evaluator.borrow_mut().eval_program(program) {
                Some(evaluated) => {
                    if let object::Object::Null = evaluated {
                    } else {
                        output += &evaluated.string();
                    }
                }
                None => output += "cannot evaluate error!",
            }
        }

        self.new_line(input);
        return (false, output);
    }

    fn set_max_view_len(&mut self) {
        self.max_view_len = cmp::max(self.max_view_len, self.view.len() as u16);
    }

    fn fetch_row_offset(&mut self) {
        let (_, rows) = termion::terminal_size().unwrap();
        self.row_offset = rows - self.max_view_len + 1;
    }

    fn new_line(&mut self, input: String) {
        if input != String::from("\r\n") {
            self.commands.push(input);
        }
        self.i = 0;
        self.max_view_len = 0;
    }

    pub fn fetch_cursor_position<T: Write>(&mut self, out: &mut T) {
        let (x, y) = out.cursor_pos().unwrap();
        self.cur_x = if x >= 5 { x } else { 5 };
        self.cur_y = y;
    }

    pub fn get_view(&self, command: String) -> Vec<Vec<char>> {
        let mut view = Vec::new();
        for row in command.split("\r\n") {
            let col: Vec<char> = row.chars().collect();
            if col.len() != 0 {
                view.push(col);
            }
        }

        return view;
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
        write!(out, "{}", termion::cursor::Goto(self.cur_x, self.cur_y),).unwrap();
        out.flush().unwrap();
    }
}
