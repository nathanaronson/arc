use crate::frontend::Token;

pub(crate) struct Local<'source> {
    pub(crate) name: Token<'source>,
    pub(crate) depth: i32,
}

impl<'source> Local<'source> {
    pub(crate) fn new(name: Token<'source>, depth: i32) -> Self {
        Self { name, depth }
    }
}
