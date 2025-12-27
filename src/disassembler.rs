use crate::{Chunk, Instruction};

pub struct Disassembler<'vm> {
    chunk: &'vm Chunk,
}

impl<'vm> Disassembler<'vm> {
    pub fn new(chunk: &'vm Chunk) -> Self {
        Disassembler { chunk }
    }

    pub fn disassemble_chunk(&self, name: &str) {
        println!("== BEGIN {} ==", name);

        for (offset, instruction) in self.chunk.get_code().iter().enumerate() {
            self.disassemble_instruction(offset, instruction);
        }

        println!("== END {}", name);
    }

    pub(crate) fn disassemble_instruction(&self, offset: usize, instruction: &Instruction) {
        print!("{:04} ", offset);

        let line = self.chunk.get_line(offset);

        if offset > 0 && line == self.chunk.get_line(offset - 1) {
            print!("   | ");
        } else {
            print!("{:4} ", line);
        }

        match instruction {
            Instruction::Constant(index) => self.constant_instruction("OP_CONSTANT", *index),
            Instruction::Add => self.simple_instruction("OP_ADD"),
            Instruction::Subtract => self.simple_instruction("OP_SUBTRACT"),
            Instruction::Multiply => self.simple_instruction("OP_MULTIPLY"),
            Instruction::Divide => self.simple_instruction("OP_DIVIDE"),
            Instruction::Negate => self.simple_instruction("OP_NEGATE"),
            Instruction::Return => self.simple_instruction("OP_RETURN"),
        }
    }

    fn constant_instruction(&self, name: &str, index: usize) {
        let value = self.chunk.get_constant(index);
        println!("{:<16} {:4} '{}'", name, index, value);
    }

    fn simple_instruction(&self, name: &str) {
        println!("{}", name);
    }
}
