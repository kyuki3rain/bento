use super::{ast, builtin, environment, object};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(PartialEq)]
pub struct Evaluator {
    env: Rc<RefCell<environment::Environment>>,
    builtin: HashMap<String, Rc<object::Object>>,
}

impl Evaluator {
    pub fn new() -> Self {
        return Evaluator {
            env: Rc::new(RefCell::new(environment::Environment::new())),
            builtin: builtin::new_builtins(),
        };
    }

    pub fn eval_program(&mut self, program: ast::Program) -> Option<Rc<object::Object>> {
        match self.eval_block_statement(program.statements) {
            Some(r) => match &*r {
                object::Object::Return(value) => return Some(Rc::clone(value)),
                _ => return Some(r),
            },
            None => return None,
        }
    }

    fn eval_block_statement(
        &mut self,
        statements: Vec<ast::Statement>,
    ) -> Option<Rc<object::Object>> {
        let mut result = None;
        for stmt in statements {
            result = self.eval_statement(stmt);
            if let Some(r) = result {
                match &*r {
                    object::Object::Return(_) => return Some(Rc::clone(&r)),
                    object::Object::Exit => return Some(Rc::new(object::EXIT)),
                    object::Object::Error(_) => return Some(Rc::clone(&r)),
                    _ => result = Some(r),
                }
            }
        }
        return result;
    }
    fn eval_statement(&mut self, stmt: ast::Statement) -> Option<Rc<object::Object>> {
        match stmt {
            ast::Statement::LetStatement { name, value } => match self.eval_expression(value) {
                Some(val) => {
                    if !Evaluator::is_error(&val) {
                        self.env.borrow_mut().set(name.to_string(), Rc::clone(&val));
                    }
                    return Some(Rc::clone(&val));
                }
                None => return None,
            },
            ast::Statement::ReturnStatement { return_value } => {
                match self.eval_expression(return_value) {
                    Some(value) => {
                        if Evaluator::is_error(&value) {
                            return Some(value);
                        }
                        return Some(Rc::new(object::Object::Return(Rc::clone(&value))));
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
    fn eval_expression(&mut self, exp: ast::Expression) -> Option<Rc<object::Object>> {
        match exp {
            ast::Expression::Identifier { value } => return self.eval_identifier(value),
            ast::Expression::IntegerLiteral { value } => {
                return Some(Rc::new(object::Object::Integer(value)))
            }
            ast::Expression::StringLiteral { value } => {
                return Some(Rc::new(object::Object::String(value)))
            }
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
                                operator.to_string(),
                                left_evaluated,
                                right_evaluated,
                            );
                        }
                        None => return None,
                    }
                }
                None => return None,
            },
            ast::Expression::AssignExpression { left, right } => {
                match self.eval_expression(*right) {
                    Some(right_evaluated) => {
                        if Evaluator::is_error(&right_evaluated) {
                            return Some(right_evaluated);
                        }
                        match *left {
                            ast::Expression::Identifier { value } => {
                                if !self.env.borrow_mut().contains_key(&value) {
                                    return Some(object::Object::new_error(format!(
                                        "{} is not defined before.",
                                        &value
                                    )));
                                }
                                self.env
                                    .borrow_mut()
                                    .set(value, Rc::clone(&right_evaluated));
                                return Some(right_evaluated);
                            }
                            _ => return None,
                        }
                    }
                    None => return None,
                }
            }
            ast::Expression::Boolean { value } => return Some(Evaluator::eval_boolean(value)),
            ast::Expression::ArrayLiteral { elements } => {
                let elms = self.eval_expressions(elements);
                if elms.len() == 1 && Evaluator::is_error(&elms[0]) {
                    return Some(Rc::clone(&elms[0]));
                }
                return Some(Rc::new(object::Object::Array(elms)));
            }
            ast::Expression::IndexExpression { left, index } => {
                let left = self.eval_expression(*left)?;
                let index = self.eval_expression(*index)?;

                if Evaluator::is_error(&left) {
                    return Some(left);
                }
                if Evaluator::is_error(&index) {
                    return Some(index);
                }

                return Evaluator::eval_index_expression(left, index);
            }
            ast::Expression::IfExpression {
                condition,
                consequence,
                alternative,
            } => return self.eval_if_expression(condition, consequence, alternative),
            ast::Expression::WhileExpression {
                condition,
                consequence,
            } => return self.eval_while_expression(condition, consequence),
            ast::Expression::FunctionLiteral { parameters, body } => {
                return Some(Rc::new(object::Object::Function {
                    parameters,
                    env: Rc::clone(&self.env),
                    body: *body,
                }))
            }
            ast::Expression::CallExpression {
                function,
                arguments,
            } => {
                if let Some(func) = self.eval_expression(*function) {
                    if Evaluator::is_error(&func) {
                        return Some(func);
                    }
                    let args = self.eval_expressions(arguments);
                    if args.len() == 1 && Evaluator::is_error(&args[0]) {
                        return Some(Rc::clone(&args[0]));
                    }
                    return self.apply_function(func, args);
                } else {
                    return None;
                }
            }
            ast::Expression::HashLiteral { pairs } => {
                let mut hash = HashMap::new();

                for (key_expr, value_expr) in pairs {
                    if let Some(key) = self.eval_expression(key_expr) {
                        if Self::is_error(&key) {
                            return Some(key);
                        }
                        if let Some(value) = self.eval_expression(value_expr) {
                            if Evaluator::is_error(&value) {
                                return Some(value);
                            }
                            hash.insert(key, value);
                        }
                    } else {
                        return None;
                    }
                }

                Some(Rc::new(object::Object::Hash(hash)))
            }
            ast::Expression::NeedNext => return None,
        }
    }

