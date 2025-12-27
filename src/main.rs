use arc::{Chunk, Instruction};

fn main() {
    let mut chunk = Chunk::new();
    let constant: usize = chunk.add_constant(1.2);
    chunk.write(Instruction::Constant(constant), 123);
    chunk.write(Instruction::Return, 123);
    chunk.disassemble("test chunk");
}
