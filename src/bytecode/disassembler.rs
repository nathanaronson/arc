use crate::bytecode::{Chunk, Instruction};

pub(crate) struct Disassembler<'vm> {
    chunk: &'vm Chunk,
}

impl<'vm> Disassembler<'vm> {
    pub(crate) fn new(chunk: &'vm Chunk) -> Self {
        Disassembler { chunk }
    }

    pub(crate) fn disassemble_chunk(&self, name: &str) {
        println!("== BEGIN {} ==", name);

        for (offset, instruction) in self.chunk.code.iter().enumerate() {
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
            Instruction::Nil => self.simple_instruction("OP_NIL"),
            Instruction::True => self.simple_instruction("OP_TRUE"),
            Instruction::False => self.simple_instruction("OP_FALSE"),
            Instruction::Pop => self.simple_instruction("OP_POP"),
            Instruction::GetLocal(index) => self.byte_instruction("OP_GET_LOCAL", *index),
            Instruction::SetLocal(index) => self.byte_instruction("OP_SET_LOCAL", *index),
            Instruction::GetGlobal(index) => self.constant_instruction("OP_GET_GLOBAL", *index),
            Instruction::DefineGlobal(index) => {
                self.constant_instruction("OP_DEFINE_GLOBAL", *index)
            }
            Instruction::SetGlobal(index) => self.constant_instruction("OP_SET_GLOBAL", *index),
            Instruction::Equal => self.simple_instruction("OP_EQUAL"),
            Instruction::Greater => self.simple_instruction("OP_GREATER"),
            Instruction::Less => self.simple_instruction("OP_LESS"),
            Instruction::Add => self.simple_instruction("OP_ADD"),
            Instruction::Subtract => self.simple_instruction("OP_SUBTRACT"),
            Instruction::Multiply => self.simple_instruction("OP_MULTIPLY"),
            Instruction::Divide => self.simple_instruction("OP_DIVIDE"),
            Instruction::Not => self.simple_instruction("OP_NOT"),
            Instruction::Negate => self.simple_instruction("OP_NEGATE"),
            Instruction::Print => self.simple_instruction("OP_PRINT"),
            Instruction::Jump(jump) => self.jump_instruction("OP_JUMP", offset, *jump, 1),
            Instruction::JumpIfFalse(jump) => {
                self.jump_instruction("OP_JUMPIFFALSE", offset, *jump, 1)
            }
            Instruction::Loop(jump) => {
                self.jump_instruction("OP_LOOP", offset, *jump, -1);
            }
            Instruction::Call(arg_count) => {
                self.byte_instruction("OP_CALL", *arg_count);
            }
            Instruction::Return => self.simple_instruction("OP_RETURN"),
        }
    }

    fn constant_instruction(&self, name: &str, index: usize) {
        let value = self.chunk.get_constant(index);
        println!("{:<16} {:4} '{}'", name, index, value);
    }

    fn byte_instruction(&self, name: &str, index: usize) {
        println!("{:<16} {:4}", name, index);
    }

    fn jump_instruction(&self, name: &str, offset: usize, jump: usize, sign: i8) {
        let val = offset as i8 + jump as i8 * sign + 1;
        println!("{:<16} {:4} -> {}", name, offset, val);
    }

    fn simple_instruction(&self, name: &str) {
        println!("{}", name);
    }
}
