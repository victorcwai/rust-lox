use std::convert::TryInto;

use chunk::{Chunk, OpCode};
use crate::debug::disassemble_chunk;

mod chunk;
mod debug;
mod value;
fn main() {
    let mut c = Chunk::new();

    // add the constant value itself to the chunk’s constant pool
    let constant = c.add_constant(1.2);
    c.write(OpCode::OpConstant(constant.try_into().unwrap()), 123);

    c.write(OpCode::OpReturn, 123);

    disassemble_chunk(&c, "test chunk");

  }
