A compiler written in Rust for the Lox programming language. 

Implementation of clox (bytecode) in [Robert Nystrom's *Crafting Interpreters* book](https://craftinginterpreters.com/).

I am writing it while learning Rust, so it is definitely not perfect/idiomatic.

Run by `cargo run`. Test with `cargo test`.

# Difference between rust-lox and clox #
- Op instruction is implemented with the `OpCode` enum (instead of `u8`), which could be > 1 byte. A chunk has a `Vec` of `OpCode`. 
  - Different offset calculation
- Use `usize` index instead of pointer+dereference to access element in array.
  - Though pointer+dereference should be faster?
