use std::fmt;

#[derive(PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn need_next(&self) -> bool {
        if let Some(stmt) = self.statements.last() {
            return stmt.need_next();
        } else {
            return false;
        };
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = "".to_string();
        for stmt in &self.statements {
            s += &format!("{}\r\n", stmt);
        }

        return write!(f, "{}", s);
    }
}

#[derive(Clone, PartialEq)]
pub enum Statement {
    LetStatement { name: Expression, value: Expression },
    ReturnStatement { return_value: Expression },
    ExpressionStatement { expression: Expression },
    BlockStatement { statements: Vec<Statement> },
}

impl Statement {
    pub fn need_next(&self) -> bool {
        match self {
            Statement::LetStatement { name: _, value } => value.need_next(),

            Statement::ReturnStatement { return_value } => return_value.need_next(),
            Statement::ExpressionStatement { expression } => expression.need_next(),
            Statement::BlockStatement { statements } => {
                statements.iter().any(|statement| statement.need_next())
            }
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Statement::LetStatement { name, value } => {
                return write!(f, "let {} = {};", name, value)
            }
            Statement::ReturnStatement { return_value } => {
                return write!(f, "return {};", return_value);
            }
            Statement::ExpressionStatement { expression } => return write!(f, "{}", expression),
            Statement::BlockStatement { statements } => {
                let mut s = "".to_string();
                for stmt in statements {
                    s += &format!("\t{}\r\n", stmt);
                }
                return write!(f, "{{\r\n{}}}", s);
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Expression {
    Identifier {
        value: String,
    },
    IntegerLiteral {
        value: i64,
    },
    StringLiteral {
        value: String,
    },
    PrefixExpression {
        operator: String,
        right: Box<Expression>,
    },
    InfixExpression {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    Boolean {
        value: bool,
    },
    ArrayLiteral {
        elements: Vec<Expression>,
    },
    IndexExpression {
        left: Box<Expression>,
        index: Box<Expression>,
    },
    IfExpression {
        condition: Box<Expression>,
        consequence: Box<Statement>,
        alternative: Option<Box<Statement>>,
    },
    FunctionLiteral {
        parameters: Vec<Expression>,
        body: Box<Statement>,
    },
    CallExpression {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    HashLiteral {
        pairs: Vec<(Expression, Expression)>,
    },
    NeedNext,
}

impl Expression {
    fn need_next(&self) -> bool {
        match self {
            Expression::NeedNext => true,
            _ => false,
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Identifier { value } => return write!(f, "{}", value),
            Expression::IntegerLiteral { value } => return write!(f, "{}", value),
            Expression::StringLiteral { value } => return write!(f, "\"{}\"", value),
            Expression::PrefixExpression { operator, right } => {
                return write!(f, "({}{})", operator, right);
            }
            Expression::InfixExpression {
                left,
                operator,
                right,
            } => {
                return write!(f, "({} {} {})", left, operator, right);
            }
            Expression::Boolean { value } => return write!(f, "{}", value),
            Expression::ArrayLiteral { elements } => {
                let mut s = "".to_string();
                for (i, p) in elements.iter().enumerate() {
                    if i == 0 {
                        s += &format!("{}", p);
                    } else {
                        s += &format!(", {}", p);
                    }
                }
                return write!(f, "[{}]", s);
            }
            Expression::IndexExpression { left, index } => {
                return write!(f, "({})[{}]", left, index);
            }
            Expression::IfExpression {
                condition,
                consequence,
                alternative,
            } => match alternative {
                Some(alt) => return write!(f, "if ({}) {} else {}", condition, consequence, alt),
                None => return write!(f, "if ({}) {}", condition, consequence),
            },
            Expression::FunctionLiteral { parameters, body } => {
                let mut s = "".to_string();
                for (i, p) in parameters.iter().enumerate() {
                    if i == 0 {
                        s += &format!("{}", p);
                    } else {
                        s += &format!(", {}", p);
                    }
                }
                return write!(f, "fn ({}) {}", s, body);
            }
            Expression::CallExpression {
                function,
                arguments,
            } => {
                let mut s = "".to_string();
                for (i, a) in arguments.iter().enumerate() {
                    if i == 0 {
                        s += &format!("{}", a);
                    } else {
                        s += &format!(", {}", a);
                    }
                }
                return write!(f, "{}({})", function, s);
            }
            Expression::HashLiteral { pairs } => {
                let mut s = "{ ".to_string();
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i == 0 {
                        s += &format!("{}: {}", key, value);
                    } else {
                        s += &format!(", {}: {}", key, value);
                    }
                }
                s += " }";
                return write!(f, "{}", s);
            }
            Expression::NeedNext => return write!(f, ""),
        }
    }
}

#[cfg(test)]
mod ast_tests {
    use super::*;

    #[test]
    fn test_string() {
        let program = Program {
            statements: vec![Statement::LetStatement {
                name: Expression::Identifier {
                    value: "myVar".to_string(),
                },
                value: Expression::Identifier {
                    value: "anotherVar".to_string(),
                },
            }],
        };

        assert_eq!(format!("{}", program), "let myVar = anotherVar;\r\n");
    }
}
