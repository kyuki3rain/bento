#![deny(rust_2018_idioms)]

#[macro_use]
#[cfg(test)]
extern crate counted_array;

mod ast;
mod builtin;
mod environment;
mod evaluator;
mod lexer;
mod object;
mod parser;
mod repl;
mod token;

fn main() {
    let mut r = repl::Repl::new();
    r.start();
}
