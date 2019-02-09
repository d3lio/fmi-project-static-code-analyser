#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Whitespace(String),
    Comment(String),

    Ident(String),

    KwFunction,
    KwVar,
    KwIf,
    KwElse,
    KwFor,
    KwIn,

    Number(f64),
    String(String),

    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    Or,
    And,
    Plus,
    Minus,
    Asterisk,
    FSlash,
    Eq,
    NotEq,

    Assign,
    Excl,

    Colon,
    Semicolon,
    Comma,
    Arrow,

    Unknown(String)
}
