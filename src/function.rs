use crate::{chunk::Chunk, interner::StringObjIdx};

pub struct Function {
    pub arity: u8, // # of parameters
    pub chunk: Chunk,
    pub name: Option<StringObjIdx>,
}

impl Function {
    pub fn new() -> Function {
        Function {
            arity: 0,
            chunk: Chunk::new(),
            name: None,
        }
    }
}
