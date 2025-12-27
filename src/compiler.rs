use crate::Scanner;
use crate::token::TokenType;

pub(crate) struct Compiler {}

impl Compiler {
    pub(crate) fn compile(source: &str) {
        let mut scanner = Scanner::new(source);
        let mut current_line = None;
        loop {
            let token = scanner.scan_token();

            match current_line {
                Some(line) if line == token.line => print!("   | "),
                _ => print!("{:4} ", token.line),
            }
            current_line = Some(token.line);

            println!("{:?} '{}'", token.kind, token.lexeme);

            if matches!(token.kind, TokenType::EoF) {
                break;
            }
        }
    }
}
