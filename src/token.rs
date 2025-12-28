pub(crate) struct Token<'source> {
    pub(crate) kind: TokenType,
    pub(crate) line: u32,
    pub(crate) lexeme: &'source str,
}

impl<'source> Token<'source> {
    pub(crate) fn new(kind: TokenType, line: u32, lexeme: &'source str) -> Self {
        Self { kind, line, lexeme }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub(crate) enum TokenType {
    // Single character tokens.
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
    For,
    Fun,
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

    // Other.
    Error,
    EoF,
}
