use crate::frontend::{Local, Token};
use crate::runtime::{Function, FunctionType};

pub(crate) struct Compiler<'source> {
    pub(crate) enclosing: Option<Box<Compiler<'source>>>,
    pub(crate) function: Function,
    pub(crate) function_type: FunctionType,
    pub(crate) locals: Vec<Local<'source>>,
    pub(crate) scope_depth: i32,
}

impl<'source> Compiler<'source> {
    pub(crate) const LOCAL_MAX: usize = u8::MAX as usize + 1;

    pub(crate) fn new(name: String, function_type: FunctionType) -> Box<Self> {
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

    pub(crate) fn is_same_scope(&self, name: &Token) -> u32 {
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
