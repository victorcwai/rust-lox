use crate::interner::Interner;

static ERR_MARGIN: f64 = f64::EPSILON;

// Enum = tagged union in Rust
// Ref: http://patshaughnessy.net/2018/3/15/how-rust-implements-tagged-unions
#[derive(Clone, Copy, Debug)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    // enum and the ref to String are on the stack,
    // while the actual String is stored on the heap
    StringObj(u32),  // u32 = idx in string intern vec
    Identifier(u32), // u32 = idx in string intern vec
    Function(usize), // = idx in the function list in VM
}

// The constant pool is an array of values. The instruction to load a constant looks up the value by index in that array.
pub struct ValueArray {
    pub values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> ValueArray {
        ValueArray { values: Vec::new() }
    }

    pub fn write(&mut self, v: Value) {
        self.values.push(v);
    }
}

pub fn print_value(value: &Value, interner: &Interner) {
    match value {
        Value::Bool(n) => print!("bool: {:?}", n),
        Value::Nil => print!("nil"),
        Value::Number(n) => print!("number: {:?}", n),
        Value::StringObj(s) => print!("StringObj: {:?}: {}", s, interner.lookup(*s)),
        Value::Identifier(s) => print!("Identifier: {:?}: {}", s, interner.lookup(*s)),
        Value::Function(s) => print!("Function id: {:?}", s),
    }
}

pub fn values_equal(av: Value, bv: Value) -> bool {
    match (av, bv) {
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        (Value::Number(a), Value::Number(b)) => (a - b).abs() < ERR_MARGIN,
        (Value::StringObj(a), Value::StringObj(b)) => a == b,
        (Value::Function(a), Value::Function(b)) => a == b,
        _ => false,
    }
}
