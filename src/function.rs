use std::fmt::{Display, Formatter, Result};

use crate::Chunk;

#[derive(Clone, PartialEq)]
pub(crate) struct Function {
    arity: usize,
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

    pub(crate) fn increment_arity(&mut self) -> usize {
        self.arity += 1;
        self.arity
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "<fn {}>", self.name)
    }
}

pub(crate) enum FunctionType {
    Function,
    Script,
}
