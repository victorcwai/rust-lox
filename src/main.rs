use std::convert::TryInto;
use std::time::Instant;

use crate::debug::disassemble_chunk;
use chunk::{Chunk, OpCode};
use vm::VM;

mod chunk;
mod debug;
mod value;
mod vm;

fn main() {
    let now = Instant::now();

    let mut vm = VM::new();
    let mut c = Chunk::new();

    // add the constant value itself to the chunk’s constant pool
    let constant = c.add_constant(1.2);
    c.write(OpCode::OpConstant(constant.try_into().unwrap()), 123);

    let constant = c.add_constant(3.4);
    c.write(OpCode::OpConstant(constant.try_into().unwrap()), 123);

    c.write(OpCode::OpAdd, 123);

    let constant = c.add_constant(5.6);
    c.write(OpCode::OpConstant(constant.try_into().unwrap()), 123);

    c.write(OpCode::OpDivide, 123);
    c.write(OpCode::OpNegate, 123);
    c.write(OpCode::OpReturn, 123);

    disassemble_chunk(&c, "test chunk");

    vm.interpret(c);
    // vm.debug_trace_execution();
    // freeVm();
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
