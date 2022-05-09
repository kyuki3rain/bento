use super::{ast, environment};
use std::fmt;

#[derive(Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Return(Box<Object>),
    Error(String),
    Function {
        parameters: Vec<ast::Expression>,
        body: ast::Statement,
        env: environment::Environment,
    },
    Null,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Integer(value) => return write!(f, "INTEGER"),
            Object::Boolean(value) => return write!(f, "BOOLEAN"),
            Object::Return(value) => return write!(f, "RETURN"),
            Object::Error(value) => return write!(f, "ERROR"),
            Object::Function {
                parameters,
                body,
                env,
            } => return write!(f, "FUNCTION"),
            Object::Null => return write!(f, "NULL"),
        }
    }
}

impl Object {
    pub fn string(&self) -> String {
        match self {
            Object::Integer(value) => return format!("{}", value),
            Object::Boolean(value) => return format!("{}", value),
            Object::Return(value) => return format!("{}", value),
            Object::Error(value) => return format!("{}", value),
            Object::Function {
                parameters,
                body,
                env,
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
