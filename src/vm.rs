use std::collections::{HashMap, hash_map::Entry};

use crate::{ArcError, Chunk, Instruction, Parser, Value};

#[cfg(debug_assertions)]
use crate::Disassembler;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

impl VM {
    const STACK_MAX: usize = 256;

    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::with_capacity(Self::STACK_MAX),
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), ArcError> {
        let mut chunk = Chunk::new();
        let mut compiler = Parser::new(source, &mut chunk);
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
                Instruction::Pop => {
                    self.pop_stack();
                }
                Instruction::GetLocal(index) => {
                    let value = self.stack.get(index).unwrap().clone();
                    self.push_stack(value);
                }
                Instruction::SetLocal(index) => {
                    let top = self.peek_stack(0).clone();
                    let value = self.stack.get_mut(index).unwrap();
                    *value = top;
                }
                Instruction::GetGlobal(index) => {
                    if let Value::String(name) = self.chunk.get_constant(index) {
                        match self.globals.get(name) {
                            Some(value) => self.push_stack(value.clone()),
                            None => {
                                self.runtime_error(&format!("Undefined variable '{}'.", name));
                                return Err(ArcError::RuntimeError);
                            }
                        }
                    }
                }
                Instruction::DefineGlobal(index) => {
                    let Value::String(name) = self.chunk.get_constant(index).to_owned() else {
                        unreachable!()
                    };

                    let value = self.pop_stack();
                    self.globals.insert(name, value);
                }
                Instruction::SetGlobal(index) => {
                    let Value::String(name) = self.chunk.get_constant(index) else {
                        unreachable!()
                    };
                    let value = self.peek_stack(0).to_owned();
                    match self.globals.entry(name.clone()) {
                        Entry::Vacant(_) => {
                            self.globals.remove(name);
                            self.runtime_error(&format!("Undefined variable '{}'.", name));
                            return Err(ArcError::RuntimeError);
                        }
                        Entry::Occupied(mut e) => {
                            e.insert(value);
                        }
                    }
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
                    let (b, a) = (self.peek_stack(1), self.peek_stack(0));
                    match (b, a) {
                        (Value::String(_), Value::String(_)) => {
                            if let (Value::String(b_val), Value::String(mut a_val)) =
                                (self.pop_stack(), self.pop_stack())
                            {
                                a_val.push_str(&b_val);
                                self.push_stack(Value::String(a_val));
                            }
                        }
                        (Value::Number(_), Value::Number(_)) => {
                            self.binary_operation(|x, y| x + y, Value::Number)?
                        }
                        _ => {
                            self.runtime_error("Operands must be two numbers or two strings.");
                            return Err(ArcError::RuntimeError);
                        }
                    }
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
                Instruction::Print => {
                    println!("{}", self.pop_stack());
                }
                Instruction::Jump(offset) => {
                    self.ip += offset;
                }
                Instruction::JumpIfFalse(offset) => {
                    if self.peek_stack(0).is_falsey() {
                        self.ip += offset;
                    }
                }
                Instruction::Return => return Ok(()),
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
        self.stack.get(self.stack.len() - distance - 1).unwrap()
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
