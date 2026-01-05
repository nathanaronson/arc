use crate::runtime::Closure;

pub(crate) struct CallFrame {
    pub(crate) closure: Closure,
    pub(crate) ip: usize,
    pub(crate) slot: usize,
}

impl CallFrame {
    pub(crate) fn new(closure: Closure, slot: usize) -> Self {
        Self {
            closure,
            ip: 0,
            slot,
        }
    }
}
