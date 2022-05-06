use phf::phf_map;
use std::fmt;

#[allow(dead_code)]
#[derive(PartialOrd, PartialEq, Debug, Clone)]
pub enum TokenType {
    ILLEGAL,
    EOF,

    IDENT,
    INT,

    ASSIGN,
    PLUS,
    MINUS,
    BANG,
    ASTERISK,
    SLASH,

    LT,
    GT,
    EQ,
    NOTEQ,

    COMMA,
    SEMICOLON,

    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,

    FUNCTION,
    LET,
    TRUE,
    FALSE,
    IF,
    ELSE,
    RETURN,
}

pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // is very similar to `println!`.
        // 厳密に最初の要素を、与えられた出力ストリーム `f` に書き込みます。
        // `fmt::Result`を返します。これはオペレーションが成功したか否か
        // を表します。
        // `write!`は`println!`に非常によく似た文法を使用していることに注目。
        write!(
            f,
            "token_type: {:?}, literal: {}",
            self.token_type, self.literal
        )
    }
}

static KEYWORD: phf::Map<&'static str, TokenType> = phf_map! {
    "fn" => TokenType::FUNCTION,
    "let" => TokenType::LET,
    "true" => TokenType::TRUE,
    "false" => TokenType::FALSE,
    "if" => TokenType::IF,
    "else" => TokenType::ELSE,
    "return" => TokenType::RETURN,
};

pub fn lookup_ident(ident: &str) -> TokenType {
    if let Some(token_type) = KEYWORD.get(ident) {
        return (*token_type).clone();
    }

    return TokenType::IDENT;
}
