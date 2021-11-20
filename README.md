A compiler written in Rust for the Lox programming language. 

Implementation of clox (bytecode) in [Robert Nystrom's *Crafting Interpreters* book](https://craftinginterpreters.com/).

I am writing it while learning Rust, so it is definitely not perfect/idiomatic.

Run by `cargo run`. Run with debug mode by `cargo run --all-features`.

Test with `cargo test`.

# Difference between rust-lox and clox #
- Op instruction is implemented with the `OpCode` enum (instead of `u8`), which could be > 1 byte. A chunk has a `Vec` of `OpCode`. 
  - Different offset calculation
  - Instead of reading 2 bytes, `OpCode::Constant`, `OpCode::GetGlobal` and `OpCode::DefineGlobal` includes a `u8` as the extra byte
- Use `usize` index instead of pointer+dereference to access element in array.
  - Though pointer+dereference should be faster?
- Tagged union replaced by Enum(T)
- No `Value::Obj` that can save arbitary object
- String Object (`Value::StringObj(u32)`) is interned by `HashMap<String, u32>`
- No printing for `Function` object