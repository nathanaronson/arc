use crate::runtime::Function;

#[derive(Clone, PartialEq)]
pub(crate) struct Closure {
    pub(crate) function: Function,
}

impl Closure {
    pub(crate) fn new(function: Function) -> Self {
        Self { function }
    }
}
