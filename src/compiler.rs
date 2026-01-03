use crate::{
    ArcError, Chunk, Function, FunctionType, Instruction, Scanner, Token, TokenType, Value,
};
use std::collections::HashMap;
use std::mem;

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

pub(crate) struct Compiler<'source> {
    enclosing: Option<Box<Compiler<'source>>>,
    function: Function,
    function_type: FunctionType,
    locals: Vec<Local<'source>>,
    scope_depth: i32,
}

impl<'source> Compiler<'source> {
    pub(crate) const LOCAL_MAX: usize = u8::MAX as usize + 1;

    fn new(name: String, function_type: FunctionType) -> Box<Self> {
        let mut locals = Vec::with_capacity(Self::LOCAL_MAX);
        locals.push(Local::new(Token::null(), 0));
        Box::new(Self {
            enclosing: None,
            function: Function::new(name),
            function_type,
            locals,
            scope_depth: 0,
        })
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
    compiler: Box<Compiler<'source>>,
    had_error: bool,
    panic_mode: bool,
}

impl<'source> Parser<'source> {
    const JUMP_PLACEHOLDER: usize = u16::MAX as usize;

    pub(crate) fn new(source: &'source str) -> Self {
        Self {
            scanner: Scanner::new(source),
            rules: Parser::build_rules(),
            previous: Token::null(),
            current: Token::null(),
            compiler: Compiler::new("".to_string(), FunctionType::Script),
            had_error: false,
            panic_mode: false,
        }
    }

    fn build_rules() -> HashMap<TokenType, ParseRule<'source>> {
        let mut rules = HashMap::new();

        // Literals / grouping.
        rule!(
            rules,
            LeftParen,
            Some(Parser::grouping),
            Some(Parser::call),
            Call
        );
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
        rule!(rules, And, None, Some(Parser::and), And);
        rule!(rules, Class, None, None, None);
        rule!(rules, Else, None, None, None);
        rule!(rules, For, None, None, None);
        rule!(rules, Fun, None, None, None);
        rule!(rules, If, None, None, None);
        rule!(rules, Or, None, Some(Parser::or), Or);
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

    fn current_chunk(&self) -> &Chunk {
        &self.compiler.function.chunk
    }

    fn current_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.compiler.function.chunk
    }

    pub(crate) fn compile(mut self) -> Result<Function, ArcError> {
        self.advance();
        while !self.matches(TokenType::EoF) {
            self.declaration();
        }
        self.consume(TokenType::EoF, "Expected end of expression.");
        match self.had_error {
            true => Err(ArcError::CompileError),
            false => Ok(self.end_compiler()),
        }
    }

    fn advance(&mut self) {
        self.previous = mem::replace(&mut self.current, self.scanner.scan_token());

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
        let line = self.previous.line;
        self.current_chunk_mut().write(instruction, line);
    }

    fn emit_instructions(&mut self, instruction_1: Instruction, instruction_2: Instruction) {
        self.emit_instruction(instruction_1);
        self.emit_instruction(instruction_2);
    }

