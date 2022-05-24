use super::object::*;
use super::*;
use std::collections::HashMap;
use std::fs::read_to_string;

pub fn new_builtins() -> HashMap<String, Object> {
    let mut builtins = HashMap::new();
    builtins.insert(String::from("len"), Object::Builtin(bento_len));
    builtins.insert(String::from("first"), Object::Builtin(bento_first));
    builtins.insert(String::from("last"), Object::Builtin(bento_last));
    builtins.insert(String::from("rest"), Object::Builtin(bento_rest));
    builtins.insert(String::from("push"), Object::Builtin(bento_push));
    builtins.insert(String::from("import"), Object::Builtin(bento_import));
    builtins
}

fn bento_import(args: Vec<Object>, eval: &mut evaluator::Evaluator) -> Object {
    if args.len() != 1 {
        return Object::Error(format!(
            "wrong number of arguments. got={}, want=1",
            args.len()
        ));
    }
    match &args[0] {
        Object::String(s) => {
            let data = read_to_string(s);
            let input = match data {
                Ok(content) => content,
                Err(error) => {
                    return object::Object::Error(format!(
                        "Could not open or find file: {}",
                        error
                    ));
                }
            };

            let l = lexer::Lexer::new(&input);
            let mut p = parser::Parser::new(l);
            let program = p.parse_program();

            if p.errors.len() != 0 {
                let mut s = format!("parser errors:\r\n");
                for err in p.errors {
                    s += &format!("\t{}\r\n", err);
                }
                return object::Object::Error(s);
            }
            if program.need_next() {
                let mut s = format!("parser errors:\r\n");
                for err in p.errors {
                    s += &format!("\t{}\r\n", err);
                }
                return object::Object::Error(s);
            }

            match eval.eval_program(program) {
                Some(evaluated) => {
                    return evaluated;
                }
                None => return object::Object::Error(format!("cannot evaluate error!\r\n")),
            }
        }
        o => Object::Error(format!("argument to `len` not supported, got {}", o)),
    }
}

fn bento_len(args: Vec<Object>, _: &mut evaluator::Evaluator) -> Object {
    if args.len() != 1 {
        return Object::Error(format!(
            "wrong number of arguments. got={}, want=1",
            args.len()
        ));
    }
    match &args[0] {
        Object::String(s) => Object::Integer(s.len() as i64),
        Object::Array(s) => Object::Integer(s.len() as i64),
        o => Object::Error(format!("argument to `len` not supported, got {}", o)),
    }
}

fn bento_first(args: Vec<Object>, _: &mut evaluator::Evaluator) -> Object {
    match &args[0] {
        Object::Array(o) => {
            if let Some(ao) = o.first() {
                ao.clone()
            } else {
                Object::Null
            }
        }
        o => Object::Error(format!("argument to `first` must be array. got {}", o)),
    }
}

fn bento_last(args: Vec<Object>, _: &mut evaluator::Evaluator) -> Object {
    match &args[0] {
        Object::Array(o) => {
            if let Some(ao) = o.last() {
                ao.clone()
            } else {
                Object::Null
            }
        }
        o => Object::Error(format!("argument to `last` must be array. got {}", o)),
    }
}

fn bento_rest(args: Vec<Object>, _: &mut evaluator::Evaluator) -> Object {
    match &args[0] {
        Object::Array(o) => {
            if o.len() > 0 {
                Object::Array(o[1..].to_vec())
            } else {
                Object::Null
            }
        }
        o => Object::Error(format!("argument to `rest` must be array. got {}", o)),
    }
}

fn bento_push(args: Vec<Object>, _: &mut evaluator::Evaluator) -> Object {
    match &args[0] {
        Object::Array(o) => {
            let mut arr = o.clone();
            arr.push(args[1].clone());
            Object::Array(arr)
        }
        o => Object::Error(format!("argument to `push` must be array. got {}", o)),
    }
}
