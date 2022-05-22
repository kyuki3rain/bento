use super::{ast, environment, object};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Evaluator {
    env: Rc<RefCell<environment::Environment>>,
}

impl Evaluator {
    pub fn new() -> Self {
        return Evaluator {
            env: Rc::new(RefCell::new(environment::Environment::new())),
        };
    }

    pub fn eval_program(&mut self, program: ast::Program) -> Option<object::Object> {
        let mut result = None;
        for stmt in program.statements {
            result = self.eval_statement(stmt);
            if let Some(r) = result {
                match r {
                    object::Object::Return(value) => return Some(*value),
                    object::Object::Error(value) => return Some(object::Object::Error(value)),
                    _ => result = Some(r),
                }
            }
        }
        return result;
    }
    fn eval_block_statement(&mut self, statements: Vec<ast::Statement>) -> Option<object::Object> {
        let mut result = None;
        for stmt in statements {
            result = self.eval_statement(stmt);
            if let Some(r) = result {
                match r {
                    object::Object::Return(value) => return Some(object::Object::Return(value)),
                    object::Object::Error(value) => return Some(object::Object::Error(value)),
                    _ => result = Some(r),
                }
            }
        }
        return result;
    }
    fn eval_statement(&mut self, stmt: ast::Statement) -> Option<object::Object> {
        match stmt {
            ast::Statement::LetStatement { name, value } => match self.eval_expression(value) {
                Some(val) => {
                    if !Evaluator::is_error(&val) {
                        self.env.borrow_mut().set(name.to_string(), &val);
                    }
                    return Some(val);
                }
                None => return None,
            },
            ast::Statement::ReturnStatement { return_value } => {
                match self.eval_expression(return_value) {
                    Some(value) => {
                        if Evaluator::is_error(&value) {
                            return Some(value);
                        }
                        return Some(object::Object::Return(Box::new(value)));
                    }
                    None => return None,
                }
            }
            ast::Statement::ExpressionStatement { expression } => {
                return self.eval_expression(expression)
            }
            ast::Statement::BlockStatement { statements } => {
                return self.eval_block_statement(statements)
            }
        }
    }
    fn eval_expression(&mut self, exp: ast::Expression) -> Option<object::Object> {
        match exp {
            ast::Expression::Identifier { value } => return self.eval_identifier(value),
            ast::Expression::IntegerLiteral { value } => {
                return Some(object::Object::Integer(value))
            }
            ast::Expression::StringLiteral { value } => return Some(object::Object::String(value)),
            ast::Expression::PrefixExpression { operator, right } => {
                match self.eval_expression(*right) {
                    Some(right_evaluated) => {
                        if Evaluator::is_error(&right_evaluated) {
                            return Some(right_evaluated);
                        }
                        return Evaluator::eval_prefix_expression(operator, right_evaluated);
                    }
                    None => return None,
                }
            }
            ast::Expression::InfixExpression {
                left,
                operator,
                right,
            } => match self.eval_expression(*right) {
                Some(right_evaluated) => {
                    if Evaluator::is_error(&right_evaluated) {
                        return Some(right_evaluated);
                    }
                    match self.eval_expression(*left) {
                        Some(left_evaluated) => {
                            if Evaluator::is_error(&left_evaluated) {
                                return Some(left_evaluated);
                            }
                            return self.eval_infix_expression(
                                operator,
                                left_evaluated,
                                right_evaluated,
                            );
                        }
                        None => return None,
                    }
                }
                None => return None,
            },
            ast::Expression::Boolean { value } => return Some(Evaluator::eval_boolean(value)),
            ast::Expression::IfExpression {
                condition,
                consequence,
                alternative,
            } => return self.eval_if_expression(condition, consequence, alternative),
            ast::Expression::FunctionLiteral { parameters, body } => {
                return Some(object::Object::Function {
                    parameters,
                    env: Rc::clone(&self.env),
                    body: *body,
                })
            }
            ast::Expression::CallExpression {
                function,
                arguments,
            } => {
                if let Some(func) = self.eval_expression(*function) {
                    let args = self.eval_expressions(arguments);
                    if args.len() == 1 && Evaluator::is_error(&args[0]) {
                        return Some(args[0].clone());
                    }
                    return self.apply_function(func, args);
                } else {
                    return None;
                }
            }
            ast::Expression::NeedNext => return None,
        }
    }
    fn apply_function(
        &mut self,
        func: object::Object,
        args: Vec<object::Object>,
    ) -> Option<object::Object> {
        match func {
            object::Object::Function {
                parameters,
                body,
                env,
            } => {
                let current_env = Rc::clone(&self.env);
                let mut extended_env =
                    environment::Environment::new_enclosed_environment(Rc::clone(&env));
                for (i, p) in parameters.iter().enumerate() {
                    match p {
                        ast::Expression::Identifier { value } => {
                            extended_env.set((&value).to_string(), &args[i])
                        }
                        _ => return None,
                    }
                }
                self.env = Rc::new(RefCell::new(extended_env));
                if let Some(evaluated) = self.eval_statement(body) {
                    match evaluated {
                        object::Object::Return(value) => return Some(*value),
                        _ => {
                            return Some(evaluated);
                        }
                    }
                }
                self.env = current_env;
                return None;
            }
            _ => None,
        }
    }
    fn eval_expressions(&mut self, exps: Vec<ast::Expression>) -> Vec<object::Object> {
        let mut result = Vec::new();
        for e in exps {
            if let Some(evaluated) = self.eval_expression(e) {
                if Evaluator::is_error(&evaluated) {
                    return vec![evaluated];
                }
                result.push(evaluated);
            }
        }
        return result;
    }
    fn eval_prefix_expression(operator: String, right: object::Object) -> Option<object::Object> {
        match &*operator {
            "!" => return Evaluator::eval_bang_operator_expression(right),
            "-" => return Evaluator::eval_minus_prefix_operator_expression(right),
            _ => {
                return Some(object::Object::Error(format!(
                    "unknown operator: {}{}",
                    operator, right
                )))
            }
        }
    }
    fn eval_infix_expression(
        &mut self,
        operator: String,
        left: object::Object,
        right: object::Object,
    ) -> Option<object::Object> {
        if format!("{}", left) != format!("{}", right) {
            return Some(object::Object::Error(format!(
                "type mismatch: {} {} {}",
                left, operator, right
            )));
        }
        match left {
            object::Object::Integer(left_value) => match right {
                object::Object::Integer(right_value) => {
                    return Evaluator::eval_integer_infix_expression(
                        operator,
                        left_value,
                        right_value,
                    )
                }
                _ => return None,
            },
            object::Object::String(left_value) => match right {
                object::Object::String(right_value) => {
                    return Evaluator::eval_string_infix_expression(
                        operator,
                        left_value,
                        right_value,
                    )
                }
                _ => return None,
            },
            object::Object::Boolean(left_value) => match right {
                object::Object::Boolean(right_value) => match &*operator {
                    "==" => return Some(Evaluator::eval_boolean(left_value == right_value)),
                    "!=" => return Some(Evaluator::eval_boolean(left_value != right_value)),
                    _ => {
                        return Some(object::Object::Error(format!(
                            "unknown operator: {} {} {}",
                            left, operator, right
                        )))
                    }
                },
                _ => return None,
            },
            _ => return None,
        }
    }
    fn eval_integer_infix_expression(
        operator: String,
        left_value: i64,
        right_value: i64,
    ) -> Option<object::Object> {
        match &*operator {
            "+" => return Some(object::Object::Integer(left_value + right_value)),
            "-" => return Some(object::Object::Integer(left_value - right_value)),
            "*" => return Some(object::Object::Integer(left_value * right_value)),
            "/" => return Some(object::Object::Integer(left_value / right_value)),
            "<" => return Some(Evaluator::eval_boolean(left_value < right_value)),
            ">" => return Some(Evaluator::eval_boolean(left_value > right_value)),
            "==" => return Some(Evaluator::eval_boolean(left_value == right_value)),
            "!=" => return Some(Evaluator::eval_boolean(left_value != right_value)),
            _ => {
                return Some(object::Object::Error(format!(
                    "unknown operator: {} {} {}",
                    left_value, operator, right_value
                )))
            }
        }
    }

