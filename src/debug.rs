use crate::{
    chunk::{Chunk, OpCode},
    value::print_value,
};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);
    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{} ", offset);
    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{} ", chunk.lines[offset]);
    }

    let instruction = &chunk.code[offset];
    match instruction {
        OpCode::Constant(idx) => constant_instruction("OP_CONSTANT", chunk, offset, (*idx).into()),
        OpCode::Nil => simple_instruction("OP_NIL", offset),
        OpCode::True => simple_instruction("OP_TRUE", offset),
        OpCode::False => simple_instruction("OP_FALSE", offset),
        OpCode::Pop => simple_instruction("OP_POP", offset),
        OpCode::DefineGlobal(idx) => {
            constant_instruction("OP_DEFINE_GLOBAL", chunk, offset, (*idx).into())
        }
        OpCode::GetGlobal(idx) => {
            constant_instruction("OP_GET_GLOBAL", chunk, offset, (*idx).into())
        }
        OpCode::SetGlobal(idx) => {
            constant_instruction("OP_SET_GLOBAL", chunk, offset, (*idx).into())
        }
        OpCode::GetLocal(idx) => byte_instruction("OP_GET_LOCAL", offset, (*idx).into()),
        OpCode::SetLocal(idx) => byte_instruction("OP_SET_LOCAL", offset, (*idx).into()),
        OpCode::Equal => simple_instruction("OP_EQUAL", offset),
        OpCode::Greater => simple_instruction("OP_GREATER", offset),
        OpCode::Less => simple_instruction("OP_LESS", offset),
        OpCode::Add => simple_instruction("OP_ADD", offset),
        OpCode::Subtract => simple_instruction("OP_SUBTRACT", offset),
        OpCode::Multiply => simple_instruction("OP_MULTIPLY", offset),
        OpCode::Divide => simple_instruction("OP_DIVIDE", offset),
        OpCode::Not => simple_instruction("OP_NOT", offset),
        OpCode::Negate => simple_instruction("OP_NEGATE", offset),
        OpCode::Print => simple_instruction("OP_PRINT", offset),
        OpCode::Return => simple_instruction("OP_RETURN", offset),
        // _ => {
        //     println!("Unknown opcode {:?}\n", instruction);
        //     offset + 1
        // }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{}", name);
    offset + 1
}

fn byte_instruction(name: &str, offset: usize, constant_idx: usize) -> usize {
    println!("{} {:?} '", name, constant_idx);
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize, constant_idx: usize) -> usize {
    print!("{} {:?} '", name, constant_idx);
    print_value(&chunk.constants.values[constant_idx], &chunk.interner);
    println!("'");
    offset + 1
}
