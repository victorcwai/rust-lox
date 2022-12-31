use crate::compiler::Parser;
use crate::compiler::USIZE_COUNT;
use crate::function::Function;
use crate::interner::Interner;
use crate::{
    chunk::OpCode,
    value::{print_value, values_equal, Value},
};
use std::collections::HashMap;

const STACK_SIZE: usize = FRAMES_MAX * USIZE_COUNT;
const FRAMES_MAX: usize = 64;

#[derive(Clone, Copy)]
pub struct CallFrame {
    pub f_idx: usize,
    pub ip: usize,          // ip of the caller (local frame index, not VM index)
    pub slot_offset: usize, // offset of slots, i.e. starting position of this CallFrame's stack
}

impl CallFrame {
    fn new(f_idx: usize, current_slot: usize) -> Self {
        CallFrame {
            f_idx,
            ip: 0,
            slot_offset: current_slot,
        }
    }
}

pub struct VM {
    pub frames: Vec<CallFrame>,
    pub interner: Interner,
    pub stack: Vec<Value>,
    pub globals: HashMap<u32, Value>, // u32 is interner idx
    pub functions: Vec<Function>,
}

#[derive(PartialEq, Debug)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl VM {
    pub fn new() -> VM {
        VM {
            frames: Vec::with_capacity(FRAMES_MAX),
            interner: Interner::default(),
            stack: Vec::with_capacity(STACK_SIZE), // = reset stack
            globals: HashMap::with_capacity(STACK_SIZE),
            functions: Vec::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretResult> {
        let parser = Parser::new(source, &mut self.interner, &mut self.functions);

        match parser.compile() {
            Some(function) => {
                // push top-level script to the functions Vec
                // at this point, the functions Vec is empty
                self.functions.push(function);
                let top_level_f_idx = self.functions.len() - 1;
                self.frames.push(CallFrame::new(top_level_f_idx, 0));
            }
            None => return Err(InterpretResult::CompileError),
        }

        self.run()
    }

    // We run every single instruction here, so this is the most performance critical part of the VM.
    // TODO: look up “direct threaded code”, “jump table”, and “computed goto” for optimization techniques
    fn run(&mut self) -> Result<(), InterpretResult> {
        // wrap in Result, so that we can use the question mark operator to:
        // 1. *Return* InterpretResult if error
        // 2. Unpacks the Result ((), i.e. do nothing) if no error

        // let mut frame = self.frames.last_mut().unwrap();
        // `frame` has to be mutable/owned, so in the below code when it tries to borrow the mutable
        // it "cannot borrow `*self` as mutable more than once at a time second mutable borrow occurs"
        // so instead of using a single reference `frame`,
        // we call a mutable/immutable reference to the last frame whenever we need it

        // TODO: refactor self.frames.last().unwrap() and self.frames.last_mut().unwrap() into a single function
        loop {
            let op = self.functions[self.frames.last().unwrap().f_idx].chunk.code
                [self.frames.last().unwrap().ip];
            match op {
                OpCode::Constant(idx) => {
                    let constant = self.functions[self.frames.last().unwrap().f_idx]
                        .chunk
                        .constants
                        .values[idx as usize];
                    print_value(&constant, &self.interner);
                    self.stack.push(constant);
                    println!();
                }
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::DefineGlobal(idx) => {
                    let constant = self.functions[self.frames.last().unwrap().f_idx]
                        .chunk
                        .constants
                        .values[idx as usize];
                    if let Value::Identifier(name) = constant {
                        self.globals.insert(name, *self.peek(0));
                        self.stack.pop(); //TODO: pop wat?
                    } else {
                        return self.runtime_error("constant is not Value::Identifier!");
                    }
                }
                OpCode::GetGlobal(idx) => {
                    let constant = self.functions[self.frames.last().unwrap().f_idx]
                        .chunk
                        .constants
                        .values[idx as usize];
                    if let Value::Identifier(name) = constant {
                        if let Some(v) = self.globals.get(&name) {
                            self.stack.push(v.to_owned());
                        } else {
                            let msg = format!("Undefined variable {}.", name);
                            return self.runtime_error(&msg);
                        }
                    } else {
                        return self.runtime_error("constant is not Value::Identifier!");
                    }
                }
                OpCode::SetGlobal(idx) => {
                    let constant = self.functions[self.frames.last().unwrap().f_idx]
                        .chunk
                        .constants
                        .values[idx as usize];
                    if let Value::Identifier(name) = constant {
                        if self.globals.contains_key(&name) {
                            self.globals.insert(name, *self.peek(0));
                            // no pop -> in case the assignment is nested inside some larger expression
                        } else {
                            let msg = format!("Cannot assign to undefined variable {}.", name);
                            return self.runtime_error(&msg);
                        }
                    } else {
                        return self.runtime_error("constant is not Value::Identifier!");
                    }
                }
                OpCode::GetLocal(idx) => {
                    let idx = self.frames.last().unwrap().slot_offset + idx as usize;
                    self.stack.push(self.stack[idx]);
                }
                OpCode::SetLocal(idx) => {
                    let idx = self.frames.last().unwrap().slot_offset + idx as usize;
                    self.stack[idx] = *self.peek(0);
                }
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(Value::Bool(values_equal(a, b)));
                }
                OpCode::Greater => {
                    self.binary_op(|x, y| x > y, Value::Bool)?;
                }
                OpCode::Less => {
                    self.binary_op(|x, y| x < y, Value::Bool)?;
                }
                OpCode::Add => match (self.peek(0), self.peek(1)) {
                    (Value::Number(_), Value::Number(_)) => {
                        self.binary_op(|x, y| x + y, Value::Number)?;
                    }
                    (Value::StringObj(_), Value::StringObj(_)) => {
                        self.concatenate()?;
                    }
                    _ => return self.runtime_error("Operand must be a number."),
                },
                OpCode::Subtract => {
                    self.binary_op(|x, y| x - y, Value::Number)?;
                }
                OpCode::Multiply => {
                    self.binary_op(|x, y| x * y, Value::Number)?;
                }
                OpCode::Divide => {
                    self.binary_op(|x, y| x / y, Value::Number)?;
                }
                OpCode::Not => {
                    let val = self.pop();
                    self.stack.push(Value::Bool(self.is_falsey(&val)))
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
                OpCode::Print => {
                    print!("OpCode::Print: ");
                    print_value(&self.pop(), &self.interner);
                    println!();
                }
                OpCode::Jump(offset) => {
                    self.frames.last_mut().unwrap().ip += offset;
                }
                OpCode::JumpIfFalse(offset) => {
                    if self.is_falsey(self.peek(0)) {
                        self.frames.last_mut().unwrap().ip += offset;
                    }
                }
                OpCode::Loop(offset) => {
                    self.frames.last_mut().unwrap().ip -= offset + 1;
                }
                OpCode::Return => {
                    // When a function returns a value, that value will be on top of the stack.
                    // We’re about to discard the called function’s entire stack window,
                    // so we pop that return value off and hang on to it.
                    let ret_val = self.pop();
                    // Then we discard the CallFrame for the current returning function.
                    self.frames.pop();
                    // If that was the very last CallFrame, it means we’ve finished executing the top-level code.
                    // The entire program is done, so we pop the main script function from the stack and then exit the interpreter.
                    if self.frames.is_empty() {
                        self.stack.pop();
                        return Ok(());
                    }
                    // Otherwise, we discard all of the slots the callee was using for its parameters and local variables.
                    // Then we push the return value back onto the stack, where the caller can find it.
                    self.stack.truncate(self.frames.last().unwrap().slot_offset); // TODO: check if correct
                    self.stack.push(ret_val);
                    // frame = *self.frames.last().unwrap(); // switch back to caller
                    // no need, because we will always get the last frame in the next iteration, and we just popped the last one
                }
                OpCode::Call(arg_count) => {
                    if !self.call_value(*self.peek(arg_count.into()), arg_count) {
                        return Err(InterpretResult::RuntimeError);
                    }
                    // frame = *self.frames.last().unwrap(); // switch to new CallFrame
                    // no need, because we will always get the last frame in the next iteration, and we just pushed the new one
                    continue; // don't increment self.frames.last().unwrap().ip if this is a new call
                }
            }
            self.frames.last_mut().unwrap().ip += 1;
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

    fn call(&mut self, f_idx: usize, arg_count: u8) -> bool {
        if arg_count != self.functions[f_idx].arity {
            let msg = format!(
                "Expected {} arguments but got {}.",
                self.functions[f_idx].arity, arg_count
            );
            self.runtime_error(&msg);
            return false;
        }
        if self.frames.len() == FRAMES_MAX {
            self.runtime_error("Stack overflow.");
            return false;
        }
        let frame = CallFrame::new(f_idx, self.stack.len() - arg_count as usize - 1);
        self.frames.push(frame);
        true
    }

    fn call_value(&mut self, callee: Value, arg_count: u8) -> bool {
        match callee {
            Value::Function(f_idx) => self.call(f_idx, arg_count),
            _ => {
                self.runtime_error("Can only call functions and classes.");
                false
            }
        }
    }

    fn is_falsey(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => !b,
            Value::Nil => true,
            _ => false,
        }
    }

    fn concatenate(&mut self) -> Result<(), InterpretResult> {
        match (self.pop(), self.pop()) {
            // note: the first pop returns the right operand
            (Value::StringObj(b), Value::StringObj(a)) => {
                let b_str = self.interner.lookup(b);
                let a_str = self.interner.lookup(a);
                let res = a_str.to_owned() + b_str;
                let res_idx = self.interner.intern_string(res);
                self.stack.push(Value::StringObj(res_idx));
                Ok(())
            }
            (b, a) => {
                // Push them back on the stack
                // TODO: Unnecessary? Runtime failure will crash program anyway
                self.stack.push(a);
                self.stack.push(b);
                self.runtime_error("Operands must be two strings.")
            }
        }
    }

    fn binary_op<T>(
        &mut self,
        f: fn(f64, f64) -> T,
        convert: fn(T) -> Value,
    ) -> Result<(), InterpretResult> {
        match (self.pop(), self.pop()) {
            // note: the first pop returns the right operand
            (Value::Number(b), Value::Number(a)) => {
                self.stack.push(convert(f(a, b)));
                Ok(())
            }
            (b, a) => {
                // Push them back on the stack
                // TODO: Unnecessary? Runtime failure will crash program anyway
                self.stack.push(a);
                self.stack.push(b);
                self.runtime_error("Operands must be two numbers.")
            }
        }
    }

    // pub fn debug_trace_execution(&self) {
    //     print!("          ");
    //     for slot in &self.stack {
    //         print!("[ ");
    //         print_value(slot, &self.interner);
    //         print!(" ]");
    //     }
    //     println!();
    //     disassemble_instruction(&self.chunk, self.ip);
    // }

    // Note: All errors are fatal and immediately halt the interpreter.
    // No variadic functions in rust
    fn runtime_error(&self, msg: &str) -> Result<(), InterpretResult> {
        eprintln!("{}", msg);

        for frame in self.frames.iter().rev() {
            let instruction = frame.ip - 1;
            let line = self.functions[frame.f_idx].chunk.lines[instruction];
            if self.functions[frame.f_idx].name.is_some() {
                let name = self
                    .interner
                    .lookup(self.functions[frame.f_idx].name.unwrap());
                eprintln!("[line {}] in {}()", line, name);
            } else {
                eprintln!("[line {}] in script", line);
            }
        }
        Err(InterpretResult::RuntimeError)
    }
}
