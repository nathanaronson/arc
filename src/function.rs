use std::{
    fmt::{Display, Formatter, Result},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{Chunk, Value};

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

#[derive(Clone)]
pub(crate) struct Native(pub(crate) fn(&[Value]) -> Value);

impl PartialEq for Native {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Display for Native {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "<native fn>")
    }
}

impl Native {
    pub(crate) fn clock(_args: &[Value]) -> Value {
        Value::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as f64,
        )
    }
}