    fn eval_string_infix_expression(
        operator: String,
        left_value: String,
        right_value: String,
    ) -> Option<object::Object> {
        match &*operator {
            "+" => {
                return Some(object::Object::String(format!(
                    "{}{}",
                    left_value, right_value
                )))
            }
            "==" => return Some(Evaluator::eval_boolean(left_value == right_value)),
            "!=" => return Some(Evaluator::eval_boolean(left_value != right_value)),
            _ => {
                return Some(object::Object::Error(format!(
                    "unknown operator: STRING {} STRING",
                    operator
                )))
            }
        }
    }

    fn eval_bang_operator_expression(right: object::Object) -> Option<object::Object> {
        match right {
            object::Object::Boolean(value) => return Some(Evaluator::eval_boolean(!value)),
            object::Object::Null => return Some(object::TRUE),
            _ => Some(object::FALSE),
        }
    }

    fn eval_boolean(value: bool) -> object::Object {
        if value {
            return object::TRUE;
        } else {
            return object::FALSE;
        }
    }
    fn eval_minus_prefix_operator_expression(right: object::Object) -> Option<object::Object> {
        match right {
            object::Object::Integer(value) => return Some(object::Object::Integer(-value)),
            _ => {
                return Some(object::Object::Error(format!(
                    "unknown operator: -{}",
                    right
                )))
            }
        }
    }
    fn eval_if_expression(
        &mut self,
        condition: Box<ast::Expression>,
        consequence: Box<ast::Statement>,
        alternative: Option<Box<ast::Statement>>,
    ) -> Option<object::Object> {
        if let Some(evaluated_condition) = self.eval_expression(*condition) {
            if Evaluator::is_error(&evaluated_condition) {
                return Some(evaluated_condition);
            }
            if Evaluator::is_truthy(evaluated_condition) {
                return self.eval_statement(*consequence);
            } else if let Some(alt) = alternative {
                return self.eval_statement(*alt);
            } else {
                return Some(object::NULL);
            }
        } else {
            return None;
        }
    }

