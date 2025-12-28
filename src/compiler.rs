use crate::{Chunk, Disassembler, Instruction, Scanner, Token, TokenType, Value};
use std::collections::HashMap;

#[derive(Eq, PartialEq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    fn next(&self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::None,
        }
    }
}

struct ParseRule<'source> {
    prefix: Option<ParseFn<'source>>,
    infix: Option<ParseFn<'source>>,
    precedence: Precedence,
}

impl<'source> ParseRule<'source> {
    fn new(
        prefix: Option<ParseFn<'source>>,
        infix: Option<ParseFn<'source>>,
        precedence: Precedence,
    ) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}

type ParseFn<'source> = fn(&mut Compiler<'source>);

macro_rules! rule {
    ($map:expr, $kind:ident, $prefix:expr, $infix:expr, $prec:ident) => {
        $map.insert(
            TokenType::$kind,
            ParseRule::new($prefix, $infix, Precedence::$prec),
        );
    };
}

pub(crate) struct Compiler<'source> {
    scanner: Scanner<'source>,
    rules: HashMap<TokenType, ParseRule<'source>>,
    previous: Token<'source>,
    current: Token<'source>,
    chunk: &'source mut Chunk,
    had_error: bool,
    panic_mode: bool,
}

impl<'source> Compiler<'source> {
    pub(crate) fn new(source: &'source str, chunk: &'source mut Chunk) -> Self {
        Self {
            scanner: Scanner::new(source),
            rules: Compiler::build_rules(),
            previous: Token::null(),
            current: Token::null(),
            chunk,
            had_error: false,
            panic_mode: false,
        }
    }

    fn build_rules() -> HashMap<TokenType, ParseRule<'source>> {
        let mut rules = HashMap::new();

        // Literals / grouping.
        rule!(rules, LeftParen, Some(Compiler::grouping), None, None);
        rule!(rules, RightParen, None, None, None);
        rule!(rules, LeftBrace, None, None, None);
        rule!(rules, RightBrace, None, None, None);
        rule!(rules, Comma, None, None, None);
        rule!(rules, Dot, None, None, None);

        // Operators.
        rule!(
            rules,
            Minus,
            Some(Compiler::unary),
            Some(Compiler::binary),
            Term
        );
        rule!(rules, Plus, None, Some(Compiler::binary), Term);
        rule!(rules, Semicolon, None, None, None);
        rule!(rules, Slash, None, Some(Compiler::binary), Factor);
        rule!(rules, Star, None, Some(Compiler::binary), Factor);

        // Logical / comparison.
        rule!(rules, Bang, None, None, None);
        rule!(rules, BangEqual, None, None, None);
        rule!(rules, Equal, None, None, None);
        rule!(rules, EqualEqual, None, None, None);
        rule!(rules, Greater, None, None, None);
        rule!(rules, GreaterEqual, None, None, None);
        rule!(rules, Less, None, None, None);
        rule!(rules, LessEqual, None, None, None);

        // Identifiers / literals.
        rule!(rules, Identifier, None, None, None);
        rule!(rules, String, None, None, None);
        rule!(rules, Number, Some(Compiler::number), None, None);
        rule!(rules, False, None, None, None);
        rule!(rules, Nil, None, None, None);
        rule!(rules, True, None, None, None);

        // Keywords.
        rule!(rules, And, None, None, None);
        rule!(rules, Class, None, None, None);
        rule!(rules, Else, None, None, None);
        rule!(rules, For, None, None, None);
        rule!(rules, Fun, None, None, None);
        rule!(rules, If, None, None, None);
        rule!(rules, Or, None, None, None);
        rule!(rules, Print, None, None, None);
        rule!(rules, Return, None, None, None);
        rule!(rules, Super, None, None, None);
        rule!(rules, This, None, None, None);
        rule!(rules, Var, None, None, None);
        rule!(rules, While, None, None, None);

        // Special / error tokens.
        rule!(rules, Error, None, None, None);
        rule!(rules, EoF, None, None, None);

        rules
    }

    pub(crate) fn compile(&mut self) -> bool {
        self.advance();
        self.expression();
        self.consume(TokenType::EoF, "Expected end of expression.");
        self.end_compiler();
        !self.had_error
    }

    fn advance(&mut self) {
        self.previous = std::mem::replace(&mut self.current, self.scanner.scan_token());

        while matches!(self.current.kind, TokenType::Error) {
            self.error_at_current(self.current.lexeme);
            self.current = self.scanner.scan_token();
        }
    }

    fn consume(&mut self, kind: TokenType, message: &str) {
        if self.current.kind == kind {
            return self.advance();
        }

        self.error_at_current(message);
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        self.chunk.write(instruction, self.previous.line);
    }

    fn end_compiler(&mut self) {
        #[cfg(debug_assertions)]
        {
            if !self.had_error {
                let disassembler = Disassembler::new(self.chunk);
                disassembler.disassemble_chunk("code");
            }
        }
        self.emit_return();
    }

    fn binary(&mut self) {
        let operator_type = self.previous.kind;
        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.next());

        match operator_type {
            TokenType::Plus => self.emit_instruction(Instruction::Add),
            TokenType::Minus => self.emit_instruction(Instruction::Subtract),
            TokenType::Star => self.emit_instruction(Instruction::Multiply),
            TokenType::Slash => self.emit_instruction(Instruction::Divide),
            _ => {}
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn number(&mut self) {
        let value: f64 = self.previous.lexeme.parse().unwrap();
        self.emit_constant(value);
    }

    fn unary(&mut self) {
        let operator_type = self.previous.kind;
        self.parse_precedence(Precedence::Unary);
        if matches!(operator_type, TokenType::Minus) {
            self.emit_instruction(Instruction::Negate);
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(self.previous.kind).prefix;
        match prefix_rule {
            None => self.error("Expect expression."),
            Some(rule) => {
                rule(self);
                while precedence <= self.get_rule(self.current.kind).precedence {
                    self.advance();
                    let infix_rule = self.get_rule(self.previous.kind).infix.unwrap();
                    infix_rule(self);
                }
            }
        }
    }

    fn get_rule(&self, kind: TokenType) -> &ParseRule<'source> {
        self.rules.get(&kind).unwrap()
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn emit_return(&mut self) {
        self.emit_instruction(Instruction::Return);
    }

    fn make_constant(&mut self, value: Value) -> usize {
        let index = self.chunk.add_constant(value);
        match u8::try_from(index) {
            Ok(_) => index,
            Err(_) => {
                self.error("Too many constants in one chunk.");
                0
            }
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let index = self.make_constant(value);
        self.emit_instruction(Instruction::Constant(index));
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current, message);
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.previous, message);
    }

    fn error_at(&mut self, token: Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.had_error = true;
        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);
        match token.kind {
            TokenType::EoF => eprint!(" at end"),
            TokenType::Error => {}
            _ => eprint!(" at '{}'", token.lexeme),
        }
        eprintln!(": {}", message);
    }
}
