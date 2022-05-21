use super::{ast, lexer, token};

#[allow(dead_code)]
#[derive(PartialEq, PartialOrd, Debug)]
pub enum Precedence {
    LOWEST,
    EQUALS,
    LESSGREATER,
    SUM,
    PRODUCT,
    PREFIX,
    CALL,
}

#[allow(dead_code)]
pub fn token_type_to_precedence(t: &token::TokenType) -> Precedence {
    match t {
        token::TokenType::EQ => return Precedence::EQUALS,
        token::TokenType::NOTEQ => return Precedence::EQUALS,
        token::TokenType::LT => return Precedence::LESSGREATER,
        token::TokenType::GT => return Precedence::LESSGREATER,
        token::TokenType::PLUS => return Precedence::SUM,
        token::TokenType::MINUS => return Precedence::SUM,
        token::TokenType::SLASH => return Precedence::PRODUCT,
        token::TokenType::ASTERISK => return Precedence::PRODUCT,
        token::TokenType::LPAREN => return Precedence::CALL,
        _ => return Precedence::LOWEST,
    }
}

#[allow(dead_code)]
pub struct Parser {
    l: lexer::Lexer,
    cur_token: token::Token,
    peek_token: token::Token,
    pub errors: Vec<String>,
}

#[allow(dead_code)]
impl Parser {
    pub fn new(l: lexer::Lexer) -> Parser {
        let mut p = Parser {
            l: l,
            cur_token: token::Token {
                token_type: token::TokenType::ILLEGAL,
                literal: "".to_string(),
            },
            peek_token: token::Token {
                token_type: token::TokenType::ILLEGAL,
                literal: "".to_string(),
            },
            errors: Vec::new(),
        };

        p.next_token();
        p.next_token();

        return p;
    }

    fn peek_error(&mut self, t: token::TokenType) {
        self.errors.push(String::from(format!(
            "\nexpected next token to be {:?}, got {:?} instead.",
            t, self.peek_token.token_type
        )))
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.l.next_token();
    }

