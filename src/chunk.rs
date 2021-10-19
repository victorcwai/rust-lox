use crate::value::{Value, ValueArray};

#[derive(Debug)]
pub enum OpCode {
    Constant(u8), // u8 = constant_idx
    Nil,
    True,
    False,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Return,
}

pub struct Chunk {
    // Vec is already a dynamic array, also see:
    // https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation
    // When count > capacity, capacity will be doubled (as of today's rust vec implementation)
    // https://github.com/rust-lang/rust/blob/68dfa07e3bbbfe9100a9b1047c274717bdf452a1/library/alloc/src/raw_vec.rs#L422
    pub code: Vec<OpCode>,
    pub constants: ValueArray,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: ValueArray::new(),
            lines: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: OpCode, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, v: Value) -> usize {
        self.constants.write(v);
        self.constants.values.len() - 1
    }
}