    fn eval_identifier(&mut self, ident: String) -> Option<object::Object> {
        match self.env.borrow_mut().get((&ident).to_string()) {
            Some(value) => return Some(value),
            None => {
                return Some(object::Object::Error(format!(
                    "identifier not found: {}",
                    ident
                )))
            }
        }
    }
    fn is_truthy(obj: object::Object) -> bool {
        match obj {
            object::Object::Null => return false,
            object::Object::Boolean(value) => return value,
            _ => return true,
        }
    }
    fn is_error(obj: &object::Object) -> bool {
        match obj {
            object::Object::Error(_) => return true,
            _ => return false,
        }
    }
}

#[cfg(test)]
mod evaluator_tests {
    use super::super::{lexer, parser};
    use super::*;

    #[test]
    fn test_eval_integer_expression() {
        counted_array!(
            let tests: [(&str, i64); _] = [
                ("5", 5),
                ("10", 10),
                ("-5", -5),
                ("-10", -10),
                ("5 + 5 + 5 + 5 - 10", 10),
                ("2 * 2 * 2 * 2 * 2", 32),
                ("-50 + 100 + -50", 0),
                ("5 * 2 + 10", 20),
                ("5 + 2 * 10", 25),
                ("20 + 2 * 10", 40),
                ("50 / 2 * 2 + 10", 60),
                ("2 * (5 + 10)", 30),
                ("3 * 3 * 3 + 10", 37),
                ("3 * (3 * 3) + 10", 37),
                ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            test_integer_object(evaluated, t.1);
        }
    }

    #[test]
    fn test_eval_string_literal() {
        counted_array!(
            let tests: [(&str, &str); _] = [
                ("\"Hello World!\"", "Hello World!"),
                ("\"Hello\" + \" \" + \"World!\"", "Hello World!"),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            if let object::Object::String(value) = evaluated {
                assert_eq!(value, t.1.to_string());
            } else {
                panic!("not string");
            }
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        counted_array!(
            let tests: [(&str, bool); _] = [
                ("true", true),
                ("false", false),
                ("1 < 2", true),
                ("1 > 2", false),
                ("1 < 1", false),
                ("1 > 1", false),
                ("1 == 1", true),
                ("1 != 1", false),
                ("1 == 2", false),
                ("1 != 2", true),
                ("true == true", true),
                ("false == false", true),
                ("true == false", false),
                ("true != false", true),
                ("false != true", true),
                ("(1 < 2) == true", true),
                ("(1 < 2) == false", false),
                ("(1 > 2) == true", false),
                ("(1 > 2) == false", true),
                ("\"Hello\" == \"Hello\"", true),
                ("\"Hello\" == \"World\"", false),

            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            test_boolean_object(evaluated, t.1);
        }
    }

    #[test]
    fn test_bang_operator() {
        counted_array!(
            let tests: [(&str, bool); _] = [
                ("!true", false),
                ("!false", true),
                ("!5", false),
                ("!!true", true),
                ("!!false", false),
                ("!!5", true),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            test_boolean_object(evaluated, t.1);
        }
    }

    #[test]
    fn test_if_else_expression() {
        counted_array!(
            let tests: [(&str, Option<i64>); _] = [
                ("if (true) { 10 }", Some(10)),
                ("if (false) { 10 }", None),
                ("if (1) { 10 }", Some(10)),
                ("if (1 < 2) { 10 }", Some(10)),
                ("if (1 > 2) { 10 }", None),
                ("if (1 > 2) { 10 } else { 20 }", Some(20)),
                ("if (1 < 2) { 10 } else { 20 }", Some(10)),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            if let Some(integ) = t.1 {
                test_integer_object(evaluated, integ);
            } else {
                test_null_object(evaluated);
            }
        }
    }

    #[test]
    fn test_return_statements() {
        counted_array!(
            let tests: [(&str, i64); _] = [
                ("return 10;", 10),
                ("return 10; 9;", 10),
                ("return 2 * 5; 9;", 10),
                ("9; return 2 * 5; 9;", 10),
                ("if (10 > 1) {
                    if ( 10 > 1) {
                        return 10;
                    }
                    return 1;
                }", 10),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            test_integer_object(evaluated, t.1);
        }
    }

    #[test]
    fn test_error_handling() {
        counted_array!(
            let tests: [(&str, &str); _] = [
                ("5 + true;", "type mismatch: INTEGER + BOOLEAN"),
                ("5 + true; 5;", "type mismatch: INTEGER + BOOLEAN"),
                ("-true", "unknown operator: -BOOLEAN"),
                ("true + false;", "unknown operator: BOOLEAN + BOOLEAN"),
                ("5; true + false; 5", "unknown operator: BOOLEAN + BOOLEAN"),
                ("if(10 > 1) {true + false; }", "unknown operator: BOOLEAN + BOOLEAN"),
                ("if(10 > 1) {
                    if (-true) {
                        return true + false;
                    }
                    return 1
                }", "unknown operator: -BOOLEAN"),
                ("foobar", "identifier not found: foobar"),
                ("\"Hello\" - \"World\"", "unknown operator: STRING - STRING"),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            match evaluated {
                object::Object::Error(value) => {
                    assert_eq!(value, t.1);
                }
                _ => {
                    panic!("{}", evaluated);
                }
            }
        }
    }

    #[test]
    fn test_function_object() {
        counted_array!(
            let tests: [(&str, Vec<&str>, &str); _] = [
                ("fn(x) { x + 2; };", vec!["x"], "{\r\n\t(x + 2)\r\n}"),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            match evaluated {
                object::Object::Function {
                    parameters,
                    body,
                    env: _,
                } => {
                    assert_eq!(parameters.len(), t.1.len());
                    for (i, p) in parameters.iter().enumerate() {
                        assert_eq!(format!("{}", p), t.1[i]);
                    }
                    assert_eq!(format!("{}", body), t.2);
                }
                _ => {
                    panic!("{}", evaluated);
                }
            }
        }
    }
    #[test]
    fn test_function_application() {
        counted_array!(
            let tests: [(&str, i64); _] = [
                ("let identity = fn(x) { x; }; identity(5);", 5),
                ("let identity = fn(x) { return x; }; identity(5);", 5),
                ("let double = fn(x) { x * 2; }; double(5);", 10),
                ("let add = fn(x, y) { x + y; }; add(5, 5);", 10),
                ("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));", 20),
                ("fn(x){x;}(5)", 5),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            test_integer_object(evaluated, t.1);
        }
    }

    fn test_eval(input: String) -> object::Object {
        let mut evaluator = Evaluator::new();
        let l = lexer::Lexer::new(&input);
        let mut p = parser::Parser::new(l);
        let program = p.parse_program();

        match evaluator.eval_program(program) {
            Some(obj) => return obj,
            None => panic!(),
        }
    }

    fn test_integer_object(obj: object::Object, expected: i64) -> bool {
        match obj {
            object::Object::Integer(value) => {
                if value != expected {
                    panic!("{} is not match to {}", value, expected);
                }
                return true;
            }
            _ => panic!("{} is not integer object.", obj),
        }
    }

    fn test_boolean_object(obj: object::Object, expected: bool) -> bool {
        match obj {
            object::Object::Boolean(value) => {
                if value != expected {
                    panic!("{} is not match to {}", value, expected);
                }
                return true;
            }
            _ => panic!("{} is not integer object.", obj),
        }
    }

    fn test_null_object(obj: object::Object) -> bool {
        if let object::Object::Null = obj {
            return false;
        }
        return true;
    }
}
