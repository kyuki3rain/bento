use super::token;
use std::fmt;

pub struct Program {
    pub statements: Vec<Statement>,
}

#[allow(dead_code)]
impl Program {
    pub fn token_literal(&self) -> &str {
        if self.statements.len() > 0 {
            return self.statements[0].token_literal();
        } else {
            return " ";
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = "".to_string();
        for stmt in &self.statements {
            s += &format!("{}\n", stmt);
        }

        return write!(f, "{}", s);
    }
}

#[derive(Clone)]
pub enum Statement {
    LetStatement {
        token: token::Token,
        name: Expression,
        value: Expression,
    },
    ReturnStatement {
        token: token::Token,
        return_value: Expression,
    },
    ExpressionStatement {
        token: token::Token,
        expression: Expression,
    },
    BlockStatement {
        token: token::Token,
        statements: Vec<Statement>,
    },
}

impl Statement {
    pub fn token_literal(&self) -> &str {
        match self {
            Statement::LetStatement {
                token,
                name: _,
                value: _,
            } => return &token.literal,
            Statement::ReturnStatement {
                token,
                return_value: _,
            } => return &token.literal,
            Statement::ExpressionStatement {
                token,
                expression: _,
            } => return &token.literal,
            Statement::BlockStatement {
                token,
                statements: _,
            } => return &token.literal,
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Statement::LetStatement { token, name, value } => {
                return write!(f, "{} {} = {};", token.literal, name, value)
            }
            Statement::ReturnStatement {
                token,
                return_value,
            } => {
                return write!(f, "{} {};", token.literal, return_value);
            }
            Statement::ExpressionStatement {
                token: _,
                expression,
            } => return write!(f, "{}", expression),
            Statement::BlockStatement {
                token: _,
                statements,
            } => {
                let mut s = "".to_string();
                for stmt in statements {
                    s += &format!("\t{}\n", stmt);
                }
                return write!(f, "{{\n{}}}", s);
            }
        }
    }
}

#[derive(Clone)]
pub enum Expression {
    Identifier {
        token: token::Token,
        value: String,
    },
    IntegerLiteral {
        token: token::Token,
        value: i64,
    },
    PrefixExpression {
        token: token::Token,
        operator: String,
        right: Box<Expression>,
    },
    InfixExpression {
        token: token::Token,
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    Boolean {
        token: token::Token,
        value: bool,
    },
    IfExpression {
        token: token::Token,
        condition: Box<Expression>,
        consequence: Box<Statement>,
        alternative: Option<Box<Statement>>,
    },
    FunctionLiteral {
        token: token::Token,
        parameters: Vec<Expression>,
        body: Box<Statement>,
    },
    CallExpression {
        token: token::Token,
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

#[allow(dead_code)]
impl Expression {
    pub fn token_literal(&self) -> &str {
        match self {
            Expression::Identifier { token, value: _ } => return &token.literal,
            Expression::IntegerLiteral { token, value: _ } => return &token.literal,
            Expression::PrefixExpression {
                token,
                operator: _,
                right: _,
            } => return &token.literal,
            Expression::InfixExpression {
                token,
                left: _,
                operator: _,
                right: _,
            } => return &token.literal,
            Expression::Boolean { token, value: _ } => return &token.literal,
            Expression::IfExpression {
                token,
                condition: _,
                consequence: _,
                alternative: _,
            } => return &token.literal,
            Expression::FunctionLiteral {
                token,
                parameters: _,
                body: _,
            } => return &token.literal,
            Expression::CallExpression {
                token,
                function: _,
                arguments: _,
            } => return &token.literal,
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Identifier { token: _, value } => return write!(f, "{}", value),
            Expression::IntegerLiteral { token: _, value } => return write!(f, "{}", value),
            Expression::PrefixExpression {
                token: _,
                operator,
                right,
            } => {
                return write!(f, "({}{})", operator, right);
            }
            Expression::InfixExpression {
                token: _,
                left,
                operator,
                right,
            } => {
                return write!(f, "({} {} {})", left, operator, right);
            }
            Expression::Boolean { token: _, value } => return write!(f, "{}", value),
            Expression::IfExpression {
                token: _,
                condition,
                consequence,
                alternative,
            } => match alternative {
                Some(alt) => return write!(f, "if ({}) {} else {}", condition, consequence, alt),
                None => return write!(f, "if ({}) {}", condition, consequence),
            },
            Expression::FunctionLiteral {
                token: _,
                parameters,
                body,
            } => {
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
                token: _,
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
                token: token::Token {
                    token_type: token::TokenType::LET,
                    literal: "let".to_string(),
                },
                name: Expression::Identifier {
                    token: token::Token {
                        token_type: token::TokenType::IDENT,
                        literal: "myVar".to_string(),
                    },
                    value: "myVar".to_string(),
                },
                value: Expression::Identifier {
                    token: token::Token {
                        token_type: token::TokenType::IDENT,
                        literal: "anotherVar".to_string(),
                    },
                    value: "anotherVar".to_string(),
                },
            }],
        };

        assert_eq!(format!("{}", program), "let myVar = anotherVar;\n");
    }
}
