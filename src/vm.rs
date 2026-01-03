use std::collections::{HashMap, hash_map::Entry};

use crate::{
    ArcError, Chunk, Compiler, Instruction, Parser, Value,
    function::{Function, Native},
};

#[cfg(debug_assertions)]
use crate::Disassembler;

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

impl VM {
    const FRAMES_MAX: usize = 64;
    const STACK_MAX: usize = Self::FRAMES_MAX * Compiler::LOCAL_MAX;

    pub fn new() -> Self {
        let mut vm = Self {
            frames: Vec::with_capacity(Self::FRAMES_MAX),
            stack: Vec::with_capacity(Self::STACK_MAX),
            globals: HashMap::new(),
        };

        vm.define_native("clock", Native(Native::clock));
        vm
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), ArcError> {
        let compiler = Parser::new(source);
        match compiler.compile() {
            Ok(function) => {
                self.push_stack(Value::Function(function.clone()));
                let _ = self.call(function, 0);
                self.run()
            }
            Err(error) => Err(error),
        }
    }

    fn run(&mut self) -> Result<(), ArcError> {
        loop {
            let instruction = *self.chunk_ref().get_instruction(self.get_frame_ref().ip);

            #[cfg(debug_assertions)]
            {
                let disassembler = Disassembler::new(self.chunk_ref());

                print!("          ");
                for value in self.stack.iter() {
                    print!("[ {:?} ]", value);
                }
                println!();

                disassembler.disassemble_instruction(self.get_frame_ref().ip, &instruction);
            }

            self.get_frame().ip += 1;
            match instruction {
                Instruction::Constant(index) => {
                    let constant = self.chunk().get_constant(index).clone();
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
                    let value = self
                        .stack
                        .get(index + self.get_frame_ref().slot)
                        .unwrap()
                        .clone();
                    self.push_stack(value);
                }
                Instruction::SetLocal(index) => {
                    let top = self.peek_stack(0).clone();
                    let offset = index + self.get_frame_ref().slot;
                    let value = self.stack.get_mut(offset).unwrap();
                    *value = top;
                }
                Instruction::GetGlobal(index) => {
                    if let Value::String(name) = self.chunk_ref().get_constant(index) {
                        match self.globals.get(name) {
                            Some(value) => self.push_stack(value.clone()),
                            None => {
                                return self
                                    .runtime_error(&format!("Undefined variable '{}'.", name));
                            }
                        }
                    }
                }
                Instruction::DefineGlobal(index) => {
                    let Value::String(name) = self.chunk().get_constant(index).to_owned() else {
                        unreachable!()
                    };

                    let value = self.pop_stack();
                    self.globals.insert(name, value);
                }
                Instruction::SetGlobal(index) => {
                    let Value::String(name) = self.chunk_ref().get_constant(index).to_owned()
                    else {
                        unreachable!()
                    };
                    let value = self.peek_stack(0).to_owned();
                    match self.globals.entry(name.clone()) {
                        Entry::Vacant(_) => {
                            self.globals.remove(&name);
                            return self.runtime_error(&format!("Undefined variable '{}'.", name));
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
                            return self
                                .runtime_error("Operands must be two numbers or two strings.");
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
                        return self.runtime_error("Operand must be a number.");
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
                    self.get_frame().ip += offset;
                }
                Instruction::JumpIfFalse(offset) => {
                    if self.peek_stack(0).is_falsey() {
                        self.get_frame().ip += offset;
                    }
                }
                Instruction::Loop(offset) => {
                    self.get_frame().ip -= offset;
                }
                Instruction::Call(arg_count) => {
                    self.call_value(arg_count)?;
                }
                Instruction::Return => {
                    let frame = self.frames.pop().unwrap();
                    let result = self.pop_stack();
                    if self.frames.is_empty() {
                        self.pop_stack();
                        return Ok(());
                    }
                    self.stack.truncate(frame.slot);
                    self.push_stack(result);
                }
            }
        }
    }

    fn runtime_error(&mut self, message: &str) -> Result<(), ArcError> {
        eprintln!("{}", message);

        for frame in self.frames.iter().rev() {
            let line = frame.function.chunk.get_line(frame.ip - 1);
            eprint!("[line {}] in ", line);
            match frame.function.name.as_str() {
                "" => eprintln!("script"),
                _ => eprintln!("{}()", frame.function.name),
            }
        }
        self.clear_stack();
        Err(ArcError::RuntimeError)
    }

    fn define_native(&mut self, name: &str, native: Native) {
        let name = name.to_string();
        self.push_stack(Value::String(name.clone()));
        self.push_stack(Value::Native(native));
        self.globals.insert(
            self.peek_stack(1).try_string().unwrap().to_string(),
            self.peek_stack(0).to_owned(),
        );
        self.pop_stack();
        self.pop_stack();
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
            self.runtime_error("Operands must be numbers.")
        }
    }

    fn call_value(&mut self, arg_count: usize) -> Result<(), ArcError> {
        let callee = self.peek_stack(arg_count).to_owned();
        match callee {
            Value::Function(function) => self.call(function.clone(), arg_count),
            Value::Native(native) => {
                let pivot = self.stack.len() - arg_count - 1;
                let args = self.stack.split_off(pivot);
                let result = native.0(args.as_slice());
                self.push_stack(result);
                Ok(())
            }
            _ => self.runtime_error("Can only call functions and classes."),
        }
    }

    fn call(&mut self, function: Function, arg_count: usize) -> Result<(), ArcError> {
        if arg_count != function.arity {
            return self.runtime_error(&format!(
                "Expected {} arguments but got {}.",
                function.arity, arg_count
            ));
        }

        if self.frames.len() == Self::FRAMES_MAX {
            return self.runtime_error("Stack overflow.");
        }

        let frame = CallFrame::new(function.clone(), self.stack.len() - arg_count - 1);
        self.frames.push(frame);
        Ok(())
    }

    fn get_frame(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn get_frame_ref(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn chunk(&mut self) -> &mut Chunk {
        &mut self.get_frame().function.chunk
    }

    fn chunk_ref(&self) -> &Chunk {
        &self.get_frame_ref().function.chunk
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
        self.frames.clear();
    }
}

pub struct CallFrame {
    function: Function,
    ip: usize,
    slot: usize,
}

impl CallFrame {
    fn new(function: Function, slot: usize) -> Self {
        Self {
            function,
            ip: 0,
            slot,
        }
    }
}