    pub fn parse_program(&mut self) -> ast::Program {
        let mut program = ast::Program {
            statements: Vec::new(),
        };
        while self.cur_token.token_type != token::TokenType::EOF {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }
            self.next_token();
        }
        return program;
    }
    fn parse_statement(&mut self) -> Option<ast::Statement> {
        match self.cur_token.token_type {
            token::TokenType::LET => return self.parse_let_statement(),
            token::TokenType::RETURN => return self.parse_return_statement(),
            _ => return self.parse_expression_statement(),
        }
    }
    fn parse_let_statement(&mut self) -> Option<ast::Statement> {
        if !self.expect_peek(token::TokenType::IDENT) {
            return None;
        }
        let name = ast::Expression::Identifier {
            value: (&self.cur_token.literal).to_string(),
        };
        if !self.expect_peek(token::TokenType::ASSIGN) {
            return None;
        }

        self.next_token();

        if let Some(expression) = self.parse_expression(Precedence::LOWEST) {
            let stmt = ast::Statement::LetStatement {
                name,
                value: expression,
            };
            if self.peek_token_is(&token::TokenType::SEMICOLON) {
                self.next_token();
            }
            return Some(stmt);
        } else {
            return None;
        }
    }
    fn parse_return_statement(&mut self) -> Option<ast::Statement> {
        self.next_token();

        if let Some(expression) = self.parse_expression(Precedence::LOWEST) {
            let stmt = ast::Statement::ReturnStatement {
                return_value: expression,
            };
            if self.peek_token_is(&token::TokenType::SEMICOLON) {
                self.next_token();
            }
            return Some(stmt);
        } else {
            return None;
        }
    }
    fn parse_expression_statement(&mut self) -> Option<ast::Statement> {
        if let Some(expression) = self.parse_expression(Precedence::LOWEST) {
            let stmt = ast::Statement::ExpressionStatement {
                expression: expression,
            };
            if self.peek_token_is(&token::TokenType::SEMICOLON) {
                self.next_token();
            }
            return Some(stmt);
        } else {
            return None;
        }
    }

    fn parse_block_statement(&mut self) -> Option<ast::Statement> {
        let mut statements = Vec::new();

        self.next_token();

        while !self.cur_token_is(&token::TokenType::RBRACE)
            && !self.cur_token_is(&token::TokenType::EOF)
        {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        return Some(ast::Statement::BlockStatement { statements });
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<ast::Expression> {
        if let Some(mut left_exp) = self.parse_prefix_expression_fns() {
            while !self.peek_token_is(&token::TokenType::SEMICOLON)
                && precedence < self.peek_precedence()
            {
                self.next_token();
                if let Some(left_exp_new) = self.parse_infix_expression_fns(
                    self.cur_token.token_type.clone(),
                    Box::new(left_exp.clone()),
                ) {
                    left_exp = left_exp_new;
                } else {
                    return Some(left_exp);
                }
            }

            return Some(left_exp);
        } else {
            self.no_prefix_parse_fn_error();
            return None;
        }
    }

    fn parse_prefix_expression_fns(&mut self) -> Option<ast::Expression> {
        match self.cur_token.token_type {
            token::TokenType::IDENT => return Some(self.parse_identifier()),
            token::TokenType::INT => return self.parse_integer_literal(),
            token::TokenType::BANG => return self.parse_prefix_expression(),
            token::TokenType::MINUS => return self.parse_prefix_expression(),
            token::TokenType::TRUE => return Some(self.parse_boolean()),
            token::TokenType::FALSE => return Some(self.parse_boolean()),
            token::TokenType::LPAREN => return self.parse_grouped_expression(),
            token::TokenType::IF => return self.parse_if_expression(),
            token::TokenType::FUNCTION => return self.parse_function_literal(),
            _ => return None,
        }
    }

    fn parse_infix_expression_fns(
        &mut self,
        t: token::TokenType,
        left_exp: Box<ast::Expression>,
    ) -> Option<ast::Expression> {
        match t {
            token::TokenType::PLUS => return self.parse_infix_expression(left_exp),
            token::TokenType::MINUS => return self.parse_infix_expression(left_exp),
            token::TokenType::SLASH => return self.parse_infix_expression(left_exp),
            token::TokenType::ASTERISK => return self.parse_infix_expression(left_exp),
            token::TokenType::EQ => return self.parse_infix_expression(left_exp),
            token::TokenType::NOTEQ => return self.parse_infix_expression(left_exp),
            token::TokenType::LT => return self.parse_infix_expression(left_exp),
            token::TokenType::GT => return self.parse_infix_expression(left_exp),
            token::TokenType::LPAREN => return self.parse_call_expression(left_exp),
            _ => return None,
        }
    }

    fn parse_prefix_expression(&mut self) -> Option<ast::Expression> {
        let expression_operator = (&self.cur_token.literal).to_string();

        self.next_token();

        if let Some(right) = self.parse_expression(Precedence::PREFIX) {
            return Some(ast::Expression::PrefixExpression {
                operator: expression_operator,
                right: Box::new(right),
            });
        } else {
            return None;
        }
    }

    fn parse_infix_expression(&mut self, left: Box<ast::Expression>) -> Option<ast::Expression> {
        let operator = (&self.cur_token.literal).to_string();

        let precedence = self.cur_precedence();
        self.next_token();
        if let Some(right) = self.parse_expression(precedence) {
            return Some(ast::Expression::InfixExpression {
                left,
                operator: operator,
                right: Box::new(right),
            });
        } else {
            return None;
        }
    }

    fn parse_call_expression(&mut self, function: Box<ast::Expression>) -> Option<ast::Expression> {
        match self.parse_call_arguments() {
            Some(arguments) => {
                return Some(ast::Expression::CallExpression {
                    function,
                    arguments,
                })
            }
            None => return None,
        }
    }

    fn parse_call_arguments(&mut self) -> Option<Vec<ast::Expression>> {
        let mut args = Vec::new();

        if self.peek_token_is(&token::TokenType::RPAREN) {
            self.next_token();
            return Some(args);
        }

        self.next_token();
        match self.parse_expression(Precedence::LOWEST) {
            Some(expression) => {
                args.push(expression);
                while self.peek_token_is(&token::TokenType::COMMA) {
                    self.next_token();
                    self.next_token();
                    match self.parse_expression(Precedence::LOWEST) {
                        Some(exp) => args.push(exp),
                        None => return None,
                    }
                }

                if !self.expect_peek(token::TokenType::RPAREN) {
                    return None;
                }

                return Some(args);
            }
            None => return None,
        };
    }

    fn parse_if_expression(&mut self) -> Option<ast::Expression> {
        if !self.expect_peek(token::TokenType::LPAREN) {
            return None;
        }

        self.next_token();
        match self.parse_expression(Precedence::LOWEST) {
            Some(condition) => {
                if !self.expect_peek(token::TokenType::RPAREN) {
                    return None;
                }
                if !self.expect_peek(token::TokenType::LBRACE) {
                    return None;
                }

                match self.parse_block_statement() {
                    Some(consequence) => {
                        if self.peek_token_is(&token::TokenType::ELSE) {
                            self.next_token();

                            if !self.expect_peek(token::TokenType::LBRACE) {
                                return None;
                            }

                            match self.parse_block_statement() {
                                Some(alternative) => {
                                    let expression = ast::Expression::IfExpression {
                                        condition: Box::new(condition),
                                        consequence: Box::new(consequence),
                                        alternative: Some(Box::new(alternative)),
                                    };
                                    return Some(expression);
                                }
                                None => return None,
                            }
                        }

                        let expression = ast::Expression::IfExpression {
                            condition: Box::new(condition),
                            consequence: Box::new(consequence),
                            alternative: None,
                        };
                        return Some(expression);
                    }
                    None => return None,
                }
            }
            None => return None,
        }
    }
    fn parse_function_literal(&mut self) -> Option<ast::Expression> {
        if !self.expect_peek(token::TokenType::LPAREN) {
            return None;
        }

        match self.parse_function_parameters() {
            Some(parameters) => {
                if !self.expect_peek(token::TokenType::LBRACE) {
                    return None;
                }
                match self.parse_block_statement() {
                    Some(body) => {
                        return Some(ast::Expression::FunctionLiteral {
                            parameters,
                            body: Box::new(body),
                        })
                    }
                    None => return None,
                }
            }
            None => return None,
        }
    }

    fn parse_identifier(&self) -> ast::Expression {
        return ast::Expression::Identifier {
            value: (&self.cur_token.literal).to_string(),
        };
    }

    fn parse_integer_literal(&mut self) -> Option<ast::Expression> {
        if let Ok(value) = self.cur_token.literal.parse::<i64>() {
            return Some(ast::Expression::IntegerLiteral { value });
        } else {
            self.errors.push(format!(
                "could not parse {} as integer",
                self.cur_token.literal
            ));
            return None;
        }
    }

    fn parse_boolean(&mut self) -> ast::Expression {
        return ast::Expression::Boolean {
            value: self.cur_token_is(&token::TokenType::TRUE),
        };
    }

    fn parse_grouped_expression(&mut self) -> Option<ast::Expression> {
        self.next_token();

        let exp = self.parse_expression(Precedence::LOWEST);

        if !self.expect_peek(token::TokenType::RPAREN) {
            return None;
        }

        return exp;
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<ast::Expression>> {
        let mut parameters = Vec::new();
        if self.peek_token_is(&token::TokenType::RPAREN) {
            self.next_token();
            return Some(parameters);
        }

        self.next_token();

        parameters.push(ast::Expression::Identifier {
            value: (&self.cur_token.literal).to_string(),
        });
        while self.peek_token_is(&token::TokenType::COMMA) {
            self.next_token();
            self.next_token();
            parameters.push(ast::Expression::Identifier {
                value: (&self.cur_token.literal).to_string(),
            });
        }

        if !self.expect_peek(token::TokenType::RPAREN) {
            return None;
        }

        return Some(parameters);
    }

    fn cur_token_is(&self, t: &token::TokenType) -> bool {
        return self.cur_token.token_type == *t;
    }
    fn peek_token_is(&self, t: &token::TokenType) -> bool {
        return self.peek_token.token_type == *t;
    }
    fn expect_peek(&mut self, t: token::TokenType) -> bool {
        if self.peek_token_is(&t) {
            self.next_token();
            return true;
        } else {
            self.peek_error(t);
            return false;
        }
    }

    fn no_prefix_parse_fn_error(&mut self) {
        self.errors.push(format!(
            "no prefix parse function for {:?} found",
            self.cur_token.token_type
        ));
    }

    fn peek_precedence(&mut self) -> Precedence {
        return token_type_to_precedence(&self.peek_token.token_type);
    }

    fn cur_precedence(&mut self) -> Precedence {
        return token_type_to_precedence(&self.cur_token.token_type);
    }
}

#[cfg(test)]
mod lexer_tests {
    use super::*;
    #[test]
    fn test_let_statements() {
        let input = "
let x = 5;
let y = 10;
let foobar = 838383;
        "
        .to_string();

        let l = lexer::Lexer::new(input);
        let mut p = Parser::new(l);

        counted_array!(
            let tests: [&str; _] = [
                "x",
                "y",
                "foobar"
            ]
        );

        let program = p.parse_program();
        check_parser_errors(p);

        assert_eq!(program.statements.len(), 3);
        for (i, t) in tests.iter().enumerate() {
            let stmt = &program.statements[i];
            test_let_statement(stmt, t);
        }
    }

    #[test]
    fn test_return_statements() {
        let input = "
return 5;
return 10;
return 993322;
        "
        .to_string();

        let l = lexer::Lexer::new(input);
        let mut p = Parser::new(l);

        let program = p.parse_program();
        check_parser_errors(p);

        assert_eq!(program.statements.len(), 3);
        assert_eq!(
            format!("{}", program.statements[0]),
            format!("return {};", 5)
        );
        assert_eq!(
            format!("{}", program.statements[1]),
            format!("return {};", 10)
        );
        assert_eq!(
            format!("{}", program.statements[2]),
            format!("return {};", 993322)
        );
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;".to_string();

        let l = lexer::Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(p);

        assert_eq!(program.statements.len(), 1);
        if let ast::Statement::ExpressionStatement { expression } = &program.statements[0] {
            if let ast::Expression::Identifier { value } = expression {
                assert_eq!(value, "foobar");
            } else {
                panic!(
                    "program.Statement[0] is not ast.Identifier. got={}",
                    expression
                );
            }
        } else {
            panic!(
                "program.Statement[0] is not ast.ExpressionStatement. got={}",
                program.statements[0]
            );
        }
    }
    #[test]
    fn test_integer_literal_expression() {
        let input = "5;".to_string();

        let l = lexer::Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(p);

        assert_eq!(program.statements.len(), 1);
        if let ast::Statement::ExpressionStatement { expression } = &program.statements[0] {
            if let ast::Expression::IntegerLiteral { value } = expression {
                assert_eq!(*value, 5 as i64);
            } else {
                panic!(
                    "program.Statement[0] is not ast.IntegerLiteral. got={}",
                    expression
                );
            }
        } else {
            panic!(
                "program.Statement[0] is not ast.ExpressionStatement. got={}",
                program.statements[0]
            );
        }
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        counted_array!(
            let prefix_tests: [(&str, &str, i64); _] = [
                ("!5;", "!", 5),
                ("-15;", "-", 15),
            ]
        );

        for t in prefix_tests {
            let l = lexer::Lexer::new(t.0.to_string());
            let mut p = Parser::new(l);
            let program = p.parse_program();

            check_parser_errors(p);
            assert_eq!(program.statements.len(), 1);

            if let ast::Statement::ExpressionStatement { expression } = &program.statements[0] {
                if let ast::Expression::PrefixExpression { operator, right } = expression {
                    assert_eq!(operator, t.1);
                    if !test_integer_literal((**right).clone(), t.2) {
                        return;
                    }
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        }
    }

    #[test]
    fn test_parsing_infix_expressions() {
        counted_array!(
            let infix_tests: [(&str, i64, &str, i64); _] = [
                ("5 + 5;", 5, "+", 5),
                ("5 - 5;", 5, "-", 5),
                ("5 * 5;", 5, "*", 5),
                ("5 / 5;", 5, "/", 5),
                ("5 > 5;", 5, ">", 5),
                ("5 < 5;", 5, "<", 5),
                ("5 == 5;", 5, "==", 5),
                ("5 != 5;", 5, "!=", 5),
            ]
        );
        for t in infix_tests {
            let l = lexer::Lexer::new(t.0.to_string());
            let mut p = Parser::new(l);
            let program = p.parse_program();

            check_parser_errors(p);
            assert_eq!(program.statements.len(), 1);

            if let ast::Statement::ExpressionStatement { expression } = &program.statements[0] {
                if let ast::Expression::InfixExpression {
                    left,
                    operator,
                    right,
                } = expression
                {
                    assert_eq!(operator, t.2);
                    let left_exp: ast::Expression = (**left).clone();
                    if !test_integer_literal(left_exp, t.1) {
                        return;
                    }
                    let right_exp: ast::Expression = (**right).clone();
                    if !test_integer_literal(right_exp, t.3) {
                        return;
                    }
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        counted_array!(
            let tests: [(&str, &str); _] = [
                ("a + b / c - d", "((a + (b / c)) - d)\n"),
                ("-a + b", "((-a) + b)\n"),
                ("!-a", "(!(-a))\n"),
                ("a + b + c", "((a + b) + c)\n"),
                ("a + b - c", "((a + b) - c)\n"),
                ("a * b * c", "((a * b) * c)\n"),
                ("a * b / c", "((a * b) / c)\n"),
                ("a + b / c", "(a + (b / c))\n"),
                ("a + b * c + d / e - f", "(((a + (b * c)) + (d / e)) - f)\n"),
                ("3 + 4; -5 * 5", "(3 + 4)\n((-5) * 5)\n"),
                ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4))\n"),
                ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4))\n"),
                ("3 + 4 * 5 == 3 * 1 + 4 * 5", "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))\n"),
                ("true", "true\n"),
                ("false", "false\n"),
                ("3 > 5 == false", "((3 > 5) == false)\n"),
                ("3 < 5 == true", "((3 < 5) == true)\n"),
                ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4)\n"),
                ("(5 + 5) * 2", "((5 + 5) * 2)\n"),
                ("2 / (5 + 5)", "(2 / (5 + 5))\n"),
                ("-(5 + 5)", "(-(5 + 5))\n"),
                ("!(true == true)", "(!(true == true))\n"),
                ("a + add(b * c) + d", "((a + add((b * c))) + d)\n"),
                ("add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))", "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))\n"),
                ("add(a + b + c * d / f + g)", "add((((a + b) + ((c * d) / f)) + g))\n"),
            ]
        );

        for t in tests {
            let l = lexer::Lexer::new(t.0.to_string());
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(p);

            let actual = format!("{}", program);
            assert_eq!(actual, t.1);
        }
    }

    #[test]
    fn test_boolean() {
        counted_array!(
            let tests: [(&str, &str); _] = [
                ("true;", "true\n"),
                ("false;", "false\n"),
                ("let foobar = true;", "let foobar = true;\n"),
                ("let barfoo = false", "let barfoo = false;\n"),
            ]
        );

        for t in tests {
            let l = lexer::Lexer::new(t.0.to_string());
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(p);

            let actual = format!("{}", program);
            assert_eq!(actual, t.1);
        }
    }

    #[test]
    fn test_if_expression() {
        counted_array!(
            let tests: [(&str, &str); _] = [
                ("if (x < y) { x }", "if ((x < y)) {\n\tx\n}\n"),
                ("if (x < y) { x } else { y }", "if ((x < y)) {\n\tx\n} else {\n\ty\n}\n"),
            ]
        );

        for t in tests {
            let l = lexer::Lexer::new(t.0.to_string());
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(p);

            let actual = format!("{}", program);
            assert_eq!(actual, t.1);
        }
    }

    #[test]
    fn test_function_parameter_parsing() {
        counted_array!(
            let tests: [(&str, &str); _] = [
                ("fn() {};", "fn () {\n}\n"),
                ("fn(x) {};", "fn (x) {\n}\n"),
                ("fn(x, y, z) {};", "fn (x, y, z) {\n}\n"),
            ]
        );

        for t in tests {
            let l = lexer::Lexer::new(t.0.to_string());
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(p);

            let actual = format!("{}", program);
            assert_eq!(actual, t.1);
        }
    }

    #[test]
    fn test_call_function() {
        counted_array!(
            let tests: [(&str, &str); _] = [
                ("add()", "add()\n"),
                ("sum(x);", "sum(x)\n"),
                ("get(x, y, z)", "get(x, y, z)\n"),
            ]
        );

        for t in tests {
            let l = lexer::Lexer::new(t.0.to_string());
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(p);

            let actual = format!("{}", program);
            assert_eq!(actual, t.1);
        }
    }

    fn test_let_statement(stmt: &ast::Statement, t: &str) {
        if let ast::Statement::LetStatement { name, value: _ } = stmt {
            if let ast::Expression::Identifier { value } = name {
                assert_eq!(value, t);
            } else {
                panic!("expression does not equal to identifier.");
            }
        } else {
            panic!("statement does not equal to letstatement.");
        }
    }

    fn test_integer_literal(il: ast::Expression, value: i64) -> bool {
        if let ast::Expression::IntegerLiteral { value: integ } = il {
            assert_eq!(integ, value);
            return true;
        } else {
            return false;
        }
    }

    fn check_parser_errors(p: Parser) {
        if p.errors.len() == 0 {
            return;
        }

        let mut error_messages = "".to_string();

        for msg in p.errors {
            error_messages += &msg;
        }
        panic!("{}", error_messages);
    }
}
