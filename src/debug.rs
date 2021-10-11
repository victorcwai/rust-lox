use crate::{chunk::{Chunk, OpCode}, value::print_value};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    print!("== {} ==\n", name);
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
        OpCode::OpReturn => return simple_instruction("OP_RETURN", offset),
        OpCode::OpConstant(idx) => return constant_instruction("OP_CONSTANT", chunk, offset, (*idx).into()),  
        _ => {
            println!("Unknown opcode {:?}\n", instruction);
            return offset + 1;
        }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
  print!("{}\n", name);
  return offset + 1;
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize, constant_idx: usize) -> usize {
  print!("{} {:?} '", name, constant_idx);
  print_value(&chunk.constants.values[constant_idx]);
  print!("'\n");
  return offset + 1;
}
