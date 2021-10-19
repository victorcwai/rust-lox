use crate::compiler::Parser;
use crate::{
    chunk::{Chunk, OpCode},
    debug::disassemble_instruction,
    value::{print_value, values_equal, Value},
};

const STACK_SIZE: usize = 256;

pub struct VM {
    pub chunk: Chunk, // TODO: use &?
    pub ip: usize,
    pub stack: Vec<Value>,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl VM {
    pub fn new() -> VM {
        VM {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::with_capacity(STACK_SIZE), // = reset stack
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretResult> {
        let mut parser = Parser::new(source);

        if !parser.compile() {
            return Err(InterpretResult::CompileError);
        }

        self.chunk = parser.chunk;
        self.ip = 0; // or self.chunk.code?

        self.run()
    }

    // We run every single instruction here, so this is the most performance critical part of the VM.
    // TODO: look up “direct threaded code”, “jump table”, and “computed goto” for optimization techniques
    fn run(&mut self) -> Result<(), InterpretResult> {
        loop {
            let op = &self.chunk.code[self.ip];
            match op {
                OpCode::Constant(idx) => {
                    let constant = &self.chunk.constants.values[*idx as usize];
                    print_value(constant);
                    self.stack.push(*constant);
                    println!();
                }
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(Value::Bool(values_equal(a, b)));
                }
                OpCode::Greater => {
                    self.binary_op_bool(|x, y| x > y)?;
                }
                OpCode::Less => {
                    self.binary_op_bool(|x, y| x < y)?;
                }
                OpCode::Add => {
                    self.binary_op(|x, y| x + y)?;
                }
                OpCode::Subtract => {
                    self.binary_op(|x, y| x - y)?;
                }
                OpCode::Multiply => {
                    self.binary_op(|x, y| x * y)?;
                }
                OpCode::Divide => {
                    self.binary_op(|x, y| x / y)?;
                }
                OpCode::Not => {
                    let val = self.pop();
                    self.stack.push(Value::Bool(self.is_falsey(val)))
                }
                OpCode::Negate => {
                    if let Value::Number(val) = self.peek(0) {
                        let neg_val = -val;
                        self.pop();
                        self.stack.push(Value::Number(neg_val));
                    } else {
                        return self.runtime_error("Operand must be a number.");
                    }
                }
                OpCode::Return => {
                    print_value(&self.pop());
                    println!();
                    return Ok(());
                }
            }
            self.ip += 1;
        }
    }

    // helper function for popping stack
    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Empty stack")
    }

    fn peek(&self, distance: usize) -> &Value {
        return self
            .stack
            .get(self.stack.len() - 1 - distance)
            .expect("Failed to peek");
    }

    fn is_falsey(&self, value: Value) -> bool {
        match value {
            Value::Bool(b) => !b,
            Value::Nil => true,
            _ => false,
        }
    }

    // wrap in Result, so that we can use the question mark operator to:
    // 1. *Return* InterpretResult if error
    // 2. Unpacks the Result ((), i.e. do nothing) if no error
    fn binary_op(&mut self, f: fn(f64, f64) -> f64) -> Result<(), InterpretResult> {
        match (self.pop(), self.pop()) {
            // note: the first pop returns the right operand
            (Value::Number(b), Value::Number(a)) => {
                self.stack.push(Value::Number(f(a, b)));
                Ok(())
            }
            (b, a) => {
                // Push them back on the stack
                // TODO: Unnecessary? Runtime failure will crash program anyway
                self.stack.push(a);
                self.stack.push(b);
                self.runtime_error("Operands must be numbers.")
            }
        }
    }

    // TODO: maybe can use macro to minize repetition in binary_op and binary_op_bool
    // The closure return type can be generic so that f can return f64/bool
    fn binary_op_bool(&mut self, f: fn(f64, f64) -> bool) -> Result<(), InterpretResult> {
        match (self.pop(), self.pop()) {
            // note: the first pop returns the right operand
            (Value::Number(b), Value::Number(a)) => {
                self.stack.push(Value::Bool(f(a, b)));
                Ok(())
            }
            (b, a) => {
                // Push them back on the stack
                // TODO: Unnecessary? Runtime failure will crash program anyway
                self.stack.push(a);
                self.stack.push(b);
                self.runtime_error("Operands must be boolean.")
            }
        }
    }

    pub fn debug_trace_execution(&self) {
        print!("          ");
        for slot in &self.stack {
            print!("[ ");
            print_value(slot);
            print!(" ]");
        }
        println!();
        disassemble_instruction(&self.chunk, self.ip);
    }

    // Note: All errors are fatal and immediately halt the interpreter.
    // No variadic functions in rust
    fn runtime_error(&self, msg: &str) -> Result<(), InterpretResult> {
        eprintln!("{}", msg);
        let instruction = self.ip - 1;
        let line = self.chunk.lines[instruction];
        eprintln!("[line {}] in script", line);
        // resetStack(); // TODO: no need?
        Err(InterpretResult::RuntimeError)
    }
}
