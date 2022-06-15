use super::object::*;
use super::*;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::rc::Rc;

pub fn new_builtins() -> HashMap<String, Rc<Object>> {
    let mut builtins = HashMap::new();
    builtins.insert(
        String::from("len"),
        Object::new_builtin(BuiltinFunc(1, strainer_len)),
    );
    builtins.insert(
        String::from("first"),
        Object::new_builtin(BuiltinFunc(2, strainer_first)),
    );
    builtins.insert(
        String::from("last"),
        Object::new_builtin(BuiltinFunc(3, strainer_last)),
    );
    builtins.insert(
        String::from("rest"),
        Object::new_builtin(BuiltinFunc(4, strainer_rest)),
    );
    builtins.insert(
        String::from("push"),
        Object::new_builtin(BuiltinFunc(5, strainer_push)),
    );
    builtins.insert(
        String::from("import"),
        Object::new_builtin(BuiltinFunc(6, strainer_import)),
    );
    builtins.insert(
        String::from("exit"),
        Object::new_builtin(BuiltinFunc(7, strainer_exit)),
    );
    builtins.insert(
        String::from("puts"),
        Object::new_builtin(BuiltinFunc(8, strainer_puts)),
    );
    builtins
}

fn strainer_exit(args: Vec<Rc<Object>>, _: &mut evaluator::Evaluator) -> Rc<Object> {
    if args.len() > 1 {
        return Object::new_error(format!(
            "wrong number of arguments. got={}, want=1",
            args.len()
        ));
    }

    if args.len() == 0 {
        return Rc::new(object::EXIT);
    }

    match &*args[0] {
        Object::Integer(_) => {
            return Rc::new(object::EXIT);
        }
        o => Object::new_error(format!("argument to `len` not supported, got {}", o)),
    }
}

fn strainer_import(args: Vec<Rc<Object>>, eval: &mut evaluator::Evaluator) -> Rc<Object> {
    if args.len() != 1 {
        return Object::new_error(format!(
            "wrong number of arguments. got={}, want=1",
            args.len()
        ));
    }
    match &*args[0] {
        Object::String(s) => {
            let data = read_to_string(s);
            let input = match data {
                Ok(content) => content,
                Err(error) => {
                    return object::Object::new_error(format!(
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
                return object::Object::new_error(s);
            }
            if program.need_next() {
                let mut s = format!("parser errors:\r\n");
                for err in p.errors {
                    s += &format!("\t{}\r\n", err);
                }
                return object::Object::new_error(s);
            }

            match eval.eval_program(program) {
                Some(evaluated) => {
                    return evaluated;
                }
                None => return object::Object::new_error(format!("cannot evaluate error!\r\n")),
            }
        }
        o => Object::new_error(format!("argument to `len` not supported, got {}", o)),
    }
}

fn strainer_len(args: Vec<Rc<Object>>, _: &mut evaluator::Evaluator) -> Rc<Object> {
    if args.len() != 1 {
        return Object::new_error(format!(
            "wrong number of arguments. got={}, want=1",
            args.len()
        ));
    }
    match &*args[0] {
        Object::String(s) => Rc::new(Object::Integer(s.len() as i64)),
        Object::Array(s) => Rc::new(Object::Integer(s.len() as i64)),
        o => Object::new_error(format!("argument to `len` not supported, got {}", o)),
    }
}

fn strainer_first(args: Vec<Rc<Object>>, _: &mut evaluator::Evaluator) -> Rc<Object> {
    match &*args[0] {
        Object::Array(o) => {
            if let Some(ao) = o.first() {
                Rc::clone(ao)
            } else {
                Rc::new(NULL)
            }
        }
        o => Object::new_error(format!("argument to `first` must be array. got {}", o)),
    }
}

fn strainer_last(args: Vec<Rc<Object>>, _: &mut evaluator::Evaluator) -> Rc<Object> {
    match &*args[0] {
        Object::Array(o) => {
            if let Some(ao) = o.last() {
                Rc::clone(ao)
            } else {
                Rc::new(NULL)
            }
        }
        o => Object::new_error(format!("argument to `last` must be array. got {}", o)),
    }
}

fn strainer_rest(args: Vec<Rc<Object>>, _: &mut evaluator::Evaluator) -> Rc<Object> {
    match &*args[0] {
        Object::Array(o) => {
            if o.len() > 0 {
                Rc::new(Object::Array(o[1..].to_vec()))
            } else {
                Rc::new(NULL)
            }
        }
        o => Object::new_error(format!("argument to `rest` must be array. got {}", o)),
    }
}

fn strainer_push(args: Vec<Rc<Object>>, _: &mut evaluator::Evaluator) -> Rc<Object> {
    match &*args[0] {
        Object::Array(o) => {
            let mut arr = o.clone();
            arr.push(Rc::clone(&args[1]));
            Rc::new(Object::Array(arr))
        }
        o => Object::new_error(format!("argument to `push` must be array. got {}", o)),
    }
}

fn strainer_puts(args: Vec<Rc<Object>>, _: &mut evaluator::Evaluator) -> Rc<Object> {
    for arg in args {
        print!("{}\r\n", arg.string());
    }

    return Rc::new(NULL);
}
