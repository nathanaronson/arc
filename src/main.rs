use arc::{Chunk, Disassembler, Instruction, VM};

fn main() {
    let mut vm = VM::new();
    let mut chunk = Chunk::new();
    let index = chunk.add_constant(1.2);
    chunk.write(Instruction::Constant(index), 123);
    let index = chunk.add_constant(3.4);
    chunk.write(Instruction::Constant(index), 123);
    chunk.write(Instruction::Add, 123);
    let index = chunk.add_constant(5.6);
    chunk.write(Instruction::Constant(index), 123);
    chunk.write(Instruction::Divide, 123);
    chunk.write(Instruction::Negate, 123);
    chunk.write(Instruction::Return, 123);
    let disassembler = Disassembler::new(&chunk);
    disassembler.disassemble_chunk("test chunk");
    let _ = vm.interpret(chunk);
}
