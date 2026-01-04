use crate::runtime::Function;

pub(crate) struct CallFrame {
    pub(crate) function: Function,
    pub(crate) ip: usize,
    pub(crate) slot: usize,
}

impl CallFrame {
    pub(crate) fn new(function: Function, slot: usize) -> Self {
        Self {
            function,
            ip: 0,
            slot,
        }
    }
}
