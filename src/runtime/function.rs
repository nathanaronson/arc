use crate::bytecode::Chunk;
use std::fmt::{Display, Formatter, Result};

pub(crate) enum FunctionType {
    Function,
    Script,
}

#[derive(Clone, PartialEq)]
pub(crate) struct Function {
    pub(crate) arity: usize,
    pub(crate) chunk: Chunk,
    pub(crate) name: String,
}

impl Function {
    pub(crate) const MAX_PARAMS: usize = 255;

    pub(crate) fn new(name: String) -> Self {
        Self {
            arity: 0,
            chunk: Chunk::new(),
            name,
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "<fn {}>", self.name)
    }
}
