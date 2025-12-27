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
