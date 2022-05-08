#[macro_use]
#[cfg(test)]
extern crate counted_array;

mod ast;
mod lexer;
mod parser;
mod repl;
mod token;

fn main() {
    repl::start();
}