    fn end_compiler(&mut self) -> Function {
        self.emit_return();
        let function = self.compiler.function.clone();
        #[cfg(debug_assertions)]
        {
            if !self.had_error {
                let disassembler = Disassembler::new(self.current_chunk());
                let name = match &function.name {
                    s if s.is_empty() => "<script>",
                    _ => &self.compiler.function.name,
                };
                disassembler.disassemble_chunk(name);
            }
        }
        self.pop_compiler();
        function
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

    fn call(&mut self, _can_assign: bool) {
        let arg_count = self.argument_list();
        self.emit_instruction(Instruction::Call(arg_count));
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

    fn argument_list(&mut self) -> usize {
        let mut arg_count = 0;
        if !self.check(TokenType::RightParen) {
            loop {
                self.expression();
                if arg_count == Function::MAX_PARAMS {
                    self.error("Can't have more than 255 arguments.");
                }
                arg_count += 1;
                if !self.matches(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expected ')' after arguments.");
        arg_count
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(true);
        self.emit_instruction(Instruction::Pop);
        self.parse_precedence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or(&mut self, _can_assign: bool) {
        let else_jump = self.emit_jump(true);
        let end_jump = self.emit_jump(false);
        self.patch_jump(else_jump);
        self.emit_instruction(Instruction::Pop);
        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn mark_initialized(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }
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

    fn init_compiler(&mut self, name: String, function_type: FunctionType) {
        let new = Compiler::new(name, function_type);
        let old = mem::replace(&mut self.compiler, new);
        self.compiler.enclosing = Some(old);
    }

    fn pop_compiler(&mut self) {
        if let Some(enclosing) = self.compiler.enclosing.take() {
            let _ = mem::replace(&mut self.compiler, enclosing);
        }
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

    fn function(&mut self, function_type: FunctionType) {
        self.init_compiler(self.previous.lexeme.to_owned(), function_type);
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expected '(' after function name.");
        if !self.check(TokenType::RightParen) {
            loop {
                if self.compiler.function.increment_arity() > Function::MAX_PARAMS {
                    self.error_at_current(&format!(
                        "Can't have more than {} parameters.",
                        Function::MAX_PARAMS
                    ));
                }
                let constant = self.parse_variable("Expected parameter name.");
                self.define_variable(constant);
                if !self.matches(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expected ')' after parameters.");
        self.consume(TokenType::LeftBrace, "Expected '{' before function body.");
        self.block();
        let function = self.end_compiler();
        let index = self.make_constant(Value::Function(function));
        self.emit_instruction(Instruction::Constant(index));
    }

    fn declaration(&mut self) {
        if self.matches(TokenType::Fun) {
            self.fun_declaration();
        } else if self.matches(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn fun_declaration(&mut self) {
        let index = self.parse_variable("Expected function name.");
        self.mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(index);
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

        if self.matches(TokenType::Return) {
            self.return_statement();
            return;
        }

        if self.matches(TokenType::If) {
            self.if_statement();
            return;
        }

        if self.matches(TokenType::While) {
            self.while_statement();
            return;
        }

        if self.matches(TokenType::For) {
            self.for_statement();
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

    fn return_statement(&mut self) {
        if matches!(self.compiler.function_type, FunctionType::Script) {
            self.error("Can't return from top-level code.");
        }

        if self.matches(TokenType::Semicolon) {
            self.emit_return();
        } else {
            self.expression();
            self.consume(TokenType::Semicolon, "Expected ';' after expression.");
            self.emit_instruction(Instruction::Return);
        }
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expected ')' after condition.");

        let then_jump = self.emit_jump(true);
        self.emit_instruction(Instruction::Pop);
        self.statement();
        let else_jump = self.emit_jump(false);
        self.patch_jump(then_jump);
        self.emit_instruction(Instruction::Pop);
        if self.matches(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk_mut().get_count();
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expected ')' after condition.");

        let exit_jump = self.emit_jump(true);
        self.emit_instruction(Instruction::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_instruction(Instruction::Pop);
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'.");

        if self.matches(TokenType::Semicolon) {
        } else if self.matches(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.current_chunk_mut().get_count();
        let mut exit_jump = None;

        if !self.matches(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expected ';'.");

            exit_jump = Some(self.emit_jump(true));
            self.emit_instruction(Instruction::Pop);
        }

        if !self.matches(TokenType::RightParen) {
            let body_jump = self.emit_jump(false);
            let increment_start = self.current_chunk_mut().get_count();
            self.expression();
            self.emit_instruction(Instruction::Pop);
            self.consume(TokenType::RightParen, "Expected ')' after 'for' clauses.");
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }
        self.statement();
        self.emit_loop(loop_start);
        if let Some(value) = exit_jump {
            self.patch_jump(value);
            self.emit_instruction(Instruction::Pop);
        }
        self.end_scope();
    }

    fn emit_jump(&mut self, if_false: bool) -> usize {
        match if_false {
            true => self.emit_instruction(Instruction::JumpIfFalse(Self::JUMP_PLACEHOLDER)),
            false => self.emit_instruction(Instruction::Jump(Self::JUMP_PLACEHOLDER)),
        }
        self.current_chunk_mut().get_count() - 1
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.current_chunk_mut().get_count() - loop_start + 1;
        let offset = self.clip_u16(offset, "Loop body too large.");
        self.emit_instruction(Instruction::Loop(offset));
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expected ';' after expression.");
        self.emit_instruction(Instruction::Pop);
    }

    fn emit_return(&mut self) {
        self.emit_instructions(Instruction::Nil, Instruction::Return);
    }

    fn make_constant(&mut self, value: Value) -> usize {
        let index = self.current_chunk_mut().add_constant(value);
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

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_chunk_mut().get_count() - offset - 1;
        let jump = self.clip_u16(jump, "Too much code to jump over.");

        match self.current_chunk_mut().get_instruction_mut(offset) {
            Instruction::Jump(o) | Instruction::JumpIfFalse(o) => *o = jump,
            _ => unreachable!(),
        }
    }

    fn clip_u16(&mut self, value: usize, message: &str) -> usize {
        match u16::try_from(value) {
            Ok(_) => value,
            Err(_) => {
                self.error(message);
                Self::JUMP_PLACEHOLDER
            }
        }
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
