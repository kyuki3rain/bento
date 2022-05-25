use super::token;

pub struct Lexer {
    pub input: String,
    position: usize,
    read_position: usize,
    ch: char,
}

#[allow(dead_code)]
impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let mut l = Lexer {
            input: input.to_string(),
            position: 0,
            read_position: 0,
            ch: 'a',
        };
        l.read_char();
        return l;
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input.chars().nth(self.read_position).unwrap();
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> token::Token {
        self.skip_whitespace();

        let tok = match self.ch {
            '=' => {
                if self.peek_char() == '=' {
                    let ch = self.ch;
                    self.read_char();
                    token::new_token(token::TokenType::EQ, ch.to_string() + &self.ch.to_string())
                } else {
                    token::new_token(token::TokenType::ASSIGN, self.ch.to_string())
                }
            }
            '+' => token::new_token(token::TokenType::PLUS, self.ch.to_string()),
            '-' => token::new_token(token::TokenType::MINUS, self.ch.to_string()),
            '!' => {
                if self.peek_char() == '=' {
                    let ch = self.ch;
                    self.read_char();
                    token::new_token(
                        token::TokenType::NOTEQ,
                        ch.to_string() + &self.ch.to_string(),
                    )
                } else {
                    token::new_token(token::TokenType::BANG, self.ch.to_string())
                }
            }
            '*' => token::new_token(token::TokenType::ASTERISK, self.ch.to_string()),
            '/' => token::new_token(token::TokenType::SLASH, self.ch.to_string()),
            '<' => token::new_token(token::TokenType::LT, self.ch.to_string()),
            '>' => token::new_token(token::TokenType::GT, self.ch.to_string()),
            ',' => token::new_token(token::TokenType::COMMA, self.ch.to_string()),
            ';' => token::new_token(token::TokenType::SEMICOLON, self.ch.to_string()),
            '(' => token::new_token(token::TokenType::LPAREN, self.ch.to_string()),
            ')' => token::new_token(token::TokenType::RPAREN, self.ch.to_string()),
            '{' => token::new_token(token::TokenType::LBRACE, self.ch.to_string()),
            '}' => token::new_token(token::TokenType::RBRACE, self.ch.to_string()),
            '[' => token::new_token(token::TokenType::LBRACKET, self.ch.to_string()),
            ']' => token::new_token(token::TokenType::RBRACKET, self.ch.to_string()),
            '"' => token::new_token(token::TokenType::STRING, self.read_string()),
            ':' => token::new_token(token::TokenType::COLON, self.ch.to_string()),
            '.' => token::new_token(token::TokenType::DOT, self.ch.to_string()),
            '\0' => token::Token {
                token_type: token::TokenType::EOF,
                literal: String::from(""),
            },
            _ => {
                if self.ch.is_alphabetic() {
                    let literal = self.read_identifier();
                    let token_type = token::lookup_ident(&literal);

                    return token::new_token(token_type, literal);
                } else if self.ch.is_numeric() {
                    return token::new_token(token::TokenType::INT, self.read_number());
                } else {
                    token::new_token(token::TokenType::ILLEGAL, self.ch.to_string())
                }
            }
        };

        self.read_char();
        return tok;
    }

    fn read_number(&mut self) -> String {
        let position = self.position;
        while self.ch.is_numeric() {
            self.read_char();
        }

        return self.get_slice(position, self.position);
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while self.ch.is_alphabetic() {
            self.read_char();
        }

        return self.get_slice(position, self.position);
    }

    fn read_string(&mut self) -> String {
        let position = self.position + 1;
        loop {
            self.read_char();
            if self.ch == '"' || self.ch == '\0' {
                break;
            }
        }

        return self.get_slice(position, self.position);
    }

    fn get_slice(&self, start: usize, end: usize) -> String {
        let begin = self.input.char_indices().nth(start).unwrap().0;
        if let Some(end) = self.input.char_indices().nth(end) {
            let str2 = &self.input[begin..end.0];
            return String::from(str2);
        }

        let str2 = &self.input[begin..];
        return String::from(str2);
    }

    fn skip_whitespace(&mut self) {
        while self.ch == ' ' || self.ch == '\t' || self.ch == '\n' || self.ch == '\r' {
            self.read_char();
        }
    }

    fn peek_char(&mut self) -> char {
        if self.read_position >= self.input.len() {
            return '\0';
        } else {
            return self.input.chars().nth(self.read_position).unwrap();
        }
    }
}

