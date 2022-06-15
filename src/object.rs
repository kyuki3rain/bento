use super::{ast, environment, evaluator};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone)]
pub struct BuiltinFunc(
    pub i64,
    pub fn(Vec<Object>, &mut evaluator::Evaluator) -> Object,
);
impl PartialEq for BuiltinFunc {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, PartialEq)]
pub enum Object {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Return(Box<Object>),
    Error(String),
    Builtin(BuiltinFunc),
    Array(Vec<Object>),
    Hash(HashMap<Object, Object>),
    Function {
        parameters: Vec<ast::Expression>,
        body: ast::Statement,
        env: Rc<RefCell<environment::Environment>>,
    },
    Null,
    Exit(i32),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Integer(_) => return write!(f, "INTEGER"),
            Object::Float(_) => return write!(f, "FLOAT"),
            Object::String(_) => return write!(f, "STRING"),
            Object::Boolean(_) => return write!(f, "BOOLEAN"),
            Object::Return(_) => return write!(f, "RETURN"),
            Object::Error(_) => return write!(f, "ERROR"),
            Object::Builtin(_) => return write!(f, "BUILTIN"),
            Object::Array(_) => return write!(f, "ARRAY"),
            Object::Hash(_) => return write!(f, "HASH"),
            Object::Function {
                parameters: _,
                body: _,
                env: _,
            } => return write!(f, "FUNCTION"),
            Object::Null => return write!(f, "NULL"),
            Object::Exit(_) => return write!(f, "Exit"),
        }
    }
}

impl Object {
    pub fn string(&self) -> String {
        match self {
            Object::Integer(value) => return format!("{}", value),
            Object::Float(value) => return format!("{}", value),
            Object::String(value) => return format!("\"{}\"", value),
            Object::Boolean(value) => return format!("{}", value),
            Object::Return(value) => return format!("{}", value),
            Object::Error(value) => return format!("{}", value),
            Object::Builtin(_) => return format!("builtin-functions"),
            Object::Array(array) => {
                let mut s = "[".to_string();
                for object in array {
                    s += &format!("{}, ", object.string());
                }
                s += "]";
                return s;
            }
            Object::Hash(pairs) => {
                let mut s = "{ ".to_string();
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i == 0 {
                        s += &format!("{}: {}", key.string(), value.string());
                    } else {
                        s += &format!(", {}: {}", key.string(), value.string());
                    }
                }
                s += " }";
                return s;
            }
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
            Object::Exit(_) => return "Exit".to_string(),
        }
    }
}

impl Eq for Object {}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Object::Integer(ref i) => i.hash(state),
            Object::Boolean(ref b) => b.hash(state),
            Object::String(ref s) => s.hash(state),
            _ => "".hash(state),
        }
    }
}

pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;
