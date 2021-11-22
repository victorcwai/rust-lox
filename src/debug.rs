use crate::{
    chunk::{Chunk, OpCode},
    interner::Interner,
    value::print_value,
};

pub fn disassemble_chunk(chunk: &Chunk, name: &str, interner: &Interner) {
    println!("== {} ==", name);
    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset, interner);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize, interner: &Interner) -> usize {
    print!("{} ", offset);
    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{} ", chunk.lines[offset]);
    }

    let instruction = &chunk.code[offset];
    match instruction {
        OpCode::Constant(idx) => {
            constant_instruction("OP_CONSTANT", chunk, offset, (*idx).into(), interner)
        }
        OpCode::Nil => simple_instruction("OP_NIL", offset),
        OpCode::True => simple_instruction("OP_TRUE", offset),
        OpCode::False => simple_instruction("OP_FALSE", offset),
        OpCode::Pop => simple_instruction("OP_POP", offset),
        OpCode::DefineGlobal(idx) => {
            constant_instruction("OP_DEFINE_GLOBAL", chunk, offset, (*idx).into(), interner)
        }
        OpCode::GetGlobal(idx) => {
            constant_instruction("OP_GET_GLOBAL", chunk, offset, (*idx).into(), interner)
        }
        OpCode::SetGlobal(idx) => {
            constant_instruction("OP_SET_GLOBAL", chunk, offset, (*idx).into(), interner)
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
        OpCode::Jump(jump) => jump_instruction("OP_JUMP", chunk, offset, jump, true),
        OpCode::JumpIfFalse(jump) => {
            jump_instruction("OP_JUMP_IF_FALSE", chunk, offset, jump, true)
        }
        OpCode::Loop(jump) => jump_instruction("OP_LOOP", chunk, offset, jump, false),
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

fn jump_instruction(
    name: &str,
    chunk: &Chunk,
    offset: usize,
    jump: &usize,
    forward: bool,
) -> usize {
    let mut dest_idx = offset + jump;
    let mut signed_jump = *jump as i128;
    if !forward {
        dest_idx = offset - jump;
        signed_jump = -signed_jump;
    }

    println!(
        "{} offset:{} jump:{} -> {:?}",
        name, offset, signed_jump, chunk.code[dest_idx]
    );
    offset + 1
}

fn constant_instruction(
    name: &str,
    chunk: &Chunk,
    offset: usize,
    constant_idx: usize,
    interner: &Interner,
) -> usize {
    print!("{} {:?} '", name, constant_idx);
    print_value(&chunk.constants.values[constant_idx], interner);
    println!("'");
    offset + 1
}
