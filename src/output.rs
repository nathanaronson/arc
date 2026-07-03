//! Routes interpreter output either to the process's stdout/stderr (default)
//! or into a thread-local capture buffer (used by the wasm playground).

use std::cell::RefCell;

thread_local! {
    static CAPTURE: RefCell<Option<(String, String)>> = const { RefCell::new(None) };
}

#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn start_capture() {
    CAPTURE.with(|capture| *capture.borrow_mut() = Some((String::new(), String::new())));
}

#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn take_capture() -> (String, String) {
    CAPTURE
        .with(|capture| capture.borrow_mut().take())
        .unwrap_or_default()
}

pub(crate) fn out(text: &str) {
    CAPTURE.with(|capture| match capture.borrow_mut().as_mut() {
        Some((out, _)) => out.push_str(text),
        None => print!("{}", text),
    });
}

pub(crate) fn err(text: &str) {
    CAPTURE.with(|capture| match capture.borrow_mut().as_mut() {
        Some((_, err)) => err.push_str(text),
        None => eprint!("{}", text),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VM;

    #[test]
    fn captures_print_output() {
        start_capture();
        let mut vm = VM::new();
        vm.interpret("print(\"hello\");").unwrap();
        let (out, err) = take_capture();
        assert_eq!(out, "hello\n");
        assert_eq!(err, "");
    }

    #[test]
    fn captures_runtime_error_output() {
        start_capture();
        let mut vm = VM::new();
        assert!(vm.interpret("1 + true;").is_err());
        let (_, err) = take_capture();
        assert!(err.contains("Operands must be two numbers or two strings."));
    }

    #[test]
    fn captures_compile_error_output() {
        start_capture();
        let mut vm = VM::new();
        assert!(vm.interpret("var 1 = 2;").is_err());
        let (_, err) = take_capture();
        assert!(err.contains("Error"));
    }
}