    fn eval_index_expression(
        left: Rc<object::Object>,
        index: Rc<object::Object>,
    ) -> Option<Rc<object::Object>> {
        if let object::Object::Array(elements) = &*left {
            if let object::Object::Integer(i) = *index {
                return Evaluator::eval_array_index_expression(elements, i);
            }
        } else if let object::Object::Hash(hash) = &*left {
            if let Some(obj) = hash.get(&index) {
                return Some(Rc::clone(obj));
            }
        }

        return Some(object::Object::new_error(format!("")));
    }

    fn eval_array_index_expression(
        elements: &Vec<Rc<object::Object>>,
        i: i64,
    ) -> Option<Rc<object::Object>> {
        if i < 0 || i > elements.len() as i64 - 1 {
            return Some(object::Object::new_error(format!(
                "list index out of range"
            )));
        }
        return Some(Rc::clone(&elements[i as usize]));
    }

    fn apply_function(
        &mut self,
        func: Rc<object::Object>,
        args: Vec<Rc<object::Object>>,
    ) -> Option<Rc<object::Object>> {
        match &*func {
            object::Object::Function {
                parameters,
                body,
                env,
            } => {
                let current_env = Rc::clone(&self.env);
                let mut extended_env =
                    environment::Environment::new_enclosed_environment(Rc::clone(&env));
                if args.len() != parameters.len() {
                    return Some(object::Object::new_error(format!(
                        "wrong number argument. got={}, expected={}",
                        args.len(),
                        parameters.len()
                    )));
                }
                for (i, p) in parameters.iter().enumerate() {
                    match p {
                        ast::Expression::Identifier { value } => {
                            extended_env.set((&value).to_string(), Rc::clone(&args[i]))
                        }
                        _ => return None,
                    }
                }
                self.env = Rc::new(RefCell::new(extended_env));
                if let Some(evaluated) = self.eval_statement(body.clone()) {
                    match &*evaluated {
                        object::Object::Return(value) => return Some(Rc::clone(value)),
                        _ => {
                            return Some(evaluated);
                        }
                    }
                }
                self.env = current_env;
                return None;
            }
            object::Object::Builtin(object::BuiltinFunc(_, function)) => Some(function(args, self)),
            _ => None,
        }
    }
    fn eval_expressions(&mut self, exps: Vec<ast::Expression>) -> Vec<Rc<object::Object>> {
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
    fn eval_prefix_expression(
        operator: String,
        right: Rc<object::Object>,
    ) -> Option<Rc<object::Object>> {
        match &*operator {
            "!" => return Evaluator::eval_bang_operator_expression(right),
            "-" => return Evaluator::eval_minus_prefix_operator_expression(right),
            _ => {
                return Some(object::Object::new_error(format!(
                    "unknown operator: {}{}",
                    operator, right
                )))
            }
        }
    }
    fn eval_infix_expression(
        &mut self,
        operator: String,
        left: Rc<object::Object>,
        right: Rc<object::Object>,
    ) -> Option<Rc<object::Object>> {
        let err =
            object::Object::new_error(format!("type mismatch: {} {} {}", &left, operator, &right));
        match &*left {
            object::Object::Integer(left_value) => match *right {
                object::Object::Integer(right_value) => {
                    return Evaluator::eval_integer_infix_expression(
                        operator,
                        *left_value,
                        right_value,
                    )
                }
                object::Object::Float(right_value) => {
                    return Evaluator::eval_float_infix_expression(
                        operator,
                        *left_value as f64,
                        right_value,
                    )
                }
                _ => return Some(err),
            },
            object::Object::Float(left_value) => match *right {
                object::Object::Integer(right_value) => {
                    return Evaluator::eval_float_infix_expression(
                        operator,
                        *left_value,
                        right_value as f64,
                    )
                }
                object::Object::Float(right_value) => {
                    return Evaluator::eval_float_infix_expression(
                        operator,
                        *left_value as f64,
                        right_value,
                    )
                }
                _ => return Some(err),
            },
            object::Object::String(left_value) => match &*right {
                object::Object::String(right_value) => {
                    return Evaluator::eval_string_infix_expression(
                        operator,
                        left_value.clone(),
                        right_value.clone(),
                    )
                }
                _ => return Some(err),
            },
            object::Object::Boolean(left_value) => match *right {
                object::Object::Boolean(right_value) => match &*operator {
                    "==" => return Some(Evaluator::eval_boolean(*left_value == right_value)),
                    "!=" => return Some(Evaluator::eval_boolean(*left_value != right_value)),
                    _ => {
                        return Some(object::Object::new_error(format!(
                            "unknown operator: {} {} {}",
                            left, operator, right
                        )))
                    }
                },
                _ => return Some(err),
            },
            _ => {
                return Some(object::Object::new_error(format!(
                    "type mismatch: {} {} {}",
                    &left, operator, &right
                )))
            }
        }
    }

    fn eval_float(left_value: i64, right_value: i64) -> Rc<object::Object> {
        let mut under_dot = right_value as f64;
        while under_dot >= 1.0 {
            under_dot = under_dot / 10.0;
        }

        return Rc::new(object::Object::Float(left_value as f64 + under_dot));
    }

    fn eval_integer_infix_expression(
        operator: String,
        left_value: i64,
        right_value: i64,
    ) -> Option<Rc<object::Object>> {
        match &*operator {
            "+" => return Some(Rc::new(object::Object::Integer(left_value + right_value))),
            "-" => return Some(Rc::new(object::Object::Integer(left_value - right_value))),
            "*" => return Some(Rc::new(object::Object::Integer(left_value * right_value))),
            "/" => return Some(Rc::new(object::Object::Integer(left_value / right_value))),
            "." => return Some(Evaluator::eval_float(left_value, right_value)),
            "<" => return Some(Evaluator::eval_boolean(left_value < right_value)),
            ">" => return Some(Evaluator::eval_boolean(left_value > right_value)),
            "==" => return Some(Evaluator::eval_boolean(left_value == right_value)),
            "!=" => return Some(Evaluator::eval_boolean(left_value != right_value)),
            _ => {
                return Some(object::Object::new_error(format!(
                    "unknown operator: {} {} {}",
                    left_value, operator, right_value
                )))
            }
        }
    }

    fn eval_float_infix_expression(
        operator: String,
        left_value: f64,
        right_value: f64,
    ) -> Option<Rc<object::Object>> {
        match &*operator {
            "+" => return Some(Rc::new(object::Object::Float(left_value + right_value))),
            "-" => return Some(Rc::new(object::Object::Float(left_value - right_value))),
            "*" => return Some(Rc::new(object::Object::Float(left_value * right_value))),
            "/" => return Some(Rc::new(object::Object::Float(left_value / right_value))),
            "<" => return Some(Evaluator::eval_boolean(left_value < right_value)),
            ">" => return Some(Evaluator::eval_boolean(left_value > right_value)),
            "==" => return Some(Evaluator::eval_boolean(left_value == right_value)),
            "!=" => return Some(Evaluator::eval_boolean(left_value != right_value)),
            _ => {
                return Some(object::Object::new_error(format!(
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
    ) -> Option<Rc<object::Object>> {
        match &*operator {
            "+" => {
                return Some(Rc::new(object::Object::String(format!(
                    "{}{}",
                    left_value, right_value
                ))))
            }
            "==" => return Some(Evaluator::eval_boolean(left_value == right_value)),
            "!=" => return Some(Evaluator::eval_boolean(left_value != right_value)),
            _ => {
                return Some(object::Object::new_error(format!(
                    "unknown operator: STRING {} STRING",
                    operator
                )))
            }
        }
    }

    fn eval_bang_operator_expression(right: Rc<object::Object>) -> Option<Rc<object::Object>> {
        match *right {
            object::Object::Boolean(value) => return Some(Evaluator::eval_boolean(!value)),
            object::Object::Null => return Some(Rc::new(object::TRUE)),
            _ => Some(Rc::new(object::FALSE)),
        }
    }

    fn eval_boolean(value: bool) -> Rc<object::Object> {
        if value {
            return Rc::new(object::TRUE);
        } else {
            return Rc::new(object::FALSE);
        }
    }
    fn eval_minus_prefix_operator_expression(
        right: Rc<object::Object>,
    ) -> Option<Rc<object::Object>> {
        match *right {
            object::Object::Integer(value) => {
                return Some(Rc::new(object::Object::Integer(-value)))
            }
            _ => {
                return Some(object::Object::new_error(format!(
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
    ) -> Option<Rc<object::Object>> {
        if let Some(evaluated_condition) = self.eval_expression(*condition) {
            if Evaluator::is_error(&evaluated_condition) {
                return Some(evaluated_condition);
            }
            if Evaluator::is_truthy(evaluated_condition) {
                return self.eval_statement(*consequence);
            } else if let Some(alt) = alternative {
                return self.eval_statement(*alt);
            } else {
                return Some(Rc::new(object::NULL));
            }
        } else {
            return None;
        }
    }

    fn eval_while_expression(
        &mut self,
        condition: Box<ast::Expression>,
        consequence: Box<ast::Statement>,
    ) -> Option<Rc<object::Object>> {
        let mut object = Rc::new(object::NULL);
        loop {
            if let Some(evaluated_condition) = self.eval_expression(*condition.clone()) {
                if Evaluator::is_error(&evaluated_condition) {
                    return Some(evaluated_condition);
                }
                if !Evaluator::is_truthy(evaluated_condition) {
                    break;
                }

                object = match self.eval_statement(*consequence.clone()) {
                    Some(obj) => match &*obj {
                        object::Object::Return(_) => return Some(Rc::clone(&obj)),
                        _ => obj,
                    },
                    _ => {
                        return None;
                    }
                };

                if Evaluator::is_error(&object) {
                    return Some(object);
                }
            } else {
                return None;
            }
        }

        return Some(object);
    }

    fn eval_identifier(&mut self, ident: String) -> Option<Rc<object::Object>> {
        if let Some(value) = self.env.borrow_mut().get((&ident).to_string()) {
            return Some(value);
        }
        if let Some(value) = self.builtin.get(&ident) {
            return Some(Rc::clone(value));
        }
        return Some(object::Object::new_error(format!(
            "identifier not found: {}",
            ident
        )));
    }
    fn is_truthy(obj: Rc<object::Object>) -> bool {
        match *obj {
            object::Object::Null => return false,
            object::Object::Boolean(value) => return value,
            _ => return true,
        }
    }
    fn is_error(obj: &Rc<object::Object>) -> bool {
        match **obj {
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
            test_integer_object(&evaluated, t.1);
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
            if let object::Object::String(value) = &*evaluated {
                assert_eq!(*value, t.1.to_string());
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
            test_boolean_object(&evaluated, t.1);
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
            test_boolean_object(&evaluated, t.1);
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
                ("if (1 < 2) { 10 } else { 20 }", Some(10)),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            if let Some(integ) = t.1 {
                test_integer_object(&evaluated, integ);
            } else {
                test_null_object(&*evaluated);
            }
        }
    }

    #[test]
    fn test_while_expression() {
        counted_array!(
            let tests: [(&str, Option<i64>); _] = [
                ("let i = 0; while (i < 10) { let i = i + 1; }", Some(10)),
                ("let i = 0; while (true) { let i = i + 1; if (i == 10) { return i; } }", Some(10)),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            if let Some(integ) = t.1 {
                test_integer_object(&evaluated, integ);
            } else {
                test_null_object(&*evaluated);
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
            test_integer_object(&evaluated, t.1);
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
                ("len(1)", "argument to `len` not supported, got INTEGER"),
                ("len(\"one\", \"two\")",  "wrong number of arguments. got=2, want=1"),
                ("[1, 2, 3][3]", "list index out of range"),
                ("[1, 2, 3][-1]", "list index out of range"),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            match &*evaluated {
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
            match &*evaluated {
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
            test_integer_object(&evaluated, t.1);
        }
    }

    #[test]
    fn test_array_literals() {
        let input = "[1, 2 * 2, 3 + 3]".to_string();

        let evaluated = test_eval(input);
        if let object::Object::Array(elements) = &*evaluated {
            if elements.len() != 3 {
                panic!("array has wrong num of elements. got={}", elements.len());
            }

            test_integer_object(&*elements[0], 1);
            test_integer_object(&*elements[1], 4);
            test_integer_object(&*elements[2], 6);
        }
    }

    #[test]
    fn test_builtin_functions() {
        counted_array!(
            let tests: [(&str, i64); _] = [
                ("len(\"\")", 0),
                ("len(\"four\")", 4),
                ("len(\"hello world\")", 11),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            test_integer_object(&evaluated, t.1);
        }
    }

    #[test]
    fn test_array_index_expressions() {
        counted_array!(
            let tests: [(&str, i64); _] = [
                ("[1,2,3][0]", 1),
                ("[1,2,3][1]", 2),
                ("[1,2,3][2]", 3),
                ("let i = 0; [1][i];", 1),
                ("[1, 2, 3][1 + 1]", 3),
                ("let myArray = [1, 2, 3]; myArray[2];", 3),
                ("let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];", 6),
                ("let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i];", 2),
            ]
        );

        for t in tests {
            let evaluated = test_eval(t.0.to_string());
            test_integer_object(&evaluated, t.1);
        }
    }

    #[test]
    fn test_hash_literals() {
        let input = "let two = \"two\";
        {
            \"one\": 10 - 9,
            two: 1 + 1,
            \"thr\" + \"ee\": 6 / 2,
            4: 4,
            true: 5,
            false: 6,
        }";

        let keys = vec![
            object::Object::String(String::from("one")),
            object::Object::String(String::from("two")),
            object::Object::String(String::from("three")),
            object::Object::Integer(4),
            object::Object::Boolean(true),
            object::Object::Boolean(false),
        ];

        let values = vec![
            object::Object::Integer(1),
            object::Object::Integer(2),
            object::Object::Integer(3),
            object::Object::Integer(4),
            object::Object::Integer(5),
            object::Object::Integer(6),
        ];

        let evaluated = test_eval(input.to_string());
        if let object::Object::Hash(hash) = &*evaluated {
            for (i, key) in keys.iter().enumerate() {
                if let Some(value) = hash.get(&Rc::new(key.clone())) {
                    if **value != values[i] {
                        panic!();
                    }
                } else {
                    panic!();
                }
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_builtin_function() {
        let input = "
        let i = 2;
        let a = [1, 1];
        while(i < 30) {
            let a = push(a, a[i - 2] + a[i - 1]);
            let i = i + 1;
        }
        ";

        let mut evaluator = Evaluator::new();
        test_eval_with_evaluator(input.to_string(), &mut evaluator);
        let evaluated = test_eval_with_evaluator("len(a)".to_string(), &mut evaluator);
        test_integer_object(&evaluated, 30);
        let evaluated = test_eval_with_evaluator("first(a)".to_string(), &mut evaluator);
        test_integer_object(&evaluated, 1);
        let evaluated = test_eval_with_evaluator("last(a)".to_string(), &mut evaluator);
        test_integer_object(&evaluated, 832040);
    }

    fn test_eval_with_evaluator(input: String, evaluator: &mut Evaluator) -> Rc<object::Object> {
        let l = lexer::Lexer::new(&input);
        let mut p = parser::Parser::new(l);
        let program = p.parse_program();

        if p.errors.len() != 0 {
            let mut s = "".to_string();
            for err in p.errors {
                s += &format!("\t{}\r\n", err);
            }
            panic!("parser errors:\r\n{}", s);
        }

        match evaluator.eval_program(program) {
            Some(obj) => return Rc::clone(&obj),
            None => panic!(),
        }
    }

    fn test_eval(input: String) -> Rc<object::Object> {
        let mut evaluator = Evaluator::new();
        let l = lexer::Lexer::new(&input);
        let mut p = parser::Parser::new(l);
        let program = p.parse_program();

        if p.errors.len() != 0 {
            let mut s = "".to_string();
            for err in p.errors {
                s += &format!("\t{}\r\n", err);
            }
            panic!("parser errors:\r\n{}", s);
        }

        match evaluator.eval_program(program) {
            Some(obj) => return Rc::clone(&obj),
            None => panic!(),
        }
    }

    fn test_integer_object(obj: &object::Object, expected: i64) -> bool {
        match obj {
            object::Object::Integer(value) => {
                if *value != expected {
                    panic!("{} is not match to {}", value, expected);
                }
                return true;
            }
            _ => panic!("{} is not integer object.", obj),
        }
    }

    fn test_boolean_object(obj: &object::Object, expected: bool) -> bool {
        match obj {
            object::Object::Boolean(value) => {
                if *value != expected {
                    panic!("{} is not match to {}", value, expected);
                }
                return true;
            }
            _ => panic!("{} is not integer object.", obj),
        }
    }

    fn test_null_object(obj: &object::Object) -> bool {
        if let object::Object::Null = obj {
            return false;
        }
        return true;
    }
}
