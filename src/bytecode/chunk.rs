use crate::bytecode::Instruction;
use crate::runtime::Value;

#[derive(Clone, PartialEq)]
pub(crate) struct Chunk {
    pub(crate) code: Vec<Instruction>,
    pub(crate) lines: Vec<u32>,
    pub(crate) constants: Vec<Value>,
}

impl Chunk {
    pub(crate) fn new() -> Self {
        Self {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub(crate) fn write(&mut self, instruction: Instruction, line: u32) {
        self.code.push(instruction);
        self.lines.push(line);
    }

    pub(crate) fn add_constant(&mut self, value: Value) -> usize {
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

    pub(crate) fn get_instruction_mut(&mut self, offset: usize) -> &mut Instruction {
        self.code.get_mut(offset).unwrap()
    }

    pub(crate) fn get_count(&self) -> usize {
        self.code.len()
    }
}
