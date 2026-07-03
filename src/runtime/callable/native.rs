use crate::runtime::{ArcError, Value};
use std::fmt::{self, Display, Formatter};
#[cfg(not(target_arch = "wasm32"))]
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
    pub(crate) fn print() -> Self {
        Self {
            function: Self::print_impl,
            arity: 1,
        }
    }

    fn print_impl(args: &[Value]) -> Result<Value, ArcError> {
        if args.len() != 2 {
            Err(ArcError::RuntimeError)
        } else {
            let value = args.last().unwrap();
            crate::output::out(&format!("{}\n", value));
            Ok(Value::Nil)
        }
    }

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
            Ok(Value::Number(Self::now_millis()))
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn now_millis() -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64
    }

    #[cfg(target_arch = "wasm32")]
    fn now_millis() -> f64 {
        js_sys::Date::now()
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
                Value::Closure(_) => Ok(Value::String("closure".to_string())),
                Value::Function(_) => Ok(Value::String("function".to_string())),
                Value::Native(_) => Ok(Value::String("native".to_string())),
            }
        }
    }
}
