Implementation of clox (bytecode) in Robert Nystrom's *Crafting Interpreters* book.

I am writing it while learning Rust, so it is definitely not perfect/idiomatic.

Notes are mostly copied from the https://craftinginterpreters.com/ book.

Run by `cargo run`. Test with `cargo test`.

# Difference between rust-lox and clox #
- Op instruction is implemented with the `OpCode` enum (instead of `u8`), which could be > 1 byte. A chunk has a `Vec` of `OpCode`. 
  - Different offset calculation