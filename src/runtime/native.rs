use crate::runtime::{ArcError, Value};
use std::fmt::{self, Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub(crate) struct Native {
    pub(crate) function: fn(&[Value]) -> Result<Value, ArcError>,
    pub(crate) arity: usize,
}

impl PartialEq for Native {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Display for Native {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn>")
    }
}

impl Native {
    pub(crate) fn clock() -> Self {
        Self {
            function: Self::clock_impl,
            arity: 0,
        }
    }

    fn clock_impl(args: &[Value]) -> Result<Value, ArcError> {
        if args.len() != 1 {
            Err(ArcError::RuntimeError)
        } else {
            Ok(Value::Number(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as f64,
            ))
        }
    }

    pub(crate) fn type_of() -> Self {
        Self {
            function: Self::type_of_impl,
            arity: 1,
        }
    }

    fn type_of_impl(args: &[Value]) -> Result<Value, ArcError> {
        if args.len() != 2 {
            Err(ArcError::RuntimeError)
        } else {
            match args.last().unwrap() {
                Value::Boolean(_) => Ok(Value::String("boolean".to_string())),
                Value::Number(_) => Ok(Value::String("number".to_string())),
                Value::Nil => Ok(Value::String("nil".to_string())),
                Value::String(_) => Ok(Value::String("string".to_string())),
                Value::Function(_) => Ok(Value::String("function".to_string())),
                Value::Native(_) => Ok(Value::String("native".to_string())),
            }
        }
    }
}
