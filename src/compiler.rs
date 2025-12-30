use crate::{Chunk, Instruction, Scanner, Token, TokenType, Value};
use std::collections::HashMap;

#[cfg(debug_assertions)]
use crate::Disassembler;

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

type ParseFn<'source> = fn(&mut Parser<'source>, bool);

macro_rules! rule {
    ($map:expr, $kind:ident, $prefix:expr, $infix:expr, $prec:ident) => {
        $map.insert(
            TokenType::$kind,
            ParseRule::new($prefix, $infix, Precedence::$prec),
        );
    };
}

struct Compiler<'source> {
    locals: Vec<Local<'source>>,
    scope_depth: i32,
}

impl<'source> Compiler<'source> {
    const LOCAL_MAX: usize = u8::MAX as usize + 1;

    fn new() -> Self {
        Self {
            locals: Vec::with_capacity(Self::LOCAL_MAX),
            scope_depth: 0,
        }
    }

    fn is_same_scope(&self, name: &Token) -> u32 {
        let mut count = 0;
        for local in self.locals.iter().rev() {
            if local.depth != -1 && local.depth < self.scope_depth {
                return count;
            }

            if name.lexeme == local.name.lexeme {
                count += 1;
            }
        }
        count
    }
}

struct Local<'source> {
    name: Token<'source>,
    depth: i32,
}

impl<'source> Local<'source> {
    fn new(name: Token<'source>, depth: i32) -> Self {
        Self { name, depth }
    }
}

pub(crate) struct Parser<'source> {
    scanner: Scanner<'source>,
    rules: HashMap<TokenType, ParseRule<'source>>,
    previous: Token<'source>,
    current: Token<'source>,
    chunk: &'source mut Chunk,
    compiler: Compiler<'source>,
    had_error: bool,
    panic_mode: bool,
}

impl<'source> Parser<'source> {
    pub(crate) fn new(source: &'source str, chunk: &'source mut Chunk) -> Self {
        Self {
            scanner: Scanner::new(source),
            rules: Parser::build_rules(),
            previous: Token::null(),
            current: Token::null(),
            chunk,
            compiler: Compiler::new(),
            had_error: false,
            panic_mode: false,
        }
    }

    fn build_rules() -> HashMap<TokenType, ParseRule<'source>> {
        let mut rules = HashMap::new();

        // Literals / grouping.
        rule!(rules, LeftParen, Some(Parser::grouping), None, None);
        rule!(rules, RightParen, None, None, None);
        rule!(rules, LeftBrace, None, None, None);
        rule!(rules, RightBrace, None, None, None);
        rule!(rules, Comma, None, None, None);
        rule!(rules, Dot, None, None, None);

        // Operators.
        rule!(
            rules,
            Minus,
            Some(Parser::unary),
            Some(Parser::binary),
            Term
        );
        rule!(rules, Plus, None, Some(Parser::binary), Term);
        rule!(rules, Semicolon, None, None, None);
        rule!(rules, Slash, None, Some(Parser::binary), Factor);
        rule!(rules, Star, None, Some(Parser::binary), Factor);

        // Logical / comparison.
        rule!(rules, Bang, Some(Parser::unary), None, None);
        rule!(rules, BangEqual, None, Some(Parser::binary), Equality);
        rule!(rules, Equal, None, None, None);
        rule!(rules, EqualEqual, None, Some(Parser::binary), Equality);
        rule!(rules, Greater, None, Some(Parser::binary), Comparison);
        rule!(rules, GreaterEqual, None, Some(Parser::binary), Comparison);
        rule!(rules, Less, None, Some(Parser::binary), Comparison);
        rule!(rules, LessEqual, None, Some(Parser::binary), Comparison);

        // Identifiers / literals.
        rule!(rules, Identifier, Some(Parser::variable), None, None);
        rule!(rules, String, Some(Parser::string), None, None);
        rule!(rules, Number, Some(Parser::number), None, None);
        rule!(rules, False, Some(Parser::literal), None, None);
        rule!(rules, Nil, Some(Parser::literal), None, None);
        rule!(rules, True, Some(Parser::literal), None, None);

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
        while !self.matches(TokenType::EoF) {
            self.declaration();
        }
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

    fn check(&self, kind: TokenType) -> bool {
        self.current.kind == kind
    }

    fn matches(&mut self, kind: TokenType) -> bool {
        if !self.check(kind) {
            return false;
        }
        self.advance();
        true
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        self.chunk.write(instruction, self.previous.line);
    }

    fn emit_instructions(&mut self, instruction_1: Instruction, instruction_2: Instruction) {
        self.emit_instruction(instruction_1);
        self.emit_instruction(instruction_2);
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        #[cfg(debug_assertions)]
        {
            if !self.had_error {
                let disassembler = Disassembler::new(self.chunk);
                disassembler.disassemble_chunk("code");
            }
        }
    }

    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;

