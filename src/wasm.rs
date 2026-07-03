use crate::{VM, output};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RunResult {
    ok: bool,
    stdout: String,
    stderr: String,
}

#[wasm_bindgen]
impl RunResult {
    #[wasm_bindgen(getter)]
    pub fn ok(&self) -> bool {
        self.ok
    }

    #[wasm_bindgen(getter)]
    pub fn stdout(&self) -> String {
        self.stdout.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn stderr(&self) -> String {
        self.stderr.clone()
    }
}

/// Compiles and runs an Arc program in a fresh VM, returning captured output.
#[wasm_bindgen]
pub fn run(source: &str) -> RunResult {
    output::start_capture();
    let mut vm = VM::new();
    let ok = vm.interpret(source).is_ok();
    let (stdout, stderr) = output::take_capture();
    RunResult { ok, stdout, stderr }
}
