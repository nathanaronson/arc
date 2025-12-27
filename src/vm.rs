use crate::{ArcError, Chunk, Disassembler, Instruction, Value};

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::with_capacity(STACK_MAX),
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<(), ArcError> {
        self.chunk = chunk;
        self.ip = 0;
        self.run()
    }

    fn run(&mut self) -> Result<(), ArcError> {
        loop {
            let instruction = *self.chunk.get_instruction(self.ip);

            #[cfg(debug_assertions)]
            {
                let disassembler = Disassembler::new(&self.chunk);

                print!("          ");
                for value in self.stack.iter() {
                    print!("[ {:?} ]", value);
                }
                println!();

                disassembler.disassemble_instruction(self.ip, &instruction);
            }

            self.ip += 1;
            match instruction {
                Instruction::Constant(index) => {
                    let constant = *self.chunk.get_constant(index);
                    self.stack.push(constant);
                }
                Instruction::Add => {
                    self.binary_operation(|x, y| x + y);
                }
                Instruction::Subtract => {
                    self.binary_operation(|x, y| x - y);
                }
                Instruction::Multiply => {
                    self.binary_operation(|x, y| x * y);
                }
                Instruction::Divide => {
                    self.binary_operation(|x, y| x / y);
                }
                Instruction::Negate => {
                    let value = -self.stack.pop().unwrap();
                    self.stack.push(value);
                }
                Instruction::Return => {
                    println!("{}", self.stack.pop().unwrap());
                    return Ok(());
                }
            }
        }
    }

    fn binary_operation<F: Fn(Value, Value) -> Value>(&mut self, operation: F) {
        let (right, left) = (self.stack.pop().unwrap(), self.stack.pop().unwrap());
        self.stack.push(operation(left, right));
    }
}
