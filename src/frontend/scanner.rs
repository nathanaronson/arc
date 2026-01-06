use crate::frontend::{Token, TokenType};
use phf::phf_map;

static RESERVED_KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "for" => TokenType::For,
    "fun" => TokenType::Fun,
    "if" => TokenType::If,
    "nil" => TokenType::Nil,
    "or" => TokenType::Or,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "true" => TokenType::True,
    "var" => TokenType::Var,
    "while" => TokenType::While,
};

pub(crate) struct Scanner<'source> {
    code: &'source str,
    start: usize,
    current: usize,
    line: u32,
}

impl<'source> Scanner<'source> {
    pub(crate) fn new(source: &'source str) -> Self {
        Self {
            code: source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub(crate) fn scan_token(&mut self) -> Token<'source> {
        self.skip_whitespace();
        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TokenType::EoF);
        }

        match self.advance() {
            b'(' => self.make_token(TokenType::LeftParen),
            b')' => self.make_token(TokenType::RightParen),
            b'{' => self.make_token(TokenType::LeftBrace),
            b'}' => self.make_token(TokenType::RightBrace),
            b';' => self.make_token(TokenType::Semicolon),
            b',' => self.make_token(TokenType::Comma),
            b'.' => self.make_token(TokenType::Dot),
            b'-' => self.make_token(TokenType::Minus),
            b'+' => self.make_token(TokenType::Plus),
            b'/' => self.make_token(TokenType::Slash),
            b'*' => self.make_token(TokenType::Star),
            b'!' if self.matches(b'=') => self.make_token(TokenType::BangEqual),
            b'!' => self.make_token(TokenType::Bang),
            b'=' if self.matches(b'=') => self.make_token(TokenType::EqualEqual),
            b'=' => self.make_token(TokenType::Equal),
            b'<' if self.matches(b'=') => self.make_token(TokenType::LessEqual),
            b'<' => self.make_token(TokenType::Less),
            b'>' if self.matches(b'=') => self.make_token(TokenType::GreaterEqual),
            b'>' => self.make_token(TokenType::Greater),
            b'"' => self.string(),
            c if Self::is_digit(c) => self.number(),
            c if Self::is_alphabetic(c) => self.identifier(),
            _ => self.error_token("Unexpected character."),
        }
    }

    fn advance(&mut self) -> u8 {
        let c = self.peek();
        self.current += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                b' ' | b'\r' | b'\t' => {
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                b'/' if self.peek_next() == b'/' => {
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.advance();
                    }
                }
                _ => return,
            }
        }
    }

    fn make_token(&self, kind: TokenType) -> Token<'source> {
        Token::new(kind, self.line, self.lexeme())
    }

    fn error_token(&self, message: &'static str) -> Token<'static> {
        Token::new(TokenType::Error, self.line, message)
    }

    fn string(&mut self) -> Token<'source> {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }

        self.advance();
        self.make_token(TokenType::String)
    }

    fn number(&mut self) -> Token<'source> {
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == b'.' && Self::is_digit(self.peek_next()) {
            self.advance();

            while Self::is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token<'source> {
        while Self::is_alphabetic(self.peek()) || Self::is_digit(self.peek()) {
            self.advance();
        }

        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        RESERVED_KEYWORDS
            .get(self.lexeme())
            .cloned()
            .unwrap_or(TokenType::Identifier)
    }

    fn get_char(&self, n: usize) -> u8 {
        self.code.as_bytes()[n]
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            return b'\0';
        }
        self.get_char(self.current)
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.code.len() {
            return b'\0';
        }
        self.get_char(self.current + 1)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.code.len()
    }

    fn is_digit(c: u8) -> bool {
        c.is_ascii_digit()
    }

    fn is_alphabetic(c: u8) -> bool {
        c.is_ascii_alphabetic() || c == b'_'
    }

    fn matches(&mut self, expected: u8) -> bool {
        if self.is_at_end() || self.peek() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn lexeme(&self) -> &'source str {
        &self.code[self.start..self.current]
    }
}
