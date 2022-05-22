use super::{ast, environment};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum Object {
    Integer(i64),
    String(String),
    Boolean(bool),
    Return(Box<Object>),
    Error(String),
    Builtin(fn(Vec<Object>) -> Object),
    Function {
        parameters: Vec<ast::Expression>,
        body: ast::Statement,
        env: Rc<RefCell<environment::Environment>>,
    },
    Null,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Integer(_) => return write!(f, "INTEGER"),
            Object::String(_) => return write!(f, "STRING"),
            Object::Boolean(_) => return write!(f, "BOOLEAN"),
            Object::Return(_) => return write!(f, "RETURN"),
            Object::Error(_) => return write!(f, "ERROR"),
            Object::Builtin(_) => return write!(f, "BUILTIN"),
            Object::Function {
                parameters: _,
                body: _,
                env: _,
            } => return write!(f, "FUNCTION"),
            Object::Null => return write!(f, "NULL"),
        }
    }
}

impl Object {
    pub fn string(&self) -> String {
        match self {
            Object::Integer(value) => return format!("{}", value),
            Object::String(value) => return format!("\"{}\"", value),
            Object::Boolean(value) => return format!("{}", value),
            Object::Return(value) => return format!("{}", value),
            Object::Error(value) => return format!("{}", value),
            Object::Builtin(_) => return format!("builtin-functions"),
            Object::Function {
                parameters,
                body,
                env: _,
            } => {
                let mut s = "".to_string();
                for (i, p) in parameters.iter().enumerate() {
                    if i == 0 {
                        s += &format!("{}", p);
                    } else {
                        s += &format!(", {}", p);
                    }
                }

                return format!("fn({}) {}", s, body);
            }
            Object::Null => return "NULL".to_string(),
        }
    }
}

pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;
