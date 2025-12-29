use crate::{ArcError, Chunk, Compiler, Disassembler, Instruction, Value};

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

    pub fn interpret(&mut self, source: &str) -> Result<(), ArcError> {
        let mut chunk = Chunk::new();
        let mut compiler = Compiler::new(source, &mut chunk);
        if !compiler.compile() {
            return Err(ArcError::CompileError);
        }
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
                    let constant = self.chunk.get_constant(index).clone();
                    self.push_stack(constant);
                }
                Instruction::Nil => {
                    self.push_stack(Value::Nil);
                }
                Instruction::True => {
                    self.push_stack(Value::Boolean(true));
                }
                Instruction::False => {
                    self.push_stack(Value::Boolean(false));
                }
                Instruction::Equal => {
                    let (b, a) = (self.pop_stack(), self.pop_stack());
                    self.push_stack(Value::Boolean(a == b));
                }
                Instruction::Greater => {
                    self.binary_operation(|x, y| x > y, Value::Boolean)?;
                }
                Instruction::Less => {
                    self.binary_operation(|x, y| x < y, Value::Boolean)?;
                }
                Instruction::Add => {
                    self.binary_operation(|x, y| x + y, Value::Number)?;
                }
                Instruction::Subtract => {
                    self.binary_operation(|x, y| x - y, Value::Number)?;
                }
                Instruction::Multiply => {
                    self.binary_operation(|x, y| x * y, Value::Number)?;
                }
                Instruction::Divide => {
                    self.binary_operation(|x, y| x / y, Value::Number)?;
                }
                Instruction::Negate => match self.peek_stack(0) {
                    Value::Number(_) => {
                        if let Value::Number(value) = self.pop_stack() {
                            self.push_stack(Value::Number(-value));
                        }
                    }
                    _ => {
                        self.runtime_error("Operand must be a number.");
                        return Err(ArcError::RuntimeError);
                    }
                },
                Instruction::Not => {
                    let value = self.pop_stack().is_falsey();
                    self.push_stack(Value::Boolean(value));
                }
                Instruction::Return => {
                    println!("{}", self.pop_stack());
                    return Ok(());
                }
            }
        }
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{}", message);

        let line = self.chunk.get_line(self.ip - 1);
        eprintln!("[line {}] in script", line);
        self.clear_stack();
    }

    fn binary_operation<Type>(
        &mut self,
        operation: fn(f64, f64) -> Type,
        value_type: fn(Type) -> Value,
    ) -> Result<(), ArcError> {
        let top = (self.peek_stack(0), self.peek_stack(1));
        if let (Value::Number(_), Value::Number(_)) = top {
            if let (Value::Number(r_val), Value::Number(l_val)) =
                (self.pop_stack(), self.pop_stack())
            {
                self.push_stack(value_type(operation(l_val, r_val)));
            }
            Ok(())
        } else {
            self.runtime_error("Operands must be numbers.");
            Err(ArcError::RuntimeError)
        }
    }

    fn peek_stack(&self, distance: usize) -> &Value {
        &self.stack.get(self.stack.len() - distance - 1).unwrap()
    }

    fn pop_stack(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn push_stack(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn clear_stack(&mut self) {
        self.stack.clear();
    }
}
