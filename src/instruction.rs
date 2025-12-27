use crate::Chunk;

#[derive(Copy, Clone)]
pub enum Instruction {
    Constant(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
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
            Instruction::Add => Instruction::simple("OP_ADD"),
            Instruction::Subtract => Instruction::simple("OP_SUBTRACT"),
            Instruction::Multiply => Instruction::simple("OP_MULTIPLY"),
            Instruction::Divide => Instruction::simple("OP_DIVIDE"),
            Instruction::Negate => Instruction::simple("OP_NEGATE"),
            Instruction::Return => Instruction::simple("OP_RETURN"),
        }
    }

    fn constant(name: &str, chunk: &Chunk, index: usize) {
        let value = chunk.get_constant(index);
        println!("{:<16} {:4} '{}'", name, index, value);
    }

    fn simple(name: &str) {
        println!("{}", name);
    }
}
