use crate::{Chunk, Value};

pub enum Instruction {
    Constant(usize),
    Return,
}

impl Instruction {
    pub(crate) fn disassemble(&self, chunk: &Chunk, offset: usize) {
        print!("{:04} ", offset);

        let line = chunk.get_line(offset);

        if offset > 0 && line == chunk.get_line(offset - 1) {
            print!("   | ");
        } else {
            print!("{:4} ", line);
        }

        match self {
            Instruction::Constant(index) => Instruction::constant("OP_CONSTANT", chunk, *index),
            Instruction::Return => Instruction::simple("OP_RETURN"),
        }
    }

    fn constant(name: &str, chunk: &Chunk, index: usize) {
        let value: &Value = chunk.get_constant(index);
        println!("{:<16} {:4} '{}'", name, index, value);
    }

    fn simple(name: &str) {
        print!("{}", name);
    }
}