#[cfg(test)]
mod lexer_tests {
    use super::*;
    #[test]
    fn test_next_token() {
        let input = String::from("=+(){},;");
        counted_array!(
            let tests: [(token::TokenType, &str); _] = [
                (token::TokenType::ASSIGN, "="),
                (token::TokenType::PLUS, "+"),
                (token::TokenType::LPAREN, "("),
                (token::TokenType::RPAREN, ")"),
                (token::TokenType::LBRACE, "{"),
                (token::TokenType::RBRACE, "}"),
                (token::TokenType::COMMA, ","),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::EOF, ""),
            ]
        );

        let mut l = Lexer::new(&input);

        for (token_type, literal) in tests {
            let tok = l.next_token();

            assert_eq!(tok.token_type, token_type);
            assert_eq!(tok.literal, String::from(literal));
        }
    }

    #[test]
    fn test_next_token_2() {
        let input = String::from(
            "
let five = 5;
let ten = 10;

let add = fn(x, y) {
    x + y;
};

let result = add(five, ten);

!-/*5;
5 < 10 > 5;
if (5 < 10) {
    return true;
} else {
    return false;
}
10 == 10;
10 != 9;
\"foobar\"
\"foo bar\"
[1, 2];
{\"foo\": \"bar\"}
        ",
        );
        counted_array!(
            let tests: [(token::TokenType, &str); _] = [
                (token::TokenType::LET, "let"),
                (token::TokenType::IDENT, "five"),
                (token::TokenType::ASSIGN, "="),
                (token::TokenType::INT, "5"),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::LET, "let"),
                (token::TokenType::IDENT, "ten"),
                (token::TokenType::ASSIGN, "="),
                (token::TokenType::INT, "10"),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::LET, "let"),
                (token::TokenType::IDENT, "add"),
                (token::TokenType::ASSIGN, "="),
                (token::TokenType::FUNCTION, "fn"),
                (token::TokenType::LPAREN, "("),
                (token::TokenType::IDENT, "x"),
                (token::TokenType::COMMA, ","),
                (token::TokenType::IDENT, "y"),
                (token::TokenType::RPAREN, ")"),
                (token::TokenType::LBRACE, "{"),
                (token::TokenType::IDENT, "x"),
                (token::TokenType::PLUS, "+"),
                (token::TokenType::IDENT, "y"),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::RBRACE, "}"),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::LET, "let"),
                (token::TokenType::IDENT, "result"),
                (token::TokenType::ASSIGN, "="),
                (token::TokenType::IDENT, "add"),
                (token::TokenType::LPAREN, "("),
                (token::TokenType::IDENT, "five"),
                (token::TokenType::COMMA, ","),
                (token::TokenType::IDENT, "ten"),
                (token::TokenType::RPAREN, ")"),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::BANG, "!"),
                (token::TokenType::MINUS, "-"),
                (token::TokenType::SLASH, "/"),
                (token::TokenType::ASTERISK, "*"),
                (token::TokenType::INT, "5"),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::INT, "5"),
                (token::TokenType::LT, "<"),
                (token::TokenType::INT, "10"),
                (token::TokenType::GT, ">"),
                (token::TokenType::INT, "5"),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::IF, "if"),
                (token::TokenType::LPAREN, "("),
                (token::TokenType::INT, "5"),
                (token::TokenType::LT, "<"),
                (token::TokenType::INT, "10"),
                (token::TokenType::RPAREN, ")"),
                (token::TokenType::LBRACE, "{"),
                (token::TokenType::RETURN, "return"),
                (token::TokenType::TRUE, "true"),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::RBRACE, "}"),
                (token::TokenType::ELSE, "else"),
                (token::TokenType::LBRACE, "{"),
                (token::TokenType::RETURN, "return"),
                (token::TokenType::FALSE, "false"),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::RBRACE, "}"),
                (token::TokenType::INT, "10"),
                (token::TokenType::EQ, "=="),
                (token::TokenType::INT, "10"),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::INT, "10"),
                (token::TokenType::NOTEQ, "!="),
                (token::TokenType::INT, "9"),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::STRING, "foobar"),
                (token::TokenType::STRING, "foo bar"),
                (token::TokenType::LBRACKET, "["),
                (token::TokenType::INT, "1"),
                (token::TokenType::COMMA, ","),
                (token::TokenType::INT, "2"),
                (token::TokenType::RBRACKET, "]"),
                (token::TokenType::SEMICOLON, ";"),
                (token::TokenType::LBRACE, "{"),
                (token::TokenType::STRING, "foo"),
                (token::TokenType::COLON, ":"),
                (token::TokenType::STRING, "bar"),
                (token::TokenType::RBRACE, "}"),
                (token::TokenType::EOF, ""),
            ]
        );

        let mut l = Lexer::new(&input);

        for (token_type, literal) in tests {
            let tok = l.next_token();

            assert_eq!(tok.token_type, token_type);
            assert_eq!(tok.literal, String::from(literal));
        }
    }
}
