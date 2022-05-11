use super::{ast, environment};
use std::fmt;

#[derive(Clone)]
pub enum Object<'a> {
    Integer(i64),
    Boolean(bool),
    Return(Box<Object<'a>>),
    Error(String),
    Function {
        parameters: Vec<ast::Expression>,
        body: ast::Statement,
        env: &'a environment::Environment<'a>,
    },
    Null,
}

impl<'a> fmt::Display for Object<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Integer(_) => return write!(f, "INTEGER"),
            Object::Boolean(_) => return write!(f, "BOOLEAN"),
            Object::Return(_) => return write!(f, "RETURN"),
            Object::Error(_) => return write!(f, "ERROR"),
            Object::Function {
                parameters: _,
                body: _,
                env: _,
            } => return write!(f, "FUNCTION"),
            Object::Null => return write!(f, "NULL"),
        }
    }
}

impl<'a> Object<'a> {
    pub fn string(&self) -> String {
        match self {
            Object::Integer(value) => return format!("{}", value),
            Object::Boolean(value) => return format!("{}", value),
            Object::Return(value) => return format!("{}", value),
            Object::Error(value) => return format!("{}", value),
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
