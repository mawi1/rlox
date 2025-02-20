use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
}

#[derive(Debug)]
pub enum Literal {
    Number(f64),
    String(String),
}

#[derive(Debug)]
pub struct Token {
    pub ty: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: u32,
}

impl Token {
    pub fn new(ty: TokenType, lexeme: String, literal: Option<Literal>, line: u32) -> Self {
        Self {
            ty,
            lexeme,
            literal,
            line,
        }
    }
}
