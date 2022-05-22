use super::object::*;
use std::collections::HashMap;

pub fn new_builtins() -> HashMap<String, Object> {
    let mut builtins = HashMap::new();
    builtins.insert(String::from("len"), Object::Builtin(monkey_len));
    builtins
}

fn monkey_len(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(format!(
            "wrong number of arguments. got={}, want=1",
            args.len()
        ));
    }
    match &args[0] {
        Object::String(s) => Object::Integer(s.len() as i64),
        o => Object::Error(format!("argument to `len` not supported, got {}", o)),
    }
}

// fn monkey_first(args: Vec<Object>) -> Object {
//     match &args[0] {
//         Object::Array(o) => {
//             if let Some(ao) = o.first() {
//                 ao.clone()
//             } else {
//                 Object::Null
//             }
//         }
//         o => Object::Error(format!("argument to `first` must be array. got {}", o)),
//     }
// }

// fn monkey_last(args: Vec<Object>) -> Object {
//     match &args[0] {
//         Object::Array(o) => {
//             if let Some(ao) = o.last() {
//                 ao.clone()
//             } else {
//                 Object::Null
//             }
//         }
//         o => Object::Error(format!("argument to `last` must be array. got {}", o)),
//     }
// }

// fn monkey_rest(args: Vec<Object>) -> Object {
//     match &args[0] {
//         Object::Array(o) => {
//             if o.len() > 0 {
//                 Object::Array(o[1..].to_vec())
//             } else {
//                 Object::Null
//             }
//         }
//         o => Object::Error(format!("argument to `rest` must be array. got {}", o)),
//     }
// }

// fn monkey_push(args: Vec<Object>) -> Object {
//     match &args[0] {
//         Object::Array(o) => {
//             let mut arr = o.clone();
//             arr.push(args[1].clone());
//             Object::Array(arr)
//         }
//         o => Object::Error(format!("argument to `push` must be array. got {}", o)),
//     }
// }
