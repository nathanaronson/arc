use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Clone, PartialEq)]
pub(crate) enum Value {
    Boolean(bool),
    Number(f64),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Boolean(value) => write!(f, "{}", value),
            Self::Number(value) => write!(f, "{}", value),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Boolean(value) => write!(f, "Boolean({})", value),
            Self::Number(value) => write!(f, "Number({})", value),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl Value {
    pub(crate) fn is_falsey(&self) -> bool {
        matches!(self, Self::Nil | Self::Boolean(false))
    }
}
