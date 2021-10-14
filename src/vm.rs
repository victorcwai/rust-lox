use crate::{
    chunk::{Chunk, OpCode},
    debug::disassemble_instruction,
    value::{print_value, Value},
};

const STACK_SIZE: usize = 256;

pub struct VM {
    pub chunk: Chunk, // TODO: use &?
    pub ip: usize,
    pub stack: Vec<Value>,
}

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

impl VM {
    pub fn new() -> VM {
        VM {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::with_capacity(STACK_SIZE), // = reset stack
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        self.ip = 0;
        return self.run();
    }

    // We run every single instruction here, so this is the most performance critical part of the VM.
    // TODO: look up “direct threaded code”, “jump table”, and “computed goto” for optimization techniques
    fn run(&mut self) -> InterpretResult {
        loop {
            let op = &self.chunk.code[self.ip];
            match op {
                OpCode::OpConstant(cons) => {
                    let constant = &self.chunk.constants.values[*cons as usize];
                    print_value(constant);
                    self.stack.push(*constant);
                    print!("\n");
                }
                OpCode::OpAdd => {
                    self.binary_op(|x, y| x + y);
                }
                OpCode::OpSubtract => {
                    self.binary_op(|x, y| x - y);
                }
                OpCode::OpMultiply => {
                    self.binary_op(|x, y| x * y);
                }
                OpCode::OpDivide => {
                    self.binary_op(|x, y| x / y);
                }
                OpCode::OpNegate => {
                    let neg_val = -self.pop();
                    self.stack.push(neg_val);
                    // if let Some(last) = self.stack.last_mut() {
                    //     *last = -*last
                    // }
                }
                OpCode::OpReturn => {
                    print_value(&self.pop());
                    print!("\n");
                    return InterpretResult::InterpretOk;
                }
            }
            self.ip += 1;
        }
    }

    // helper function for popping stack
    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Empty stack")
    }

    fn binary_op(&mut self, f: fn(f64, f64) -> f64) {
        let b = self.pop(); // note: the first pop returns the right operand
        let a = self.pop();
        self.stack.push(f(a, b))
    }

    pub fn debug_trace_execution(&self) {
        print!("          ");
        for slot in &self.stack {
            print!("[ ");
            print_value(&slot);
            print!(" ]");
        }
        print!("\n");
        disassemble_instruction(&self.chunk, self.ip);
    }
}
