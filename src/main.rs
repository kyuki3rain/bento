#[macro_use]
#[cfg(test)]
extern crate counted_array;
extern crate phf;

mod lexer;
mod repl;
mod token;

fn main() {
    repl::start();
}