        while let Some(local) = self.compiler.locals.last() {
            if local.depth > self.compiler.scope_depth {
                self.emit_instruction(Instruction::Pop);
                self.compiler.locals.pop();
            } else {
                break;
            }
        }
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator_type = self.previous.kind;
        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.next());

        match operator_type {
            TokenType::BangEqual => self.emit_instructions(Instruction::Equal, Instruction::Not),
            TokenType::EqualEqual => self.emit_instruction(Instruction::Equal),
            TokenType::Greater => self.emit_instruction(Instruction::Greater),
            TokenType::GreaterEqual => self.emit_instructions(Instruction::Less, Instruction::Not),
            TokenType::Less => self.emit_instruction(Instruction::Less),
            TokenType::LessEqual => self.emit_instructions(Instruction::Greater, Instruction::Not),
            TokenType::Plus => self.emit_instruction(Instruction::Add),
            TokenType::Minus => self.emit_instruction(Instruction::Subtract),
            TokenType::Star => self.emit_instruction(Instruction::Multiply),
            TokenType::Slash => self.emit_instruction(Instruction::Divide),
            _ => {}
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        match self.previous.kind {
            TokenType::False => self.emit_instruction(Instruction::False),
            TokenType::True => self.emit_instruction(Instruction::True),
            TokenType::Nil => self.emit_instruction(Instruction::Nil),
            _ => {}
        }
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn number(&mut self, _can_assign: bool) {
        let value: f64 = self.previous.lexeme.parse().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn string(&mut self, _can_assign: bool) {
        let value = self.previous.lexeme.trim_matches('"').to_string();
        self.emit_constant(Value::String(value));
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous, can_assign);
    }

    fn resolve_local(&mut self, name: &Token) -> Option<usize> {
        for (i, local) in self.compiler.locals.iter().enumerate().rev() {
            if name.lexeme == local.name.lexeme {
                return if local.depth == -1 {
                    self.error("Can't read local variable in its own initializer");
                    None
                } else {
                    Some(i)
                };
            }
        }
        None
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let get_op;
        let set_op;
        match self.resolve_local(&name) {
            Some(arg) => {
                get_op = Instruction::GetLocal(arg);
                set_op = Instruction::SetLocal(arg);
            }
            None => {
                let arg = self.identifier_constant(name);
                get_op = Instruction::GetGlobal(arg);
                set_op = Instruction::SetGlobal(arg);
            }
        }

        if can_assign && self.matches(TokenType::Equal) {
            self.expression();
            self.emit_instruction(set_op);
        } else {
            self.emit_instruction(get_op);
        }
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator_type = self.previous.kind;
        self.parse_precedence(Precedence::Unary);
        match operator_type {
            TokenType::Minus => self.emit_instruction(Instruction::Negate),
            TokenType::Bang => self.emit_instruction(Instruction::Not),
            _ => {}
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(self.previous.kind).prefix;
        match prefix_rule {
            None => self.error("Expect expression."),
            Some(rule) => {
                let can_assign = precedence <= Precedence::Assignment;
                rule(self, can_assign);
                while precedence <= self.get_rule(self.current.kind).precedence {
                    self.advance();
                    let infix_rule = self.get_rule(self.previous.kind).infix.unwrap();
                    infix_rule(self, can_assign);
                }

                if can_assign && self.matches(TokenType::Equal) {
                    self.error("Invalid assignment target.");
                }
            }
        }
    }

    fn get_rule(&self, kind: TokenType) -> &ParseRule<'source> {
        self.rules.get(&kind).unwrap()
    }

    fn parse_variable(&mut self, message: &str) -> usize {
        self.consume(TokenType::Identifier, message);
        self.declare_variable();
        if self.compiler.scope_depth > 0 {
            return 0;
        }
        self.identifier_constant(self.previous)
    }

    fn define_variable(&mut self, index: usize) {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_instruction(Instruction::DefineGlobal(index));
    }

    fn mark_initialized(&mut self) {
        self.compiler.locals.last_mut().unwrap().depth = self.compiler.scope_depth;
    }

    fn identifier_constant(&mut self, token: Token) -> usize {
        self.make_constant(Value::String(token.lexeme.to_string()))
    }

    fn declare_variable(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }

        let name = self.previous;
        for _ in 0..self.compiler.is_same_scope(&name) {
            self.error("Already a variable with this name in this scope.");
        }

        self.add_local(name);
    }

    fn add_local(&mut self, name: Token<'source>) {
        if self.compiler.locals.len() == Compiler::LOCAL_MAX {
            self.error("Too many local variables in function.");
            return;
        }

        let local = Local::new(name, -1);
        self.compiler.locals.push(local);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EoF) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block.");
    }

    fn declaration(&mut self) {
        if self.matches(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        let index = self.parse_variable("Expected variable name.");

        match self.matches(TokenType::Equal) {
            true => self.expression(),
            false => self.emit_instruction(Instruction::Nil),
        }

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration.",
        );
        self.define_variable(index);
    }

    fn statement(&mut self) {
        if self.matches(TokenType::Print) {
            self.print_statement();
            return;
        }

        if self.matches(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
            return;
        }

        self.expression_statement();
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expected ';' after value.");
        self.emit_instruction(Instruction::Print);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expected ';' after expression.");
        self.emit_instruction(Instruction::Pop);
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

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.kind != TokenType::EoF {
            if self.previous.kind == TokenType::Semicolon {
                return;
            }
            match self.current.kind {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
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
