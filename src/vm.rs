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

pub struct CallFrame {
    pub function: Function,
    pub ip: usize,         // ip of the caller
    pub slots: Vec<Value>, // local slots of this frame
}

impl CallFrame {
    fn new(function: Function) -> Self {
        CallFrame {
            function,
            ip: 0,
            slots: Vec::with_capacity(USIZE_COUNT),
        }
    }
}

pub struct VM {
    pub frames: Vec<CallFrame>,
    pub interner: Interner,
    pub stack: Vec<Value>,
    pub globals: HashMap<u32, Value>, // u32 is interner idx
}

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
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretResult> {
        let parser = Parser::new(source, &mut self.interner);

        match parser.compile() {
            Some(function) => {
                // TODO: self.stack.push(Value::) // push the function on the stack
                self.frames.push(CallFrame::new(function));
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
        let mut frame = self.frames.pop().unwrap(); // TODO: fix this
        loop {
            let op = frame.function.chunk.code[frame.ip];
            match op {
                OpCode::Constant(idx) => {
                    let constant = frame.function.chunk.constants.values[idx as usize];
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
                    let constant = frame.function.chunk.constants.values[idx as usize];
                    if let Value::Identifier(name) = constant {
                        self.globals.insert(name, *self.peek(0));
                        self.stack.pop(); //TODO: pop wat?
                    } else {
                        return self.runtime_error("constant is not Value::Identifier!");
                    }
                }
                OpCode::GetGlobal(idx) => {
                    let constant = frame.function.chunk.constants.values[idx as usize];
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
                    let constant = frame.function.chunk.constants.values[idx as usize];
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
                    self.stack.push(frame.slots[idx as usize]);
                }
                OpCode::SetLocal(idx) => {
                    frame.slots[idx as usize] = *self.peek(0);
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
                    frame.ip += offset;
                }
                OpCode::JumpIfFalse(offset) => {
                    if self.is_falsey(self.peek(0)) {
                        frame.ip += offset;
                    }
                }
                OpCode::Loop(offset) => {
                    frame.ip -= offset + 1;
                }
                OpCode::Return => {
                    // Exit interpreter.
                    return Ok(());
                }
            }
            frame.ip += 1;
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
        let frame = &self.frames.last().unwrap();
        let instruction = frame.ip - 1;
        let line = frame.function.chunk.lines[instruction];
        eprintln!("[line {}] in script", line);
        // resetStack(); // TODO: no need?
        Err(InterpretResult::RuntimeError)
    }
}
