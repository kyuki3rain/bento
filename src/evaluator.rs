use super::{ast, environment, lexer, object, parser};

pub fn eval_program(
    program: ast::Program,
    env: &mut environment::Environment,
) -> Option<object::Object> {
    let mut result = None;
    for stmt in program.statements {
        result = eval_statement(stmt, env);

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

fn eval_block_statement(
    statements: Vec<ast::Statement>,
    env: &mut environment::Environment,
) -> Option<object::Object> {
    let mut result = None;
    for stmt in statements {
        result = eval_statement(stmt, env);
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

fn eval_statement(
    stmt: ast::Statement,
    env: &mut environment::Environment,
) -> Option<object::Object> {
    match stmt {
        ast::Statement::LetStatement {
            token: _,
            name,
            value,
        } => match eval_expression(value, env) {
            Some(val) => {
                if !is_error(&val) {
                    env.set(name.to_string(), &val);
                }
                return Some(val);
            }
            None => return None,
        },
        ast::Statement::ReturnStatement {
            token: _,
            return_value,
        } => match eval_expression(return_value, env) {
            Some(value) => {
                if is_error(&value) {
                    return Some(value);
                }
                return Some(object::Object::Return(Box::new(value)));
            }
            None => return None,
        },
        ast::Statement::ExpressionStatement {
            token: _,
            expression,
        } => return eval_expression(expression, env),
        ast::Statement::BlockStatement {
            token: _,
            statements,
        } => return eval_block_statement(statements, env),
    }
}

fn eval_expression(
    exp: ast::Expression,
    env: &mut environment::Environment,
) -> Option<object::Object> {
    match exp {
        ast::Expression::Identifier { token: _, value } => return eval_identifier(value, env),
        ast::Expression::IntegerLiteral { token: _, value } => {
            return Some(object::Object::Integer(value))
        }
        ast::Expression::PrefixExpression {
            token: _,
            operator,
            right,
        } => match eval_expression(*right, env) {
            Some(right_evaluated) => {
                if is_error(&right_evaluated) {
                    return Some(right_evaluated);
                }
                return eval_prefix_expression(operator, right_evaluated, env);
            }
            None => return None,
        },
        ast::Expression::InfixExpression {
            token: _,
            left,
            operator,
            right,
        } => match eval_expression(*right, env) {
            Some(right_evaluated) => {
                if is_error(&right_evaluated) {
                    return Some(right_evaluated);
                }
                match eval_expression(*left, env) {
                    Some(left_evaluated) => {
                        if is_error(&left_evaluated) {
                            return Some(left_evaluated);
                        }
                        return eval_infix_expression(
                            operator,
                            left_evaluated,
                            right_evaluated,
                            env,
                        );
                    }
                    None => return None,
                }
            }
            None => return None,
        },
        ast::Expression::Boolean { token: _, value } => {
            return Some(object::Object::Boolean(value))
        }
        ast::Expression::IfExpression {
            token: _,
            condition,
            consequence,
            alternative,
        } => return eval_if_expression(condition, consequence, alternative, env),
        ast::Expression::FunctionLiteral {
            token: _,
            parameters,
            body,
        } => {
            return Some(object::Object::Function {
                parameters,
                env: env.clone(),
                body: *body,
            })
        }
        ast::Expression::CallExpression {
            token: _,
            function,
            arguments,
        } => {
            if let Some(func) = eval_expression(*function, env) {
                let args = eval_expressions(arguments, env);
                if args.len() == 1 && is_error(&args[0]) {
                    return Some(args[0].clone());
                }
                return apply_function(func, args);
            } else {
                return None;
            }
        }
    }
}

fn apply_function(func: object::Object, args: Vec<object::Object>) -> Option<object::Object> {
    match func {
        object::Object::Function {
            parameters,
            body,
            env,
        } => {
            let mut extended_env = env.new_enclosed_environment();

            for (i, p) in parameters.iter().enumerate() {
                match p {
                    ast::Expression::Identifier { token: _, value } => {
                        extended_env.set(value.clone(), &args[i])
                    }
                    _ => return None,
                }
            }

            if let Some(evaluated) = eval_statement(body, &mut extended_env) {
                match evaluated {
                    object::Object::Return(value) => return Some(*value),
                    _ => {
                        return Some(evaluated);
                    }
                }
            }

            return None;
        }
        _ => None,
    }
}

fn eval_expressions(
    exps: Vec<ast::Expression>,
    env: &mut environment::Environment,
) -> Vec<object::Object> {
    let mut result = Vec::new();
    for e in exps {
        if let Some(evaluated) = eval_expression(e, env) {
            if is_error(&evaluated) {
                return vec![evaluated];
            }
            result.push(evaluated);
        }
    }
    return result;
}

fn eval_prefix_expression(
    operator: String,
    right: object::Object,
    _: &mut environment::Environment,
) -> Option<object::Object> {
    match &*operator {
        "!" => return eval_bang_operator_expression(right),
        "-" => return eval_minus_prefix_operator_expression(right),
        _ => {
            return Some(object::Object::Error(format!(
                "unknown operator: {}{}",
                operator, right
            )))
        }
    }
}

fn eval_infix_expression(
    operator: String,
    left: object::Object,
    right: object::Object,
    env: &mut environment::Environment,
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
                return eval_integer_infix_expression(operator, left_value, right_value, env)
            }
            _ => return None,
        },
        object::Object::Boolean(left_value) => match right {
            object::Object::Boolean(right_value) => match &*operator {
                "==" => return Some(object::Object::Boolean(left_value == right_value)),
                "!=" => return Some(object::Object::Boolean(left_value != right_value)),
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
    env: &mut environment::Environment,
) -> Option<object::Object> {
    match &*operator {
        "+" => return Some(object::Object::Integer(left_value + right_value)),
        "-" => return Some(object::Object::Integer(left_value - right_value)),
        "*" => return Some(object::Object::Integer(left_value * right_value)),
        "/" => return Some(object::Object::Integer(left_value / right_value)),
        "<" => return Some(object::Object::Boolean(left_value < right_value)),
        ">" => return Some(object::Object::Boolean(left_value > right_value)),
        "==" => return Some(object::Object::Boolean(left_value == right_value)),
        "!=" => return Some(object::Object::Boolean(left_value != right_value)),
        _ => {
            return Some(object::Object::Error(format!(
                "unknown operator: {} {} {}",
                left_value, operator, right_value
            )))
        }
    }
}

fn eval_bang_operator_expression(right: object::Object) -> Option<object::Object> {
    match right {
        object::Object::Boolean(value) => return Some(object::Object::Boolean(!value)),
        object::Object::Null => return Some(object::Object::Boolean(true)),
        _ => Some(object::Object::Boolean(false)),
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
    condition: Box<ast::Expression>,
    consequence: Box<ast::Statement>,
    alternative: Option<Box<ast::Statement>>,
    env: &mut environment::Environment,
) -> Option<object::Object> {
    if let Some(evaluated_condition) = eval_expression(*condition, env) {
        if is_error(&evaluated_condition) {
            return Some(evaluated_condition);
        }
        if is_truthy(evaluated_condition) {
            return eval_statement(*consequence, env);
        } else if let Some(alt) = alternative {
            return eval_statement(*alt, env);
        } else {
            return Some(object::Object::Null);
        }
    } else {
        return None;
    }
}

fn eval_identifier(ident: String, env: &mut environment::Environment) -> Option<object::Object> {
    match (*env).get(ident.clone()) {
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

#[cfg(test)]
mod evaluator_tests {
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
                ("fn(x) { x + 2; };", vec!["x"], "{\n\t(x + 2)\n}"),
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
        let mut env = environment::Environment::new();
        let l = lexer::Lexer::new(input);
        let mut p = parser::Parser::new(l);
        let program = p.parse_program();

        match eval_program(program, &mut env) {
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
                    return false;
                }
                return true;
            }
            _ => return false,
        }
    }

    fn test_null_object(obj: object::Object) -> bool {
        if let object::Object::Null = obj {
            return false;
        }
        return true;
    }
}
