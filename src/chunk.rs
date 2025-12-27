use crate::{Instruction, Value};

pub struct Chunk {
    code: Vec<Instruction>,
    lines: Vec<u32>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn write(&mut self, instruction: Instruction, line: u32) {
        self.code.push(instruction);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub(crate) fn get_constant(&self, index: usize) -> &Value {
        &self.constants[index]
    }

    pub(crate) fn get_line(&self, offset: usize) -> u32 {
        self.lines[offset]
    }

    pub(crate) fn get_instruction(&self, offset: usize) -> &Instruction {
        &self.code[offset]
    }

    pub(crate) fn get_code(&self) -> &[Instruction] {
        &self.code
    }
}
